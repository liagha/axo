use {
    super::{Backend, Entity},
    crate::{
        data::*,
        analyzer::Analysis,
        checker::TypeKind,
        internal::hash::Map,
    },
    inkwell::{
        types::BasicType,
        values::{BasicValueEnum, FunctionValue},
        FloatPredicate, IntPredicate,
    },
};
use crate::analyzer::AnalysisKind;
use crate::checker::Type;

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

    fn primitive_cast(
        &mut self,
        name: &str,
        arguments: &[Analysis<'backend>],
    ) -> Option<BasicValueEnum<'backend>> {
        let arg = arguments
            .first()
            .map(|value| self.analysis(value.clone()));

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
    ) -> BasicValueEnum<'backend> {
        self.modules.insert(name, self.context.create_module(name.as_str().unwrap()));

        let caller_block = self.builder.get_insert_block();
        for analysis in analyses {
            if self.has_terminator() {
                break;
            }
            let current_block = self.builder.get_insert_block();
            self.analysis(analysis);
            if let Some(block) = current_block {
                self.builder.position_at_end(block);
            }
        }
        if let Some(block) = caller_block {
            self.builder.position_at_end(block);
        }
        BasicValueEnum::from(self.context.i64_type().const_zero())
    }

    pub fn function(
        &mut self,
        method: Function<
            Str<'backend>,
            Analysis<'backend>,
            Box<Analysis<'backend>>,
            Option<Type<'backend>>,
        >,
    ) -> BasicValueEnum<'backend> {
        let mut parameters = vec![];
        for member in &method.members {
            if let AnalysisKind::Binding(bind) = &member.kind {
                let kind = bind
                    .annotation
                    .as_ref()
                    .map(|annotation| {
                        let llvm_kind = self.llvm_type(annotation);

                        if matches!(method.interface, Interface::C) {
                            if let TypeKind::String = annotation.kind {
                                self.context.ptr_type(inkwell::AddressSpace::default()).into()
                            } else if let TypeKind::Character = annotation.kind {
                                self.context.i8_type().into()
                            } else {
                                llvm_kind
                            }
                        } else {
                            llvm_kind
                        }
                    })
                    .unwrap_or_else(|| self.context.i64_type().into());
                parameters.push(kind);
            }
        }
        let parameter_types: Vec<inkwell::types::BasicMetadataTypeEnum<'backend>> =
            parameters.iter().map(|kind| (*kind).into()).collect();

        let return_type: Option<inkwell::types::BasicTypeEnum<'backend>> = method.output.as_ref().map(
            |output| self.llvm_type(output).into(),
        ).flatten();

        let function_type = match return_type {
            Some(kind) => kind.fn_type(&parameter_types, false),
            None => self.context.void_type().fn_type(&parameter_types, false),
        };

        let name = method.target.as_str().unwrap();

        let function = if matches!(method.interface, Interface::C) {
            let function = self.current_module().add_function(
                name,
                function_type,
                Some(inkwell::module::Linkage::External),
            );
            function.set_section(Some(".text"));
            self.entities
                .insert(method.target.clone(), Entity::Function(function));
            function
        } else {
            let linkage = if method.entry {
                Some(inkwell::module::Linkage::External)
            } else {
                Some(inkwell::module::Linkage::Internal)
            };

            let function = self.current_module().add_function(name, function_type, linkage);

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
            function
        };

        if !matches!(method.interface, Interface::C) {
            for (param_val, member) in function.get_param_iter().zip(method.members.iter()) {
                if let AnalysisKind::Binding(bind) = &member.kind {
                    let allocate = self.build_entry(function, param_val.get_type(), bind.target);
                    let _ = self.builder.build_store(allocate, param_val);
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
            let body_result = self.analysis(*method.body.clone());

            if self
                .builder
                .get_insert_block()
                .and_then(|block| block.get_terminator())
                .is_none()
            {
                if return_type.is_none() {
                    let _ = self.builder.build_return(None);
                } else {
                    let value = self.coerce(function, body_result);
                    let _ = self.builder.build_return(Some(&value));
                }
            }
        }

        self.context.i64_type().const_zero().into()
    }

    pub fn block(
        &mut self,
        analyses: Vec<Analysis<'backend>>,
    ) -> BasicValueEnum<'backend> {
        let mut last = self.context.i64_type().const_zero().into();
        for analysis in analyses {
            if self.has_terminator() {
                break;
            }
            last = self.analysis(analysis);
        }
        last
    }

    pub fn conditional(
        &mut self,
        condition: Box<Analysis<'backend>>,
        then: Box<Analysis<'backend>>,
        otherwise: Box<Analysis<'backend>>,
    ) -> BasicValueEnum<'backend> {
        if self.has_terminator() {
            return self.context.i64_type().const_zero().into();
        }

        let condition = self.analysis(*condition);
        let condition = self.truthy(condition);

        let then_block = self.context.append_basic_block(self.current_function(), "if_then");
        let else_block = self.context.append_basic_block(self.current_function(), "if_else");
        let merge_block = self.context.append_basic_block(self.current_function(), "if_merge");

        self.builder
            .build_conditional_branch(condition, then_block, else_block)
            .unwrap();

        self.builder.position_at_end(then_block);
        let then_value = self.analysis(*then);
        let then_end = self.builder.get_insert_block();
        let then_reaches_merge = !self.has_terminator();
        if then_reaches_merge {
            self.builder
                .build_unconditional_branch(merge_block)
                .unwrap();
        }

        self.builder.position_at_end(else_block);
        let else_value = self.analysis(*otherwise);
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
    ) -> BasicValueEnum<'backend> {
        if self.has_terminator() {
            return self.context.i64_type().const_zero().into();
        }

        let condition_block = self.context.append_basic_block(self.current_function(), "while_condition");
        let body_block = self.context.append_basic_block(self.current_function(), "while_body");
        let end_block = self.context.append_basic_block(self.current_function(), "while_end");

        self.builder
            .build_unconditional_branch(condition_block)
            .unwrap();

        self.builder.position_at_end(condition_block);
        let condition = self.analysis(*condition);
        let condition = self.truthy(condition);
        self.builder
            .build_conditional_branch(condition, body_block, end_block)
            .unwrap();

        self.builder.position_at_end(body_block);
        self.loop_headers.push(condition_block);
        self.loop_exits.push(end_block);
        self.analysis(*body);
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

    pub fn invoke(
        &mut self,
        invoke: Invoke<Str<'backend>, Analysis<'backend>>,
    ) -> BasicValueEnum<'backend> {
        if let Some(value) = self.primitive_cast(&*invoke.target, &invoke.members) {
            return value;
        }

        let option = self.entities.get(&invoke.target).and_then(|entity| {
            if let Entity::Function(func) = entity {
                Some(*func)
            } else {
                None
            }
        });

        if let Some(value) = option {
            let mut arguments = vec![];
            for argument in &invoke.members {
                let value = self.analysis(argument.clone());
                arguments.push(value.into());
            }
            let result = self.builder.build_call(value, &arguments, "call").unwrap();
            return result
                .try_as_basic_value().basic()
                .unwrap_or(self.context.i64_type().const_zero().into());
        }

        self.context.i64_type().const_zero().into()
    }

    pub fn r#return(
        &mut self,
        value: Option<Box<Analysis<'backend>>>,
    ) -> BasicValueEnum<'backend> {
        if self.has_terminator() {
            return self.context.i64_type().const_zero().into();
        }

        match value {
            Some(item) => {
                let result = self.analysis(*item);
                if self.current_function().get_type().get_return_type().is_none() {
                    let _ = self.builder.build_return(None);
                    self.context.i64_type().const_zero().into()
                } else {
                    let value = self.coerce(self.current_function(), result);
                    let _ = self.builder.build_return(Some(&value));
                    value
                }
            }
            None => {
                let _ = self.builder.build_return(None);
                self.context.i64_type().const_zero().into()
            }
        }
    }

    pub fn r#break(
        &mut self,
        value: Option<Box<Analysis<'backend>>>,
    ) -> BasicValueEnum<'backend> {
        if let Some(item) = value {
            self.analysis(*item);
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
    ) -> BasicValueEnum<'backend> {
        if let Some(item) = value {
            self.analysis(*item);
        }

        if self.has_terminator() {
            return self.context.i64_type().const_zero().into();
        }

        if let Some(header) = self.loop_headers.last().copied() {
            self.builder.build_unconditional_branch(header).unwrap();
        }

        self.context.i64_type().const_zero().into()
    }

    pub fn current_function(&self) -> FunctionValue<'backend> {
        self.builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap()
    }
}
