use {
    crate::{
        analyzer::{Analysis, AnalysisKind},
        data::*,
        generator::{
            inkwell::{
                error::{ControlFlowError, FunctionError},
                Entity,
            },
            Backend, ErrorKind, GenerateError,
        },
        resolver::{Type, TypeKind},
        tracker::Span,
    },
    inkwell::{
        basic_block::BasicBlock,
        types::{BasicType, BasicTypeEnum},
        values::{BasicValue, BasicValueEnum, FunctionValue, IntValue},
        FloatPredicate, IntPredicate,
    },
};

impl<'backend> super::Generator<'backend> {
    pub fn align(&self, layout: BasicTypeEnum<'backend>) -> u32 {
        if layout.is_pointer_type() || layout.is_struct_type() || layout.is_array_type() {
            return 8;
        }
        if layout.is_int_type() && layout.into_int_type().get_bit_width() >= 64 {
            return 8;
        }
        if layout.is_float_type() {
            return 8;
        }
        4
    }

    fn terminated(&self) -> bool {
        self.builder
            .get_insert_block()
            .and_then(|block| block.get_terminator())
            .is_some()
    }

    fn truth(
        &mut self,
        value: BasicValueEnum<'backend>,
        span: Span<'backend>,
    ) -> Result<IntValue<'backend>, GenerateError<'backend>> {
        match value {
            BasicValueEnum::IntValue(integer) => {
                if integer.get_type().get_bit_width() == 1 {
                    Ok(integer)
                } else {
                    self.builder
                        .build_int_compare(
                            IntPredicate::NE,
                            integer,
                            integer.get_type().const_zero(),
                            "condition",
                        )
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                }
            }
            BasicValueEnum::FloatValue(float) => self
                .builder
                .build_float_compare(
                    FloatPredicate::ONE,
                    float,
                    float.get_type().const_zero(),
                    "condition",
                )
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),
            BasicValueEnum::PointerValue(pointer) => self
                .builder
                .build_is_not_null(pointer, "condition")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),
            _ => Ok(self.context.bool_type().const_zero()),
        }
    }

    pub fn module(
        &mut self,
        target: Str<'backend>,
        analyses: Vec<Analysis<'backend>>,
        _span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let name = target.as_str().unwrap_or("module");
        self.modules.insert(target, self.context.create_module(name));

        let caller = self.builder.get_insert_block();

        for analysis in analyses {
            if self.terminated() {
                break;
            }

            let current = self.builder.get_insert_block();
            self.analysis(analysis)?;

            if let Some(block) = current {
                self.builder.position_at_end(block);
            }
        }

        if let Some(block) = caller {
            self.builder.position_at_end(block);
        }

        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn function(
        &mut self,
        routine: Function<
            Str<'backend>,
            Analysis<'backend>,
            Box<Analysis<'backend>>,
            Option<Type<'backend>>,
        >,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let mut params = vec![];

        for member in &routine.members {
            if let AnalysisKind::Binding(binding) = &member.kind {
                let layout = {
                    let layout = self.to_basic_type(&binding.annotation, member.span)?;

                    if matches!(routine.interface, Interface::C) {
                        if let TypeKind::String = &binding.annotation.kind {
                            self.context.ptr_type(inkwell::AddressSpace::default()).into()
                        } else if let TypeKind::Character = &binding.annotation.kind {
                            self.context.i8_type().into()
                        } else {
                            layout
                        }
                    } else {
                        layout
                    }
                };

                params.push(layout.into());
            }
        }

        let output = match &routine.output {
            Some(annotation) => Some(self.to_basic_type(annotation, span)?),
            None => None,
        };

        let signature = match output {
            Some(layout) => layout.fn_type(&params, false),
            None => self.context.void_type().fn_type(&params, false),
        };

        let name = routine.target.as_str().unwrap_or("function");

        let function = if matches!(routine.interface, Interface::C) {
            let external = self.current_module().add_function(
                name,
                signature,
                Some(inkwell::module::Linkage::External),
            );
            external.set_section(Some("text"));
            self.insert_entity(routine.target.clone(), Entity::Function(external));
            external
        } else {
            let linkage = if routine.entry {
                Some(inkwell::module::Linkage::External)
            } else {
                Some(inkwell::module::Linkage::External)
            };

            let internal = self.current_module().add_function(name, signature, linkage);

            self.insert_entity(routine.target.clone(), Entity::Function(internal));

            let entry = self.context.append_basic_block(internal, "entry");
            self.builder.position_at_end(entry);
            internal
        };

        if !matches!(routine.interface, Interface::C) {
            for (param, member) in function.get_param_iter().zip(routine.members.iter()) {
                if let AnalysisKind::Binding(binding) = &member.kind {
                    let pointer = self.build_entry(function, param.get_type(), binding.target.clone());
                    let align = self.align(param.get_type());

                    self.builder
                        .build_store(pointer, param)
                        .and_then(|inst| {
                            inst.set_alignment(align).ok();
                            Ok(inst)
                        })
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                    self.insert_entity(
                        binding.target.clone(),
                        Entity::Variable {
                            pointer,
                            typing: binding.annotation.clone(),
                        },
                    );
                }
            }

            self.clear_loops();
            let result = self.analysis(*routine.body.clone())?;

            if !self.terminated() {
                if output.is_none() {
                    self.builder
                        .build_return(None)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                } else {
                    let expected = function.get_type().get_return_type().unwrap();

                    if result.get_type() != expected {
                        return Err(GenerateError::new(
                            ErrorKind::Function(FunctionError::IncompatibleReturnType),
                            span,
                        ));
                    }

                    self.builder
                        .build_return(Some(&result))
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                }
            }
        }

        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn block(
        &mut self,
        analyses: Vec<Analysis<'backend>>,
        _span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let mut value = self.context.i64_type().const_zero().into();

        for analysis in analyses {
            if self.terminated() {
                break;
            }
            value = self.analysis(analysis)?;
        }

        Ok(value)
    }

    pub fn conditional(
        &mut self,
        condition: Analysis<'backend>,
        truth: Analysis<'backend>,
        fall: Option<Analysis<'backend>>,
        span: Span<'backend>,
        needed: bool,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let check = self.analysis(condition)?;
        let flag = self.truth(check, span)?;

        let current = self
            .builder
            .get_insert_block()
            .ok_or_else(|| GenerateError::new(ErrorKind::Function(FunctionError::NotInFunctionContext), span))?;

        let parent = current
            .get_parent()
            .ok_or_else(|| GenerateError::new(ErrorKind::Function(FunctionError::NotInFunctionContext), span))?;

        let pass = self.context.append_basic_block(parent, "pass");
        let fail = self.context.append_basic_block(parent, "fail");
        let merge = self.context.append_basic_block(parent, "merge");

        self.builder.build_conditional_branch(flag, pass, fail).ok();

        self.builder.position_at_end(pass);
        let left = self.analysis(truth)?;
        let left_end = self.builder.get_insert_block().unwrap_or(pass);
        let left_done = left_end.get_terminator().is_some();

        if !left_done {
            self.builder.build_unconditional_branch(merge).ok();
        }

        self.builder.position_at_end(fail);
        let right = if let Some(expression) = fall {
            self.analysis(expression)?
        } else {
            match left.get_type() {
                BasicTypeEnum::IntType(layout) => layout.const_zero().into(),
                BasicTypeEnum::FloatType(layout) => layout.const_zero().into(),
                BasicTypeEnum::PointerType(layout) => layout.const_null().into(),
                BasicTypeEnum::StructType(layout) => layout.const_zero().into(),
                BasicTypeEnum::ArrayType(layout) => layout.const_zero().into(),
                BasicTypeEnum::VectorType(layout) => layout.const_zero().into(),
                BasicTypeEnum::ScalableVectorType(layout) => layout.const_zero().into(),
            }
        };

        let right_end = self.builder.get_insert_block().unwrap_or(fail);
        let right_done = right_end.get_terminator().is_some();

        if !right_done {
            self.builder.build_unconditional_branch(merge).ok();
        }

        self.builder.position_at_end(merge);

        let mut edges: Vec<(&dyn BasicValue, BasicBlock)> = Vec::new();

        if !left_done {
            edges.push((&left, left_end));
        }

        if !right_done {
            edges.push((&right, right_end));
        }

        if edges.is_empty() {
            self.builder.build_unreachable().ok();
            return Ok(left);
        }

        if !needed {
            return Ok(left);
        }

        if edges.len() == 1 {
            return Ok(edges[0].0.as_basic_value_enum());
        }

        let first = edges[0].0.as_basic_value_enum();
        let identical = edges.iter().all(|(val, _)| val.as_basic_value_enum() == first);

        if identical {
            return Ok(first);
        }

        let layout = left.get_type();
        let phi = self
            .builder
            .build_phi(layout, "mapping")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        phi.add_incoming(&edges);

        Ok(phi.as_basic_value())
    }

    pub fn r#while(
        &mut self,
        condition: Box<Analysis<'backend>>,
        body: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if self.terminated() {
            return Ok(self.context.i64_type().const_zero().into());
        }

        let parent = self.parent(span)?;
        let start = self.context.append_basic_block(parent, "start");
        let core = self.context.append_basic_block(parent, "core");
        let exit = self.context.append_basic_block(parent, "exit");

        let pointer = self.build_entry(parent, self.context.i64_type().into(), "loop".into());
        let align = self.align(self.context.i64_type().into());

        self.builder
            .build_store(pointer, self.context.i64_type().const_zero())
            .and_then(|inst| {
                inst.set_alignment(align).ok();
                Ok(inst)
            })
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder
            .build_unconditional_branch(start)
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder.position_at_end(start);
        let check = self.analysis(*condition)?;
        let flag = self.truth(check, span)?;

        self.builder
            .build_conditional_branch(flag, core, exit)
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder.position_at_end(core);
        self.enter_loop(start, exit, Some(pointer));
        self.analysis(*body)?;
        self.exit_loop();

        if !self.terminated() {
            self.builder
                .build_unconditional_branch(start)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
        }

        self.builder.position_at_end(exit);
        let completed = self.builder
            .build_load(self.context.i64_type(), pointer, "load")
            .and_then(|value| {
                if let Some(inst) = value.as_instruction_value() {
                    inst.set_alignment(align).ok();
                }
                Ok(value)
            })
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        Ok(completed)
    }

    pub fn invoke(
        &mut self,
        call: Invoke<Str<'backend>, Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let name = call.target.as_str().unwrap_or_default();

        let entity = self.get_entity(&call.target).and_then(|item| {
            if let Entity::Function(function) = item {
                let module = self.current_module();
                let identifier = function.get_name().to_str().unwrap_or(name);

                if let Some(existing) = module.get_function(identifier) {
                    Some(existing)
                } else {
                    let layout = function.get_type();
                    let external = module.add_function(
                        identifier,
                        layout,
                        Some(inkwell::module::Linkage::External),
                    );
                    Some(external)
                }
            } else {
                None
            }
        });

        if let Some(function) = entity {
            let mut arguments = vec![];
            let params = function.get_type().get_param_types();

            for (index, argument) in call.members.iter().enumerate() {
                let mut value = self.analysis(argument.clone())?;

                if let Some(layout) = params.get(index) {
                    if let Ok(expected) = BasicTypeEnum::try_from(*layout) {
                        if value.get_type() != expected && value.is_pointer_value() {
                            let align = self.align(expected);
                            value = self.builder
                                .build_load(expected, value.into_pointer_value(), "load")
                                .and_then(|inst| {
                                    if let Some(instruction) = inst.as_instruction_value() {
                                        instruction.set_alignment(align).ok();
                                    }
                                    Ok(inst)
                                })
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                        }
                    }
                }

                arguments.push(value.into());
            }

            let result = self.builder
                .build_call(function, &arguments, "call")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            return if let Some(bound) = result.try_as_basic_value().basic() {
                Ok(bound)
            } else {
                Ok(self.context.i64_type().const_zero().into())
            };
        }

        Err(GenerateError::new(
            ErrorKind::Function(FunctionError::Undefined {
                name: call.target.to_string(),
            }),
            span,
        ))
    }

    pub fn r#return(
        &mut self,
        value: Option<Box<Analysis<'backend>>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if self.terminated() {
            return Ok(self.context.i64_type().const_zero().into());
        }

        let function = self.parent(span)?;

        match value {
            Some(item) => {
                let check = self.analysis(*item)?;
                if let Some(layout) = function.get_type().get_return_type() {
                    if check.get_type() != layout {
                        return Err(GenerateError::new(
                            ErrorKind::Function(FunctionError::IncompatibleReturnType),
                            span,
                        ));
                    }
                    self.builder
                        .build_return(Some(&check))
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                    Ok(check)
                } else {
                    self.builder
                        .build_return(None)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                    Ok(self.context.i64_type().const_zero().into())
                }
            }
            None => {
                self.builder
                    .build_return(None)
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                Ok(self.context.i64_type().const_zero().into())
            }
        }
    }

    pub fn r#break(
        &mut self,
        value: Option<Box<Analysis<'backend>>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if let Some(item) = value {
            let check = self.analysis(*item)?;
            if let Some(pointer) = self.current_loop_result() {
                let align = self.align(check.get_type());
                self.builder
                    .build_store(pointer, check)
                    .and_then(|inst| {
                        inst.set_alignment(align).ok();
                        Ok(inst)
                    })
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            }
        }

        if self.terminated() {
            return Ok(self.context.i64_type().const_zero().into());
        }

        if let Some(exit) = self.current_loop_exit() {
            self.builder
                .build_unconditional_branch(exit)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
        } else {
            return Err(GenerateError::new(
                ErrorKind::ControlFlow(ControlFlowError::BreakOutsideLoop),
                span,
            ));
        }

        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn r#continue(
        &mut self,
        value: Option<Box<Analysis<'backend>>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if let Some(item) = value {
            let check = self.analysis(*item)?;
            if let Some(pointer) = self.current_loop_result() {
                let align = self.align(check.get_type());
                self.builder
                    .build_store(pointer, check)
                    .and_then(|inst| {
                        inst.set_alignment(align).ok();
                        Ok(inst)
                    })
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            }
        }

        if self.terminated() {
            return Ok(self.context.i64_type().const_zero().into());
        }

        if let Some(start) = self.current_loop_header() {
            self.builder
                .build_unconditional_branch(start)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
        } else {
            return Err(GenerateError::new(
                ErrorKind::ControlFlow(ControlFlowError::ContinueOutsideLoop),
                span,
            ));
        }

        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn parent(
        &self,
        span: Span<'backend>,
    ) -> Result<FunctionValue<'backend>, GenerateError<'backend>> {
        self.builder
            .get_insert_block()
            .and_then(|block| block.get_parent())
            .ok_or_else(|| {
                GenerateError::new(ErrorKind::Function(FunctionError::NotInFunctionContext), span)
            })
    }

    pub fn negate(
        &mut self,
        value: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let check = self.analysis(*value)?;

        match check {
            BasicValueEnum::IntValue(integer) => self
                .builder
                .build_int_neg(integer, "negate")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            BasicValueEnum::FloatValue(float) => self
                .builder
                .build_float_neg(float, "negate")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            _ => Err(GenerateError::new(ErrorKind::Negate, span)),
        }
    }

    pub fn size_of(
        &mut self,
        layout: Type<'backend>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let target = self.to_basic_type(&layout, span)?;

        let size = target
            .size_of()
            .ok_or_else(|| GenerateError::new(ErrorKind::SizeOf, span))?;

        Ok(size.into())
    }
}
