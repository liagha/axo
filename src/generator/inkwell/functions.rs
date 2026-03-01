use {
    super::{Backend, Entity},
    crate::{
        data::Str,
        internal::hash::Map,
        schema::*,
    },
    inkwell::{
        types::BasicType,
        values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue},
        FloatPredicate, InlineAsmDialect, IntPredicate,
    },
};
use crate::analyzer::{Analysis, Instruction};
use crate::checker::TypeKind;

impl<'backend> super::Inkwell<'backend> {
    fn has_terminator(&self) -> bool {
        self.builder
            .get_insert_block()
            .and_then(|block| block.get_terminator())
            .is_some()
    }

    fn coerce(
        &mut self,
        function: FunctionValue<'backend>,
        value: BasicValueEnum<'backend>,
    ) -> BasicValueEnum<'backend> {
        let expected = match function.get_type().get_return_type() {
            Some(kind) => kind,
            None => return value,
        };

        if value.get_type() == expected {
            return value;
        }

        match (value, expected) {
            (BasicValueEnum::IntValue(int), expected) if expected.is_int_type() => self
                .builder
                .build_int_cast(int, expected.into_int_type(), "ret_cast_int")
                .unwrap()
                .into(),
            (BasicValueEnum::FloatValue(float), expected) if expected.is_float_type() => self
                .builder
                .build_float_cast(float, expected.into_float_type(), "ret_cast_float")
                .unwrap()
                .into(),
            (BasicValueEnum::IntValue(int), expected) if expected.is_float_type() => self
                .builder
                .build_signed_int_to_float(int, expected.into_float_type(), "ret_int_to_float")
                .unwrap()
                .into(),
            (BasicValueEnum::FloatValue(float), expected) if expected.is_int_type() => self
                .builder
                .build_float_to_signed_int(float, expected.into_int_type(), "ret_float_to_int")
                .unwrap()
                .into(),
            (_, expected) => expected.const_zero().into(),
        }
    }

    fn truthy(&mut self, value: BasicValueEnum<'backend>) -> inkwell::values::IntValue<'backend> {
        if value.is_int_value() {
            let int = value.into_int_value();
            if int.get_type().get_bit_width() == 1 {
                int
            } else {
                self.builder
                    .build_int_compare(
                        IntPredicate::NE,
                        int,
                        int.get_type().const_zero(),
                        "if_cond",
                    )
                    .unwrap()
            }
        } else if value.is_float_value() {
            let float = value.into_float_value();
            self.builder
                .build_float_compare(
                    FloatPredicate::ONE,
                    float,
                    float.get_type().const_zero(),
                    "if_cond",
                )
                .unwrap()
        } else {
            self.context.bool_type().const_zero()
        }
    }

    fn linux_syscall3(
        &mut self,
        number: inkwell::values::IntValue<'backend>,
        arg0: inkwell::values::IntValue<'backend>,
        arg1: inkwell::values::IntValue<'backend>,
        arg2: inkwell::values::IntValue<'backend>,
        label: &str,
    ) -> inkwell::values::IntValue<'backend> {
        let i64_type = self.context.i64_type();
        let function_type = i64_type.fn_type(
            &[
                i64_type.into(),
                i64_type.into(),
                i64_type.into(),
                i64_type.into(),
            ],
            false,
        );
        #[cfg(target_arch = "aarch64")]
        let (assembly, constraints) = (
            "svc #0".to_string(),
            "={x0},{x8},{x0},{x1},{x2},~{memory}".to_string(),
        );
        #[cfg(target_arch = "x86_64")]
        let (assembly, constraints) = (
            "syscall".to_string(),
            "=r,{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}".to_string(),
        );
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        let (assembly, constraints) = (
            "syscall".to_string(),
            "=r,{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}".to_string(),
        );
        let syscall = self.context.create_inline_asm(
            function_type,
            assembly,
            constraints,
            true,
            false,
            Some(InlineAsmDialect::ATT),
            false,
        );
        let call = self
            .builder
            .build_indirect_call(
                function_type,
                syscall,
                &[number.into(), arg0.into(), arg1.into(), arg2.into()],
                label,
            )
            .unwrap();

        call.try_as_basic_value().basic()
            .map(|value| value.into_int_value())
            .unwrap_or_else(|| i64_type.const_zero())
    }

    fn syscall_read_number(&self) -> u64 {
        #[cfg(target_arch = "aarch64")]
        {
            return 63;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            return 0;
        }
    }

    fn syscall_write_number(&self) -> u64 {
        #[cfg(target_arch = "aarch64")]
        {
            return 64;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            return 1;
        }
    }

    fn syscall_exit_number(&self) -> u64 {
        #[cfg(target_arch = "aarch64")]
        {
            return 93;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            return 60;
        }
    }

    pub(crate) fn emit_bootstrap_start(&mut self, main: FunctionValue<'backend>) {
        if self.module.get_function("_start").is_some() {
            return;
        }

        let start = self
            .module
            .add_function("_start", self.context.void_type().fn_type(&[], false), None);
        let entry = self.context.append_basic_block(start, "entry");
        let previous = self.builder.get_insert_block();
        self.builder.position_at_end(entry);

        let status = self
            .builder
            .build_call(main, &[], "main_status")
            .unwrap()
            .try_as_basic_value().basic()
            .map(|value| self.to_i64(value, "exit_status"))
            .unwrap_or_else(|| self.context.i64_type().const_zero());

        let zero = self.context.i64_type().const_zero();
        let _ = self.linux_syscall3(
            self.context
                .i64_type()
                .const_int(self.syscall_exit_number(), false),
            status,
            zero,
            zero,
            "exit_call",
        );

        self.builder.build_unreachable().unwrap();

        if let Some(block) = previous {
            self.builder.position_at_end(block);
        }
    }

    fn runtime_strlen_function(&mut self) -> FunctionValue<'backend> {
        if let Some(function) = self.module.get_function("axo_strlen") {
            return function;
        }

        let pointer = self.context.ptr_type(inkwell::AddressSpace::default());
        let function = self.module.add_function(
            "axo_strlen",
            self.context.i64_type().fn_type(&[pointer.into()], false),
            None,
        );
        self.entities
            .insert(Str::from("axo_strlen"), Entity::Function(function));

        let caller_block = self.builder.get_insert_block();
        let entry = self.context.append_basic_block(function, "entry");
        let check = self.context.append_basic_block(function, "check");
        let step = self.context.append_basic_block(function, "step");
        let done = self.context.append_basic_block(function, "done");
        self.builder.position_at_end(entry);

        let i64_type = self.context.i64_type();
        let i8_type = self.context.i8_type();
        let index = self.builder.build_alloca(i64_type, "strlen_index").unwrap();
        self.builder.build_store(index, i64_type.const_zero());
        self.builder.build_unconditional_branch(check).unwrap();

        self.builder.position_at_end(check);
        let value = function
            .get_first_param()
            .map(|item| item.into_pointer_value())
            .unwrap_or_else(|| pointer.const_null());
        let current = self
            .builder
            .build_load(i64_type, index, "strlen_current")
            .unwrap()
            .into_int_value();
        let char_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(i8_type, value, &[current], "strlen_char_ptr")
                .unwrap()
        };
        let character = self
            .builder
            .build_load(i8_type, char_ptr, "strlen_char")
            .unwrap()
            .into_int_value();
        let is_zero = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                character,
                i8_type.const_zero(),
                "strlen_is_zero",
            )
            .unwrap();
        self.builder
            .build_conditional_branch(is_zero, done, step)
            .unwrap();

        self.builder.position_at_end(step);
        let next = self
            .builder
            .build_int_add(current, i64_type.const_int(1, false), "strlen_next")
            .unwrap();
        self.builder.build_store(index, next);
        self.builder.build_unconditional_branch(check).unwrap();

        self.builder.position_at_end(done);
        let length = self
            .builder
            .build_load(i64_type, index, "strlen_result")
            .unwrap()
            .into_int_value();
        self.builder.build_return(Some(&length));

        if let Some(block) = caller_block {
            self.builder.position_at_end(block);
        }

        function
    }

    fn runtime_read_buffer_ptr(&mut self) -> inkwell::values::PointerValue<'backend> {
        let array_type = self.context.i8_type().array_type(4096);
        let global = self
            .module
            .get_global("axo_read_line_buffer")
            .unwrap_or_else(|| {
                let global = self
                    .module
                    .add_global(array_type, None, "axo_read_line_buffer");
                global.set_initializer(&array_type.const_zero());
                global
            });

        unsafe {
            self.builder
                .build_in_bounds_gep(
                    array_type,
                    global.as_pointer_value(),
                    &[
                        self.context.i32_type().const_zero(),
                        self.context.i32_type().const_zero(),
                    ],
                    "axo_read_line_buffer_ptr",
                )
                .unwrap()
        }
    }

    fn runtime_print_raw_function(
        &mut self,
        symbol: &str,
        key: Str<'backend>,
        fd: i32,
    ) -> FunctionValue<'backend> {
        if let Some(function) = self.module.get_function(symbol) {
            return function;
        }

        let pointer = self.context.ptr_type(inkwell::AddressSpace::default());
        let function = self.module.add_function(
            symbol,
            self.context.i32_type().fn_type(&[pointer.into()], false),
            None,
        );
        self.entities.insert(key, Entity::Function(function));

        let caller_block = self.builder.get_insert_block();
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        let input = function
            .get_first_param()
            .map(|value| value.into_pointer_value())
            .unwrap_or_else(|| {
                self.builder
                    .build_global_string_ptr("", "print_raw_default")
                    .unwrap()
                    .as_pointer_value()
            });

        let strlen = self.runtime_strlen_function();
        let length = self
            .builder
            .build_call(strlen, &[BasicMetadataValueEnum::from(input)], "strlen")
            .unwrap()
            .try_as_basic_value().basic()
            .map(|value| value.into_int_value())
            .unwrap_or_else(|| self.context.i64_type().const_zero());

        let input_ptr = self
            .builder
            .build_ptr_to_int(input, self.context.i64_type(), "write_ptr_int")
            .unwrap();
        let written = self.linux_syscall3(
            self.context
                .i64_type()
                .const_int(self.syscall_write_number(), false),
            self.context.i64_type().const_int(fd as u64, false),
            input_ptr,
            length,
            "write_io",
        );

        let status = self
            .builder
            .build_int_cast(written, self.context.i32_type(), "write_status")
            .unwrap();
        self.builder.build_return(Some(&status));

        if let Some(block) = caller_block {
            self.builder.position_at_end(block);
        }

        function
    }

    fn runtime_println_function(
        &mut self,
        symbol: &str,
        key: Str<'backend>,
        raw_symbol: &str,
        raw_key: Str<'backend>,
        fd: i32,
    ) -> FunctionValue<'backend> {
        if let Some(function) = self.module.get_function(symbol) {
            return function;
        }

        let pointer = self.context.ptr_type(inkwell::AddressSpace::default());
        let function = self.module.add_function(
            symbol,
            self.context.i32_type().fn_type(&[pointer.into()], false),
            None,
        );
        self.entities.insert(key, Entity::Function(function));

        let caller_block = self.builder.get_insert_block();
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        let raw = self.runtime_print_raw_function(raw_symbol, raw_key, fd);
        let input = function
            .get_first_param()
            .map(|value| value.into_pointer_value())
            .unwrap_or_else(|| {
                self.builder
                    .build_global_string_ptr("", "println_default")
                    .unwrap()
                    .as_pointer_value()
            });

        let _ = self
            .builder
            .build_call(raw, &[BasicMetadataValueEnum::from(input)], "print_body")
            .unwrap();

        let newline = self
            .builder
            .build_global_string_ptr("\n", "print_newline")
            .unwrap()
            .as_pointer_value();
        let result = self
            .builder
            .build_call(
                raw,
                &[BasicMetadataValueEnum::from(newline)],
                "print_newline_call",
            )
            .unwrap()
            .try_as_basic_value().basic()
            .map(|value| value.into_int_value())
            .unwrap_or_else(|| self.context.i32_type().const_zero());

        self.builder.build_return(Some(&result));

        if let Some(block) = caller_block {
            self.builder.position_at_end(block);
        }

        function
    }

    fn runtime_string_io_function(
        &mut self,
        symbol: &str,
        key: Str<'backend>,
    ) -> FunctionValue<'backend> {
        match symbol {
            "axo_print_raw" => self.runtime_print_raw_function(symbol, key, 1),
            "axo_eprint_raw" => self.runtime_print_raw_function(symbol, key, 2),
            "axo_println" => self.runtime_println_function(
                symbol,
                key,
                "axo_print_raw",
                Str::from("axo_print_raw"),
                1,
            ),
            "axo_eprintln" => self.runtime_println_function(
                symbol,
                key,
                "axo_eprint_raw",
                Str::from("axo_eprint_raw"),
                2,
            ),
            _ => self.runtime_print_raw_function(symbol, key, 1),
        }
    }

    fn runtime_read_line_function(&mut self) -> FunctionValue<'backend> {
        if let Some(function) = self.module.get_function("axo_read_line") {
            return function;
        }

        let pointer = self.context.ptr_type(inkwell::AddressSpace::default());
        let function = self
            .module
            .add_function("axo_read_line", pointer.fn_type(&[], false), None);
        self.entities
            .insert(Str::from("axo_read_line"), Entity::Function(function));

        let caller_block = self.builder.get_insert_block();
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        let limit = self.context.i64_type().const_int(4095, false);
        let zero = self.context.i64_type().const_zero();
        let buffer = self.runtime_read_buffer_ptr();
        let buffer_int = self
            .builder
            .build_ptr_to_int(buffer, self.context.i64_type(), "read_ptr_int")
            .unwrap();
        let bytes_read = self.linux_syscall3(
            self.context
                .i64_type()
                .const_int(self.syscall_read_number(), false),
            self.context.i64_type().const_zero(),
            buffer_int,
            limit,
            "read_line",
        );

        let positive = self
            .builder
            .build_int_compare(IntPredicate::SGT, bytes_read, zero, "read_positive")
            .unwrap();
        let count = self
            .builder
            .build_select(positive, bytes_read, zero, "read_count")
            .unwrap()
            .into_int_value();
        let has_bytes = self
            .builder
            .build_int_compare(IntPredicate::UGT, count, zero, "has_bytes")
            .unwrap();

        let trim = self.context.append_basic_block(function, "read_trim");
        let keep = self.context.append_basic_block(function, "read_keep");
        let finish = self.context.append_basic_block(function, "read_finish");
        self.builder
            .build_conditional_branch(has_bytes, trim, keep)
            .unwrap();

        self.builder.position_at_end(trim);
        let last_index = self
            .builder
            .build_int_sub(
                count,
                self.context.i64_type().const_int(1, false),
                "last_index",
            )
            .unwrap();
        let last_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(self.context.i8_type(), buffer, &[last_index], "line_last")
                .unwrap()
        };
        let last_byte = self
            .builder
            .build_load(self.context.i8_type(), last_ptr, "last_byte")
            .unwrap()
            .into_int_value();
        let is_lf = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                last_byte,
                self.context.i8_type().const_int(b'\n' as u64, false),
                "is_lf",
            )
            .unwrap();
        let is_cr = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                last_byte,
                self.context.i8_type().const_int(b'\r' as u64, false),
                "is_cr",
            )
            .unwrap();
        let is_newline = self.builder.build_or(is_lf, is_cr, "is_newline").unwrap();
        let trimmed_index = self
            .builder
            .build_select(is_newline, last_index, count, "trimmed_index")
            .unwrap()
            .into_int_value();
        self.builder.build_unconditional_branch(finish).unwrap();
        let trim_block = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(keep);
        self.builder.build_unconditional_branch(finish).unwrap();
        let keep_block = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(finish);
        let end_index = self
            .builder
            .build_phi(self.context.i64_type(), "line_end_index")
            .unwrap();
        let trimmed_value: BasicValueEnum<'backend> = trimmed_index.into();
        let count_value: BasicValueEnum<'backend> = count.into();
        end_index.add_incoming(&[(&trimmed_value, trim_block), (&count_value, keep_block)]);
        let end_index = end_index.as_basic_value().into_int_value();
        let end_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(self.context.i8_type(), buffer, &[end_index], "line_end")
                .unwrap()
        };
        self.builder
            .build_store(end_ptr, self.context.i8_type().const_zero());

        self.builder.build_return(Some(&buffer));

        if let Some(block) = caller_block {
            self.builder.position_at_end(block);
        }

        function
    }

    fn emit_string_io(
        &mut self,
        symbol: &str,
        key: Str<'backend>,
        arguments: &[Box<Analysis<'backend>>],
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let callee = self.runtime_string_io_function(symbol, key);
        let value = arguments
            .first()
            .map(|argument| self.instruction(argument.instruction.clone(), function));

        let pointer = match value {
            Some(value) if value.is_pointer_value() => value.into_pointer_value(),
            _ => self
                .builder
                .build_global_string_ptr("", "print_empty")
                .unwrap()
                .as_pointer_value(),
        };

        let result = self
            .builder
            .build_call(callee, &[BasicMetadataValueEnum::from(pointer)], "print")
            .unwrap();

        result
            .try_as_basic_value().basic()
            .unwrap_or(self.context.i32_type().const_zero().into())
    }

    fn emit_pointer_io(
        &mut self,
        symbol: &str,
        key: Str<'backend>,
        pointer: inkwell::values::PointerValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let callee = self.runtime_string_io_function(symbol, key);
        let result = self
            .builder
            .build_call(callee, &[BasicMetadataValueEnum::from(pointer)], "print_ptr")
            .unwrap();

        result
            .try_as_basic_value().basic()
            .unwrap_or(self.context.i32_type().const_zero().into())
    }

    fn runtime_i64_buffer_ptr(&mut self) -> inkwell::values::PointerValue<'backend> {
        let array_type = self.context.i8_type().array_type(64);
        let global = self
            .module
            .get_global("axo_i64_print_buffer")
            .unwrap_or_else(|| {
                let global = self.module.add_global(array_type, None, "axo_i64_print_buffer");
                global.set_initializer(&array_type.const_zero());
                global
            });

        unsafe {
            self.builder
                .build_in_bounds_gep(
                    array_type,
                    global.as_pointer_value(),
                    &[
                        self.context.i32_type().const_zero(),
                        self.context.i32_type().const_zero(),
                    ],
                    "axo_i64_print_buffer_ptr",
                )
                .unwrap()
        }
    }

    fn runtime_f64_buffer_ptr(&mut self) -> inkwell::values::PointerValue<'backend> {
        let array_type = self.context.i8_type().array_type(128);
        let global = self
            .module
            .get_global("axo_f64_print_buffer")
            .unwrap_or_else(|| {
                let global = self.module.add_global(array_type, None, "axo_f64_print_buffer");
                global.set_initializer(&array_type.const_zero());
                global
            });

        unsafe {
            self.builder
                .build_in_bounds_gep(
                    array_type,
                    global.as_pointer_value(),
                    &[
                        self.context.i32_type().const_zero(),
                        self.context.i32_type().const_zero(),
                    ],
                    "axo_f64_print_buffer_ptr",
                )
                .unwrap()
        }
    }

    fn runtime_i64_to_string_function(&mut self) -> FunctionValue<'backend> {
        if let Some(function) = self.module.get_function("axo_i64_to_string") {
            return function;
        }

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();
        let i8_type = self.context.i8_type();
        let pointer = self.context.ptr_type(inkwell::AddressSpace::default());

        let function = self
            .module
            .add_function("axo_i64_to_string", pointer.fn_type(&[i64_type.into()], false), None);
        self.entities
            .insert(Str::from("axo_i64_to_string"), Entity::Function(function));

        let caller_block = self.builder.get_insert_block();
        let entry = self.context.append_basic_block(function, "entry");
        let zero_case = self.context.append_basic_block(function, "zero_case");
        let digit_loop = self.context.append_basic_block(function, "digit_loop");
        let digit_done = self.context.append_basic_block(function, "digit_done");
        let reverse_loop = self.context.append_basic_block(function, "reverse_loop");
        let reverse_done = self.context.append_basic_block(function, "reverse_done");
        self.builder.position_at_end(entry);

        let buffer = self.runtime_i64_buffer_ptr();
        let value = function
            .get_first_param()
            .map(|item| item.into_int_value())
            .unwrap_or_else(|| i64_type.const_zero());
        let index_ptr = self.builder.build_alloca(i64_type, "i64_index").unwrap();
        let left_ptr = self.builder.build_alloca(i64_type, "i64_left").unwrap();
        let right_ptr = self.builder.build_alloca(i64_type, "i64_right").unwrap();
        self.builder.build_store(index_ptr, i64_type.const_zero());

        let is_negative = self
            .builder
            .build_int_compare(IntPredicate::SLT, value, i64_type.const_zero(), "i64_is_negative")
            .unwrap();
        let abs_value = self
            .builder
            .build_select(
                is_negative,
                self.builder.build_int_neg(value, "i64_neg").unwrap(),
                value,
                "i64_abs",
            )
            .unwrap()
            .into_int_value();
        let current_ptr = self.builder.build_alloca(i64_type, "i64_current").unwrap();
        self.builder.build_store(current_ptr, abs_value);

        let is_zero = self
            .builder
            .build_int_compare(IntPredicate::EQ, abs_value, i64_type.const_zero(), "i64_is_zero")
            .unwrap();
        self.builder
            .build_conditional_branch(is_zero, zero_case, digit_loop)
            .unwrap();

        self.builder.position_at_end(zero_case);
        let zero_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(i8_type, buffer, &[i64_type.const_zero()], "i64_zero_ptr")
                .unwrap()
        };
        self.builder
            .build_store(zero_ptr, i8_type.const_int(b'0' as u64, false));
        let one_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(
                    i8_type,
                    buffer,
                    &[i64_type.const_int(1, false)],
                    "i64_one_ptr",
                )
                .unwrap()
        };
        self.builder.build_store(one_ptr, i8_type.const_zero());
        self.builder.build_return(Some(&buffer));

        self.builder.position_at_end(digit_loop);
        let current = self
            .builder
            .build_load(i64_type, current_ptr, "i64_cur")
            .unwrap()
            .into_int_value();
        let remainder = self
            .builder
            .build_int_unsigned_rem(current, i64_type.const_int(10, false), "i64_rem")
            .unwrap();
        let digit = self
            .builder
            .build_int_add(
                remainder,
                i64_type.const_int(b'0' as u64, false),
                "i64_digit_ascii",
            )
            .unwrap();
        let digit8 = self
            .builder
            .build_int_cast(digit, i8_type, "i64_digit8")
            .unwrap();
        let index = self
            .builder
            .build_load(i64_type, index_ptr, "i64_index_load")
            .unwrap()
            .into_int_value();
        let digit_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(i8_type, buffer, &[index], "i64_digit_ptr")
                .unwrap()
        };
        self.builder.build_store(digit_ptr, digit8);
        let next_index = self
            .builder
            .build_int_add(index, i64_type.const_int(1, false), "i64_index_next")
            .unwrap();
        self.builder.build_store(index_ptr, next_index);
        let next_current = self
            .builder
            .build_int_unsigned_div(current, i64_type.const_int(10, false), "i64_div")
            .unwrap();
        self.builder.build_store(current_ptr, next_current);
        let more = self
            .builder
            .build_int_compare(IntPredicate::NE, next_current, i64_type.const_zero(), "i64_more")
            .unwrap();
        self.builder
            .build_conditional_branch(more, digit_loop, digit_done)
            .unwrap();

        self.builder.position_at_end(digit_done);
        let index_after_digits = self
            .builder
            .build_load(i64_type, index_ptr, "i64_index_after_digits")
            .unwrap()
            .into_int_value();
        let signed_index = self
            .builder
            .build_select(
                is_negative,
                self.builder
                    .build_int_add(
                        index_after_digits,
                        i64_type.const_int(1, false),
                        "i64_signed_index",
                    )
                    .unwrap(),
                index_after_digits,
                "i64_final_index",
            )
            .unwrap()
            .into_int_value();
        self.builder.build_store(index_ptr, signed_index);

        let minus_index = self
            .builder
            .build_int_sub(signed_index, i64_type.const_int(1, false), "i64_minus_index")
            .unwrap();
        let minus_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(i8_type, buffer, &[minus_index], "i64_minus_ptr")
                .unwrap()
        };
        let with_minus = self
            .builder
            .build_select(
                is_negative,
                i8_type.const_int(b'-' as u64, false),
                self.builder
                    .build_load(i8_type, minus_ptr, "i64_minus_existing")
                    .unwrap()
                    .into_int_value(),
                "i64_maybe_minus",
            )
            .unwrap();
        self.builder.build_store(minus_ptr, with_minus);

        let nul_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(i8_type, buffer, &[signed_index], "i64_nul_ptr")
                .unwrap()
        };
        self.builder.build_store(nul_ptr, i8_type.const_zero());

        self.builder.build_store(left_ptr, i64_type.const_zero());
        self.builder.build_store(right_ptr, minus_index);
        self.builder.build_unconditional_branch(reverse_loop).unwrap();

        self.builder.position_at_end(reverse_loop);
        let left = self
            .builder
            .build_load(i64_type, left_ptr, "i64_left")
            .unwrap()
            .into_int_value();
        let right = self
            .builder
            .build_load(i64_type, right_ptr, "i64_right")
            .unwrap()
            .into_int_value();
        let continue_reverse = self
            .builder
            .build_int_compare(IntPredicate::ULT, left, right, "i64_continue_reverse")
            .unwrap();
        self.builder
            .build_conditional_branch(continue_reverse, reverse_done, reverse_done)
            .unwrap();

        // Replace the self-branch above with body by mutating current block branch.
        let branch_block = self.builder.get_insert_block().unwrap();
        let reverse_body = self.context.append_basic_block(function, "reverse_body");
        branch_block.get_terminator().unwrap().erase_from_basic_block();
        self.builder.position_at_end(branch_block);
        self.builder
            .build_conditional_branch(continue_reverse, reverse_body, reverse_done)
            .unwrap();

        self.builder.position_at_end(reverse_body);
        let left_ptr_gep = unsafe {
            self.builder
                .build_in_bounds_gep(i8_type, buffer, &[left], "i64_left_ptr")
                .unwrap()
        };
        let right_ptr_gep = unsafe {
            self.builder
                .build_in_bounds_gep(i8_type, buffer, &[right], "i64_right_ptr")
                .unwrap()
        };
        let left_value = self
            .builder
            .build_load(i8_type, left_ptr_gep, "i64_left_value")
            .unwrap();
        let right_value = self
            .builder
            .build_load(i8_type, right_ptr_gep, "i64_right_value")
            .unwrap();
        self.builder.build_store(left_ptr_gep, right_value);
        self.builder.build_store(right_ptr_gep, left_value);
        self.builder.build_store(
            left_ptr,
            self.builder
                .build_int_add(left, i64_type.const_int(1, false), "i64_left_next")
                .unwrap(),
        );
        self.builder.build_store(
            right_ptr,
            self.builder
                .build_int_sub(right, i64_type.const_int(1, false), "i64_right_next")
                .unwrap(),
        );
        self.builder.build_unconditional_branch(reverse_loop).unwrap();

        self.builder.position_at_end(reverse_done);
        self.builder.build_return(Some(&buffer));

        if let Some(block) = caller_block {
            self.builder.position_at_end(block);
        }

        let _ = i32_type; // keep type imports used consistently across targets
        function
    }

    fn runtime_f64_to_string_function(&mut self) -> FunctionValue<'backend> {
        if let Some(function) = self.module.get_function("axo_f64_to_string") {
            return function;
        }

        let i64_type = self.context.i64_type();
        let i8_type = self.context.i8_type();
        let pointer = self.context.ptr_type(inkwell::AddressSpace::default());
        let function = self
            .module
            .add_function("axo_f64_to_string", pointer.fn_type(&[self.context.f64_type().into()], false), None);
        self.entities
            .insert(Str::from("axo_f64_to_string"), Entity::Function(function));

        let caller_block = self.builder.get_insert_block();
        let entry = self.context.append_basic_block(function, "entry");
        let copy_loop = self.context.append_basic_block(function, "copy_loop");
        let frac_loop = self.context.append_basic_block(function, "frac_loop");
        let done = self.context.append_basic_block(function, "done");
        self.builder.position_at_end(entry);

        let value = function
            .get_first_param()
            .map(|item| item.into_float_value())
            .unwrap_or_else(|| self.context.f64_type().const_zero());
        let is_negative = self
            .builder
            .build_float_compare(
                FloatPredicate::OLT,
                value,
                self.context.f64_type().const_zero(),
                "f64_is_negative",
            )
            .unwrap();
        let abs = self
            .builder
            .build_select(
                is_negative,
                self.builder.build_float_neg(value, "f64_neg").unwrap(),
                value,
                "f64_abs",
            )
            .unwrap()
            .into_float_value();

        let whole = self
            .builder
            .build_float_to_signed_int(abs, i64_type, "f64_whole")
            .unwrap();
        let whole_float = self
            .builder
            .build_signed_int_to_float(whole, self.context.f64_type(), "f64_whole_float")
            .unwrap();
        let fraction = self
            .builder
            .build_float_sub(abs, whole_float, "f64_fraction")
            .unwrap();
        let scaled = self
            .builder
            .build_float_mul(
                fraction,
                self.context.f64_type().const_float(1_000_000.0),
                "f64_scaled",
            )
            .unwrap();
        let rounded = self
            .builder
            .build_float_add(
                scaled,
                self.context.f64_type().const_float(0.5),
                "f64_rounded",
            )
            .unwrap();
        let frac_int = self
            .builder
            .build_float_to_signed_int(rounded, i64_type, "f64_frac_int")
            .unwrap();

        let i64_to_str = self.runtime_i64_to_string_function();
        let whole_ptr = self
            .builder
            .build_call(i64_to_str, &[BasicMetadataValueEnum::from(whole)], "whole_str")
            .unwrap()
            .try_as_basic_value().basic()
            .map(|v| v.into_pointer_value())
            .unwrap_or_else(|| pointer.const_null());

        let buffer = self.runtime_f64_buffer_ptr();
        let out_idx_ptr = self.builder.build_alloca(i64_type, "f64_out_idx").unwrap();
        self.builder.build_store(out_idx_ptr, i64_type.const_zero());
        let src_idx_ptr = self.builder.build_alloca(i64_type, "f64_src_idx").unwrap();
        self.builder.build_store(src_idx_ptr, i64_type.const_zero());

        let neg_block = self.context.append_basic_block(function, "f64_neg");
        let after_neg_block = self.context.append_basic_block(function, "f64_after_neg");
        self.builder
            .build_conditional_branch(is_negative, neg_block, after_neg_block)
            .unwrap();

        self.builder.position_at_end(neg_block);
        let idx = self
            .builder
            .build_load(i64_type, out_idx_ptr, "f64_neg_idx")
            .unwrap()
            .into_int_value();
        let ptr = unsafe {
            self.builder
                .build_in_bounds_gep(i8_type, buffer, &[idx], "f64_minus_ptr")
                .unwrap()
        };
        self.builder
            .build_store(ptr, i8_type.const_int(b'-' as u64, false));
        self.builder.build_store(
            out_idx_ptr,
            self.builder
                .build_int_add(idx, i64_type.const_int(1, false), "f64_neg_idx_next")
                .unwrap(),
        );
        self.builder.build_unconditional_branch(after_neg_block).unwrap();

        self.builder.position_at_end(after_neg_block);
        self.builder.build_unconditional_branch(copy_loop).unwrap();
        self.builder.position_at_end(copy_loop);
        let src_idx = self
            .builder
            .build_load(i64_type, src_idx_ptr, "f64_src_idx_load")
            .unwrap()
            .into_int_value();
        let src_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(i8_type, whole_ptr, &[src_idx], "f64_src_ptr")
                .unwrap()
        };
        let src_ch = self
            .builder
            .build_load(i8_type, src_ptr, "f64_src_ch")
            .unwrap()
            .into_int_value();
        let done_copy = self
            .builder
            .build_int_compare(IntPredicate::EQ, src_ch, i8_type.const_zero(), "f64_done_copy")
            .unwrap();
        let copy_body = self.context.append_basic_block(function, "copy_body");
        self.builder
            .build_conditional_branch(done_copy, frac_loop, copy_body)
            .unwrap();

        self.builder.position_at_end(copy_body);
        let out_idx = self
            .builder
            .build_load(i64_type, out_idx_ptr, "f64_out_idx_load")
            .unwrap()
            .into_int_value();
        let out_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(i8_type, buffer, &[out_idx], "f64_out_ptr")
                .unwrap()
        };
        self.builder.build_store(out_ptr, src_ch);
        self.builder.build_store(
            out_idx_ptr,
            self.builder
                .build_int_add(out_idx, i64_type.const_int(1, false), "f64_out_idx_next")
                .unwrap(),
        );
        self.builder.build_store(
            src_idx_ptr,
            self.builder
                .build_int_add(src_idx, i64_type.const_int(1, false), "f64_src_idx_next")
                .unwrap(),
        );
        self.builder.build_unconditional_branch(copy_loop).unwrap();

        self.builder.position_at_end(frac_loop);
        let dot_idx = self
            .builder
            .build_load(i64_type, out_idx_ptr, "f64_dot_idx")
            .unwrap()
            .into_int_value();
        let dot_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(i8_type, buffer, &[dot_idx], "f64_dot_ptr")
                .unwrap()
        };
        self.builder
            .build_store(dot_ptr, i8_type.const_int(b'.' as u64, false));
        let idx_after_dot = self
            .builder
            .build_int_add(dot_idx, i64_type.const_int(1, false), "f64_idx_after_dot")
            .unwrap();
        self.builder.build_store(out_idx_ptr, idx_after_dot);

        let frac_ptr = self.builder.build_alloca(i64_type, "f64_frac_ptr").unwrap();
        self.builder.build_store(frac_ptr, frac_int);
        let divisor_ptr = self.builder.build_alloca(i64_type, "f64_divisor_ptr").unwrap();
        self.builder
            .build_store(divisor_ptr, i64_type.const_int(100000, false));
        let count_ptr = self.builder.build_alloca(i64_type, "f64_count_ptr").unwrap();
        self.builder.build_store(count_ptr, i64_type.const_zero());

        let frac_body = self.context.append_basic_block(function, "frac_body");
        let frac_next = self.context.append_basic_block(function, "frac_next");
        self.builder.build_unconditional_branch(frac_body).unwrap();

        self.builder.position_at_end(frac_body);
        let count = self
            .builder
            .build_load(i64_type, count_ptr, "f64_count")
            .unwrap()
            .into_int_value();
        let more = self
            .builder
            .build_int_compare(IntPredicate::ULT, count, i64_type.const_int(6, false), "f64_more")
            .unwrap();
        self.builder
            .build_conditional_branch(more, frac_next, done)
            .unwrap();

        self.builder.position_at_end(frac_next);
        let divisor = self
            .builder
            .build_load(i64_type, divisor_ptr, "f64_divisor")
            .unwrap()
            .into_int_value();
        let frac_value = self
            .builder
            .build_load(i64_type, frac_ptr, "f64_frac_val")
            .unwrap()
            .into_int_value();
        let digit = self
            .builder
            .build_int_unsigned_div(frac_value, divisor, "f64_digit")
            .unwrap();
        let rest = self
            .builder
            .build_int_unsigned_rem(frac_value, divisor, "f64_rest")
            .unwrap();
        self.builder.build_store(frac_ptr, rest);
        let out_idx2 = self
            .builder
            .build_load(i64_type, out_idx_ptr, "f64_out_idx2")
            .unwrap()
            .into_int_value();
        let out_ptr2 = unsafe {
            self.builder
                .build_in_bounds_gep(i8_type, buffer, &[out_idx2], "f64_digit_ptr")
                .unwrap()
        };
        let digit_ascii = self
            .builder
            .build_int_add(digit, i64_type.const_int(b'0' as u64, false), "f64_digit_ascii")
            .unwrap();
        let digit8 = self
            .builder
            .build_int_cast(digit_ascii, i8_type, "f64_digit8")
            .unwrap();
        self.builder.build_store(out_ptr2, digit8);
        self.builder.build_store(
            out_idx_ptr,
            self.builder
                .build_int_add(out_idx2, i64_type.const_int(1, false), "f64_out_idx2_next")
                .unwrap(),
        );
        self.builder.build_store(
            divisor_ptr,
            self.builder
                .build_int_unsigned_div(divisor, i64_type.const_int(10, false), "f64_divisor_next")
                .unwrap(),
        );
        self.builder.build_store(
            count_ptr,
            self.builder
                .build_int_add(count, i64_type.const_int(1, false), "f64_count_next")
                .unwrap(),
        );
        self.builder.build_unconditional_branch(frac_body).unwrap();

        self.builder.position_at_end(done);
        let end_idx = self
            .builder
            .build_load(i64_type, out_idx_ptr, "f64_end_idx")
            .unwrap()
            .into_int_value();
        let end_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(i8_type, buffer, &[end_idx], "f64_end_ptr")
                .unwrap()
        };
        self.builder.build_store(end_ptr, i8_type.const_zero());
        self.builder.build_return(Some(&buffer));

        if let Some(block) = caller_block {
            self.builder.position_at_end(block);
        }

        function
    }

    fn emit_value_io(
        &mut self,
        symbol: &str,
        key: Str<'backend>,
        arguments: &[Box<Analysis<'backend>>],
        function: FunctionValue<'backend>,
        newline: bool,
    ) -> BasicValueEnum<'backend> {
        let value = arguments
            .first()
            .map(|argument| self.instruction(argument.instruction.clone(), function));

        let Some(value) = value else {
            return self.emit_string_io(symbol, key, arguments, function);
        };

        if value.is_pointer_value() {
            return self.emit_pointer_io(symbol, key, value.into_pointer_value());
        }

        if value.is_int_value() {
            let int = value.into_int_value();
            if int.get_type().get_bit_width() == 1 {
                let bool_str = self
                    .builder
                    .build_select(
                        int,
                        self.builder
                            .build_global_string_ptr("true", "print_true")
                            .unwrap()
                            .as_pointer_value(),
                        self.builder
                            .build_global_string_ptr("false", "print_false")
                            .unwrap()
                            .as_pointer_value(),
                        "print_bool_ptr",
                    )
                    .unwrap()
                    .into_pointer_value();
                return if newline {
                    self.emit_pointer_io(symbol, key, bool_str)
                } else {
                    self.emit_pointer_io(
                        if symbol == "axo_println" {
                            "axo_print_raw"
                        } else if symbol == "axo_eprintln" {
                            "axo_eprint_raw"
                        } else {
                            symbol
                        },
                        key,
                        bool_str,
                    )
                };
            }

            let widened = if int.get_type().get_bit_width() == 64 {
                int
            } else {
                self.builder
                    .build_int_cast(int, self.context.i64_type(), "print_int64")
                    .unwrap()
            };

            let to_string = self.runtime_i64_to_string_function();
            let pointer = self
                .builder
                .build_call(to_string, &[BasicMetadataValueEnum::from(widened)], "i64_to_str")
                .unwrap()
                .try_as_basic_value().basic()
                .map(|value| value.into_pointer_value())
                .unwrap_or_else(|| {
                    self.builder
                        .build_global_string_ptr("0", "print_int_default")
                        .unwrap()
                        .as_pointer_value()
                });
            return self.emit_pointer_io(symbol, key, pointer);
        }

        if value.is_float_value() {
            let float = value.into_float_value();
            let widened = if float.get_type() == self.context.f64_type() {
                float
            } else {
                self.builder
                    .build_float_cast(float, self.context.f64_type(), "print_float64")
                    .unwrap()
            };
            let to_string = self.runtime_f64_to_string_function();
            let pointer = self
                .builder
                .build_call(
                    to_string,
                    &[BasicMetadataValueEnum::from(widened)],
                    "f64_to_str",
                )
                .unwrap()
                .try_as_basic_value().basic()
                .map(|value| value.into_pointer_value())
                .unwrap_or_else(|| {
                    self.builder
                        .build_global_string_ptr("0.0", "print_float_default")
                        .unwrap()
                        .as_pointer_value()
                });
            return self.emit_pointer_io(symbol, key, pointer);
        }

        let fallback = self
            .builder
            .build_global_string_ptr("<value>", "print_value_fallback")
            .unwrap()
            .as_pointer_value();
        self.emit_pointer_io(symbol, key, fallback)
    }

    fn emit_read_line(&mut self) -> BasicValueEnum<'backend> {
        let callee = self.runtime_read_line_function();
        let result = self.builder.build_call(callee, &[], "read_line").unwrap();

        result.try_as_basic_value().basic().unwrap_or_else(|| {
            self.context
                .ptr_type(inkwell::AddressSpace::default())
                .const_null()
                .into()
        })
    }

    fn to_i64(
        &mut self,
        value: BasicValueEnum<'backend>,
        name: &str,
    ) -> inkwell::values::IntValue<'backend> {
        if value.is_int_value() {
            let int = value.into_int_value();
            if int.get_type().get_bit_width() == 64 {
                int
            } else {
                self.builder
                    .build_int_cast(int, self.context.i64_type(), name)
                    .unwrap()
            }
        } else if value.is_pointer_value() {
            self.builder
                .build_ptr_to_int(value.into_pointer_value(), self.context.i64_type(), name)
                .unwrap()
        } else {
            self.context.i64_type().const_zero()
        }
    }

    fn emit_len(
        &mut self,
        arguments: &[Box<Analysis<'backend>>],
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let value = arguments
            .first()
            .map(|argument| self.instruction(argument.instruction.clone(), function));
        let pointer = match value {
            Some(value) if value.is_pointer_value() => value.into_pointer_value(),
            _ => self
                .builder
                .build_global_string_ptr("", "len_empty")
                .unwrap()
                .as_pointer_value(),
        };
        let callee = self.runtime_strlen_function();
        self.builder
            .build_call(callee, &[BasicMetadataValueEnum::from(pointer)], "len_call")
            .unwrap()
            .try_as_basic_value().basic()
            .unwrap_or(self.context.i64_type().const_zero().into())
    }

    fn emit_write(
        &mut self,
        arguments: &[Box<Analysis<'backend>>],
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        if arguments.len() < 2 {
            return self.context.i64_type().const_zero().into();
        }

        let fd_value = self.instruction(arguments[0].instruction.clone(), function);
        let fd = self.to_i64(fd_value, "write_fd");

        let text_value = self.instruction(arguments[1].instruction.clone(), function);
        let pointer = match text_value {
            value if value.is_pointer_value() => value.into_pointer_value(),
            _ => self
                .builder
                .build_global_string_ptr("", "write_empty")
                .unwrap()
                .as_pointer_value(),
        };
        let ptr_i64 = self
            .builder
            .build_ptr_to_int(pointer, self.context.i64_type(), "write_ptr")
            .unwrap();
        let strlen = self.runtime_strlen_function();
        let length = self
            .builder
            .build_call(
                strlen,
                &[BasicMetadataValueEnum::from(pointer)],
                "write_len",
            )
            .unwrap()
            .try_as_basic_value().basic()
            .map(|value| value.into_int_value())
            .unwrap_or_else(|| self.context.i64_type().const_zero());
        let written = self.linux_syscall3(
            self.context
                .i64_type()
                .const_int(self.syscall_write_number(), false),
            fd,
            ptr_i64,
            length,
            "write_call",
        );
        written.into()
    }

    fn emit_alloc(
        &mut self,
        arguments: &[Box<Analysis<'backend>>],
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let size_value = arguments
            .first()
            .map(|argument| self.instruction(argument.instruction.clone(), function))
            .unwrap_or_else(|| self.context.i64_type().const_zero().into());
        let size = self.to_i64(size_value, "alloc_size");

        let prot = self.context.i64_type().const_int(0x1 | 0x2, false);
        let flags = self.context.i64_type().const_int(0x2 | 0x20, false);
        let fd = self.context.i64_type().const_all_ones();
        let offset = self.context.i64_type().const_zero();
        let syscall_no = {
            #[cfg(target_arch = "aarch64")]
            {
                self.context.i64_type().const_int(222, false)
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                self.context.i64_type().const_int(9, false)
            }
        };
        let fn_ty = self.context.i64_type().fn_type(
            &[
                self.context.i64_type().into(),
                self.context.i64_type().into(),
                self.context.i64_type().into(),
                self.context.i64_type().into(),
                self.context.i64_type().into(),
                self.context.i64_type().into(),
                self.context.i64_type().into(),
            ],
            false,
        );
        #[cfg(target_arch = "aarch64")]
        let (assembly, constraints) = (
            "svc #0".to_string(),
            "={x0},{x8},{x0},{x1},{x2},{x3},{x4},{x5},~{memory}".to_string(),
        );
        #[cfg(target_arch = "x86_64")]
        let (assembly, constraints) = (
            "syscall".to_string(),
            "=r,{rax},{rdi},{rsi},{rdx},{r10},{r8},{r9},~{rcx},~{r11},~{memory}".to_string(),
        );
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        let (assembly, constraints) = (
            "syscall".to_string(),
            "=r,{rax},{rdi},{rsi},{rdx},{r10},{r8},{r9},~{rcx},~{r11},~{memory}".to_string(),
        );
        let syscall = self.context.create_inline_asm(
            fn_ty,
            assembly,
            constraints,
            true,
            false,
            Some(InlineAsmDialect::ATT),
            false,
        );
        let result = self
            .builder
            .build_indirect_call(
                fn_ty,
                syscall,
                &[
                    syscall_no.into(),
                    self.context.i64_type().const_zero().into(),
                    size.into(),
                    prot.into(),
                    flags.into(),
                    fd.into(),
                    offset.into(),
                ],
                "alloc_call",
            )
            .unwrap()
            .try_as_basic_value().basic()
            .map(|value| value.into_int_value())
            .unwrap_or_else(|| self.context.i64_type().const_zero());
        result.into()
    }

    fn emit_free(
        &mut self,
        arguments: &[Box<Analysis<'backend>>],
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        if arguments.len() < 2 {
            return self.context.i64_type().const_zero().into();
        }
        let ptr_value = self.instruction(arguments[0].instruction.clone(), function);
        let ptr = self.to_i64(ptr_value, "free_ptr");
        let size_value = self.instruction(arguments[1].instruction.clone(), function);
        let size = self.to_i64(size_value, "free_size");
        let fn_ty = self.context.i64_type().fn_type(
            &[
                self.context.i64_type().into(),
                self.context.i64_type().into(),
                self.context.i64_type().into(),
            ],
            false,
        );
        #[cfg(target_arch = "aarch64")]
        let (assembly, constraints, syscall_no) = (
            "svc #0".to_string(),
            "={x0},{x8},{x0},{x1},~{memory}".to_string(),
            self.context.i64_type().const_int(215, false),
        );
        #[cfg(not(target_arch = "aarch64"))]
        let (assembly, constraints, syscall_no) = (
            "syscall".to_string(),
            "=r,{rax},{rdi},{rsi},~{rcx},~{r11},~{memory}".to_string(),
            self.context.i64_type().const_int(11, false),
        );
        let syscall = self.context.create_inline_asm(
            fn_ty,
            assembly,
            constraints,
            true,
            false,
            Some(InlineAsmDialect::ATT),
            false,
        );
        let _ = self
            .builder
            .build_indirect_call(
                fn_ty,
                syscall,
                &[syscall_no.into(), ptr.into(), size.into()],
                "free_call",
            )
            .unwrap();
        self.context.i64_type().const_zero().into()
    }

    fn invoke_target_name(instruction: &Instruction<'backend>) -> Option<Str<'backend>> {
        match instruction {
            Instruction::Usage(name) => Some(*name),
            Instruction::Access(_, member) => Self::invoke_target_name(&member.instruction),
            _ => None,
        }
    }

    fn primitive_cast(
        &mut self,
        name: &str,
        arguments: &[Box<Analysis<'backend>>],
        function: FunctionValue<'backend>,
    ) -> Option<BasicValueEnum<'backend>> {
        let arg = arguments
            .first()
            .map(|value| self.instruction(value.instruction.clone(), function));

        match name {
            "Int64" => Some(match arg {
                Some(value) if value.is_int_value() => self
                    .builder
                    .build_int_cast(value.into_int_value(), self.context.i64_type(), "cast_int")
                    .unwrap()
                    .into(),
                Some(value) if value.is_float_value() => self
                    .builder
                    .build_float_to_signed_int(
                        value.into_float_value(),
                        self.context.i64_type(),
                        "cast_float_to_int",
                    )
                    .unwrap()
                    .into(),
                _ => self.context.i64_type().const_zero().into(),
            }),
            "Int32" => Some(match arg {
                Some(value) if value.is_int_value() => self
                    .builder
                    .build_int_cast(value.into_int_value(), self.context.i32_type(), "cast_i32")
                    .unwrap()
                    .into(),
                Some(value) if value.is_float_value() => self
                    .builder
                    .build_float_to_signed_int(
                        value.into_float_value(),
                        self.context.i32_type(),
                        "cast_float_to_i32",
                    )
                    .unwrap()
                    .into(),
                _ => self.context.i32_type().const_zero().into(),
            }),
            "Float" => Some(match arg {
                Some(value) if value.is_float_value() => self
                    .builder
                    .build_float_cast(
                        value.into_float_value(),
                        self.context.f64_type(),
                        "cast_float",
                    )
                    .unwrap()
                    .into(),
                Some(value) if value.is_int_value() => self
                    .builder
                    .build_signed_int_to_float(
                        value.into_int_value(),
                        self.context.f64_type(),
                        "cast_int_to_float",
                    )
                    .unwrap()
                    .into(),
                _ => self.context.f64_type().const_zero().into(),
            }),
            "Boolean" => Some(match arg {
                Some(value) if value.is_int_value() => {
                    let int = value.into_int_value();
                    self.builder
                        .build_int_compare(
                            IntPredicate::NE,
                            int,
                            int.get_type().const_zero(),
                            "cast_bool_int",
                        )
                        .unwrap()
                        .into()
                }
                Some(value) if value.is_float_value() => {
                    let float = value.into_float_value();
                    self.builder
                        .build_float_compare(
                            FloatPredicate::ONE,
                            float,
                            float.get_type().const_zero(),
                            "cast_bool_float",
                        )
                        .unwrap()
                        .into()
                }
                _ => self.context.bool_type().const_zero().into(),
            }),
            "Character" | "Char" => Some(match arg {
                Some(value) if value.is_int_value() => self
                    .builder
                    .build_int_cast(value.into_int_value(), self.context.i32_type(), "cast_char")
                    .unwrap()
                    .into(),
                Some(value) if value.is_float_value() => self
                    .builder
                    .build_float_to_signed_int(
                        value.into_float_value(),
                        self.context.i32_type(),
                        "cast_float_to_char",
                    )
                    .unwrap()
                    .into(),
                _ => self.context.i32_type().const_zero().into(),
            }),
            _ => None,
        }
    }

    pub fn module(
        &mut self,
        name: Str<'backend>,
        analyses: Vec<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        self.modules.insert(name);
        let caller_block = self.builder.get_insert_block();
        for analysis in analyses {
            if self.has_terminator() {
                break;
            }
            let current_block = self.builder.get_insert_block();
            self.instruction(analysis.instruction, function);
            if let Some(block) = current_block {
                self.builder.position_at_end(block);
            }
        }
        if let Some(block) = caller_block {
            self.builder.position_at_end(block);
        }
        BasicValueEnum::from(self.context.i64_type().const_zero())
    }

    pub fn method(
        &mut self,
        method: Method<
            Str<'backend>,
            Box<Analysis<'backend>>,
            Box<Analysis<'backend>>,
            Option<Box<Analysis<'backend>>>,
        >,
    ) -> BasicValueEnum<'backend> {
        let mut parameters = vec![];
        for member in &method.members {
            if let Instruction::Binding(bind) = &member.instruction {
                let kind = bind
                    .annotation
                    .as_ref()
                    .map(|annotation| self.llvm_type_from_type_kind(annotation))
                    .unwrap_or_else(|| self.context.i64_type().into());
                parameters.push(kind);
            }
        }
        let parameter_types: Vec<inkwell::types::BasicMetadataTypeEnum<'backend>> =
            parameters.iter().map(|kind| (*kind).into()).collect();

        let return_type: Option<inkwell::types::BasicTypeEnum<'backend>> = method.output.as_ref().map(
            |output| match &output.instruction {
                Instruction::Usage(name) => {
                    if let Some(kind) = name.as_str().and_then(TypeKind::from_name) {
                        if matches!(kind, TypeKind::Tuple { ref members } if members.len() == 0) {
                            return None;
                        } else {
                            Some(self.llvm_type_from_type_kind(&kind))
                        }
                    } else {
                        self.annotation_type(output)
                            .or_else(|| Some(self.context.i64_type().into()))
                    }
                }
                _ => self
                    .annotation_type(output)
                    .or_else(|| Some(self.context.i64_type().into())),
            },
        ).flatten();

        let function_type = match return_type {
            Some(kind) => kind.fn_type(&parameter_types, false),
            None => self.context.void_type().fn_type(&parameter_types, false),
        };

        let name = method.target.as_str().unwrap();
        let function = self.module.add_function(name, function_type, None);

        let previous_entities = self.entities.clone();
        let mut scoped_entities = Map::default();
        for (name, entity) in previous_entities.iter() {
            if let Entity::Function(function) = entity {
                scoped_entities.insert((*name).clone(), Entity::Function(function.clone()));
            }
        }
        self.entities = scoped_entities;
        self.entities
            .insert(method.target.clone(), Entity::Function(function));

        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);

        for (param_val, member) in function.get_param_iter().zip(method.members.iter()) {
            if let Instruction::Binding(bind) = &member.instruction {
                let name = bind.target.as_str().unwrap();
                let allocate = self.build_entry_alloca(function, param_val.get_type(), name);
                self.builder.build_store(allocate, param_val);
                let signed = if param_val.get_type().is_int_type() {
                    Some(true)
                } else {
                    None
                };
                self.entities.insert(
                    bind.target.clone(),
                    Entity::Variable {
                        pointer: allocate,
                        kind: param_val.get_type(),
                        pointee: None,
                        signed,
                    },
                );
            }
        }

        self.loop_headers.clear();
        self.loop_exits.clear();
        let body_result = self.instruction(method.body.instruction.clone(), function);

        if self
            .builder
            .get_insert_block()
            .and_then(|block| block.get_terminator())
            .is_none()
        {
            if return_type.is_none() {
                self.builder.build_return(None);
            } else {
                let value = self.coerce(function, body_result);
                self.builder.build_return(Some(&value));
            }
        }

        self.entities = previous_entities;
        self.entities
            .insert(method.target.clone(), Entity::Function(function));

        self.context.i64_type().const_zero().into()
    }

    pub fn block(
        &mut self,
        analyses: Vec<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let mut last = self.context.i64_type().const_zero().into();
        for analysis in analyses {
            if self.has_terminator() {
                break;
            }
            last = self.instruction(analysis.instruction, function);
        }
        last
    }

    pub fn conditional(
        &mut self,
        condition: Box<Analysis<'backend>>,
        then: Box<Analysis<'backend>>,
        otherwise: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        if self.has_terminator() {
            return self.context.i64_type().const_zero().into();
        }

        let condition = self.instruction(condition.instruction, function);
        let condition = self.truthy(condition);

        let then_block = self.context.append_basic_block(function, "if_then");
        let else_block = self.context.append_basic_block(function, "if_else");
        let merge_block = self.context.append_basic_block(function, "if_merge");

        self.builder
            .build_conditional_branch(condition, then_block, else_block)
            .unwrap();

        self.builder.position_at_end(then_block);
        let then_value = self.instruction(then.instruction, function);
        let then_end = self.builder.get_insert_block();
        let then_reaches_merge = !self.has_terminator();
        if then_reaches_merge {
            self.builder
                .build_unconditional_branch(merge_block)
                .unwrap();
        }

        self.builder.position_at_end(else_block);
        let else_value = self.instruction(otherwise.instruction, function);
        let else_end = self.builder.get_insert_block();
        let else_reaches_merge = !self.has_terminator();
        if else_reaches_merge {
            self.builder
                .build_unconditional_branch(merge_block)
                .unwrap();
        }

        self.builder.position_at_end(merge_block);

        if then_reaches_merge && else_reaches_merge && then_value.get_type() == else_value.get_type()
        {
            let phi = self
                .builder
                .build_phi(then_value.get_type(), "if_result")
                .unwrap();
            phi.add_incoming(&[(&then_value, then_end.unwrap()), (&else_value, else_end.unwrap())]);
            phi.as_basic_value()
        } else if then_reaches_merge {
            then_value
        } else if else_reaches_merge {
            else_value
        } else {
            self.context.i64_type().const_zero().into()
        }
    }

    pub fn r#while(
        &mut self,
        condition: Box<Analysis<'backend>>,
        body: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        if self.has_terminator() {
            return self.context.i64_type().const_zero().into();
        }

        let condition_block = self.context.append_basic_block(function, "while_condition");
        let body_block = self.context.append_basic_block(function, "while_body");
        let end_block = self.context.append_basic_block(function, "while_end");

        self.builder
            .build_unconditional_branch(condition_block)
            .unwrap();

        self.builder.position_at_end(condition_block);
        let condition = self.instruction(condition.instruction, function);
        let condition = self.truthy(condition);
        self.builder
            .build_conditional_branch(condition, body_block, end_block)
            .unwrap();

        self.builder.position_at_end(body_block);
        self.loop_headers.push(condition_block);
        self.loop_exits.push(end_block);
        self.instruction(body.instruction, function);
        self.loop_exits.pop();
        self.loop_headers.pop();

        if !self.has_terminator() {
            self.builder
                .build_unconditional_branch(condition_block)
                .unwrap();
        }

        self.builder.position_at_end(end_block);
        self.context.i64_type().const_zero().into()
    }

    pub fn cycle(
        &mut self,
        condition: Box<Analysis<'backend>>,
        body: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        self.r#while(condition, body, function)
    }

    pub fn invoke(
        &mut self,
        invoke: Invoke<Box<Analysis<'backend>>, Box<Analysis<'backend>>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let name = Self::invoke_target_name(&invoke.target.instruction)
            .and_then(|value| value.as_str())
            .unwrap_or("");

        if name == "print" {
            return self.emit_value_io(
                "axo_println",
                Str::from("axo_println"),
                &invoke.members,
                function,
                true,
            );
        }

        if name == "print_raw" {
            return self.emit_value_io(
                "axo_print_raw",
                Str::from("axo_print_raw"),
                &invoke.members,
                function,
                false,
            );
        }

        if name == "eprint" {
            return self.emit_value_io(
                "axo_eprintln",
                Str::from("axo_eprintln"),
                &invoke.members,
                function,
                true,
            );
        }

        if name == "eprint_raw" {
            return self.emit_value_io(
                "axo_eprint_raw",
                Str::from("axo_eprint_raw"),
                &invoke.members,
                function,
                false,
            );
        }

        if name == "read_line" {
            return self.emit_read_line();
        }

        if name == "len" {
            return self.emit_len(&invoke.members, function);
        }

        if name == "write" {
            return self.emit_write(&invoke.members, function);
        }

        if name == "alloc" {
            return self.emit_alloc(&invoke.members, function);
        }

        if name == "free" {
            return self.emit_free(&invoke.members, function);
        }

        if let Some(value) = self.primitive_cast(name, &invoke.members, function) {
            return value;
        }

        if let Instruction::Usage(target_name) = &invoke.target.instruction {
            let option = self.entities.get(target_name).and_then(|entity| {
                if let Entity::Function(func) = entity {
                    Some(*func)
                } else {
                    None
                }
            });

            if let Some(value) = option {
                let mut arguments = vec![];
                for argument in &invoke.members {
                    let value = self.instruction(argument.instruction.clone(), function);
                    arguments.push(value.into());
                }
                let result = self.builder.build_call(value, &arguments, "call").unwrap();
                return result
                    .try_as_basic_value().basic()
                    .unwrap_or(self.context.i64_type().const_zero().into());
            }
        }

        self.context.i64_type().const_zero().into()
    }

    pub fn r#return(
        &mut self,
        value: Option<Box<Analysis<'backend>>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        if self.has_terminator() {
            return self.context.i64_type().const_zero().into();
        }

        match value {
            Some(item) => {
                let result = self.instruction(item.instruction, function);
                if function.get_type().get_return_type().is_none() {
                    self.builder.build_return(None);
                    self.context.i64_type().const_zero().into()
                } else {
                    let value = self.coerce(function, result);
                    self.builder.build_return(Some(&value));
                    value
                }
            }
            None => {
                self.builder.build_return(None);
                self.context.i64_type().const_zero().into()
            }
        }
    }

    pub fn r#break(
        &mut self,
        value: Option<Box<Analysis<'backend>>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        if let Some(item) = value {
            self.instruction(item.instruction, function);
        }

        if self.has_terminator() {
            return self.context.i64_type().const_zero().into();
        }

        if let Some(exit) = self.loop_exits.last().copied() {
            self.builder.build_unconditional_branch(exit).unwrap();
        }

        self.context.i64_type().const_zero().into()
    }

    pub fn r#continue(
        &mut self,
        value: Option<Box<Analysis<'backend>>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        if let Some(item) = value {
            self.instruction(item.instruction, function);
        }

        if self.has_terminator() {
            return self.context.i64_type().const_zero().into();
        }

        if let Some(header) = self.loop_headers.last().copied() {
            self.builder.build_unconditional_branch(header).unwrap();
        }

        self.context.i64_type().const_zero().into()
    }
}
