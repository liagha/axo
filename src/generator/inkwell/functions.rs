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
    fn terminated(&self) -> bool {
        self.builder
            .get_insert_block()
            .and_then(|block| block.get_terminator())
            .is_some()
    }

    fn coerce(
        &mut self,
        callable: FunctionValue<'backend>,
        value: BasicValueEnum<'backend>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let expected = match callable.get_type().get_return_type() {
            Some(kind) => kind,
            None => return Ok(value),
        };

        if value.get_type() == expected {
            return Ok(value);
        }

        match (value, expected) {
            (BasicValueEnum::IntValue(integer), BasicTypeEnum::IntType(target)) => {
                let source = integer.get_type().get_bit_width();
                let destination = target.get_bit_width();

                if source > destination {
                    self.builder
                        .build_int_truncate(integer, target, "truncate")
                        .map(Into::into)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                } else {
                    self.builder
                        .build_int_s_extend(integer, target, "sign_extend")
                        .map(Into::into)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                }
            }
            (BasicValueEnum::FloatValue(float), BasicTypeEnum::FloatType(target)) => self
                .builder
                .build_float_cast(float, target, "cast")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            (BasicValueEnum::IntValue(integer), BasicTypeEnum::PointerType(target)) => self
                .builder
                .build_int_to_ptr(integer, target, "cast")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            (BasicValueEnum::PointerValue(pointer), BasicTypeEnum::IntType(target)) => self
                .builder
                .build_ptr_to_int(pointer, target, "cast")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            (BasicValueEnum::PointerValue(pointer), BasicTypeEnum::PointerType(target)) => self
                .builder
                .build_pointer_cast(pointer, target, "cast")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            _ => Err(GenerateError::new(
                ErrorKind::Function(FunctionError::IncompatibleReturnType),
                span,
            )),
        }
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
        name: Str<'backend>,
        analyses: Vec<Analysis<'backend>>,
        _span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let identifier = name.as_str().unwrap_or("module");
        self.modules.insert(name, self.context.create_module(identifier));

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
        let mut parameters = vec![];

        for member in &routine.members {
            if let AnalysisKind::Binding(binding) = &member.kind {
                let kind = {
                    let resolved = self.to_basic_type(&binding.annotation, member.span)?;

                    if matches!(routine.interface, Interface::C) {
                        if let TypeKind::String = &binding.annotation.kind {
                            self.context.ptr_type(inkwell::AddressSpace::default()).into()
                        } else if let TypeKind::Character = &binding.annotation.kind {
                            self.context.i8_type().into()
                        } else {
                            resolved
                        }
                    } else {
                        resolved
                    }
                };

                parameters.push(kind.into());
            }
        }

        let output = match &routine.output {
            Some(annotation) => Some(self.to_basic_type(annotation, span)?),
            None => None,
        };

        let signature = match output {
            Some(kind) => kind.fn_type(&parameters, false),
            None => self.context.void_type().fn_type(&parameters, false),
        };

        let identifier = routine.target.as_str().unwrap_or("function");

        let callable = if matches!(routine.interface, Interface::C) {
            let external = self.current_module().add_function(
                identifier,
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

            let internal = self.current_module().add_function(identifier, signature, linkage);

            self.insert_entity(routine.target.clone(), Entity::Function(internal));
            self.enter_scope();

            let entry = self.context.append_basic_block(internal, "entry");
            self.builder.position_at_end(entry);
            internal
        };

        if !matches!(routine.interface, Interface::C) {
            for (parameter, member) in callable.get_param_iter().zip(routine.members.iter()) {
                if let AnalysisKind::Binding(binding) = &member.kind {
                    let allocation = self.build_entry(callable, parameter.get_type(), binding.target.clone());

                    self.builder
                        .build_store(allocation, parameter)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                    self.insert_entity(
                        binding.target.clone(),
                        Entity::Variable {
                            pointer: allocation,
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
                    let value = self.coerce(callable, result, span)?;
                    self.builder
                        .build_return(Some(&value))
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                }
            }

            self.exit_scope();
        }

        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn block(
        &mut self,
        analyses: Vec<Analysis<'backend>>,
        _span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let mut evaluate = self.context.i64_type().const_zero().into();

        for analysis in analyses {
            if self.terminated() {
                break;
            }
            evaluate = self.analysis(analysis)?;
        }

        Ok(evaluate)
    }

    pub fn conditional(
        &mut self,
        condition: Analysis<'backend>,
        truth: Analysis<'backend>,
        fall: Option<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let evaluated = self.analysis(condition)?;
        let boolean = self.truth(evaluated, span)?;

        let current_block = self
            .builder
            .get_insert_block()
            .ok_or_else(|| GenerateError::new(ErrorKind::Function(FunctionError::NotInFunctionContext), span))?;

        let parent = current_block
            .get_parent()
            .ok_or_else(|| GenerateError::new(ErrorKind::Function(FunctionError::NotInFunctionContext), span))?;

        let truth_block = self.context.append_basic_block(parent, "truth");
        let fall_block = self.context.append_basic_block(parent, "fall");
        let merge_block = self.context.append_basic_block(parent, "merge");

        self.builder.build_conditional_branch(boolean, truth_block, fall_block).ok();

        self.builder.position_at_end(truth_block);
        let truth_result = self.analysis(truth)?;
        let truth_end = self.builder.get_insert_block().unwrap_or(truth_block);

        let truth_ended = truth_end.get_terminator().is_some();
        if !truth_ended {
            self.builder.build_unconditional_branch(merge_block).ok();
        }

        self.builder.position_at_end(fall_block);
        let fall_result = if let Some(expression) = fall {
            self.analysis(expression)?
        } else {
            match truth_result.get_type() {
                BasicTypeEnum::IntType(layout) => layout.const_zero().into(),
                BasicTypeEnum::FloatType(layout) => layout.const_zero().into(),
                BasicTypeEnum::PointerType(layout) => layout.const_null().into(),
                BasicTypeEnum::StructType(layout) => layout.const_zero().into(),
                BasicTypeEnum::ArrayType(layout) => layout.const_zero().into(),
                BasicTypeEnum::VectorType(layout) => layout.const_zero().into(),
                BasicTypeEnum::ScalableVectorType(layout) => layout.const_zero().into(),
            }
        };

        let fall_end = self.builder.get_insert_block().unwrap_or(fall_block);

        let fall_ended = fall_end.get_terminator().is_some();
        if !fall_ended {
            self.builder.build_unconditional_branch(merge_block).ok();
        }

        self.builder.position_at_end(merge_block);

        let layout = truth_result.get_type();
        let mapping = self
            .builder
            .build_phi(layout, "mapping")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        let mut incoming: Vec<(&dyn BasicValue, BasicBlock)> = Vec::new();

        if !truth_ended {
            incoming.push((&truth_result, truth_end));
        }

        if !fall_ended {
            incoming.push((&fall_result, fall_end));
        }

        if !incoming.is_empty() {
            mapping.add_incoming(&incoming);
        }

        Ok(mapping.as_basic_value())
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

        let callable = self.parent(span)?;
        let heading = self.context.append_basic_block(callable, "heading");
        let core = self.context.append_basic_block(callable, "core");
        let end = self.context.append_basic_block(callable, "end");

        let allocation = self.build_entry(callable, self.context.i64_type().into(), "loop".into());

        self.builder
            .build_store(allocation, self.context.i64_type().const_zero())
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder
            .build_unconditional_branch(heading)
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder.position_at_end(heading);
        let evaluated = self.analysis(*condition)?;
        let boolean = self.truth(evaluated, span)?;

        self.builder
            .build_conditional_branch(boolean, core, end)
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder.position_at_end(core);
        self.enter_loop(heading, end, Some(allocation));
        self.analysis(*body)?;
        self.exit_loop();

        if !self.terminated() {
            self.builder
                .build_unconditional_branch(heading)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
        }

        self.builder.position_at_end(end);
        let completed = self.builder
            .build_load(self.context.i64_type(), allocation, "load")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        Ok(completed)
    }

    pub fn invoke(
        &mut self,
        call: Invoke<Str<'backend>, Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let entity = self.get_entity(&call.target).and_then(|item| {
            if let Entity::Function(callable) = item {
                let module = self.current_module();
                let identifier = call.target.as_str().unwrap_or_default();

                if let Some(existing) = module.get_function(identifier) {
                    Some(existing)
                } else {
                    let layout = callable.get_type();
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

        if let Some(callable) = entity {
            let mut arguments = vec![];
            let expected = callable.get_type().get_param_types();

            for (index, argument) in call.members.iter().enumerate() {
                let mut argument_value = self.analysis(argument.clone())?;

                if let Some(&kind) = expected.get(index) {
                    if kind.is_pointer_type() {
                        if argument_value.is_array_value() {
                            let array = argument_value.into_array_value();
                            let parent = self.parent(span)?;
                            let allocation = self.build_entry(parent, array.get_type().into(), "decay".into());

                            self.builder
                                .build_store(allocation, array)
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                            let zero = self.context.i32_type().const_zero();
                            argument_value = unsafe {
                                self.builder
                                    .build_in_bounds_gep(
                                        array.get_type(),
                                        allocation,
                                        &[zero, zero],
                                        "pointer",
                                    )
                                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                            }
                                .into();
                        } else if argument_value.is_struct_value() {
                            let structure = argument_value.into_struct_value();
                            let parent = self.parent(span)?;
                            let allocation = self.build_entry(parent, structure.get_type().into(), "decay".into());

                            self.builder
                                .build_store(allocation, structure)
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                            argument_value = allocation.into();
                        }
                    }

                    if kind.is_struct_type() && argument_value.is_pointer_value() {
                        let layout = kind.into_struct_type();
                        argument_value = self.builder
                            .build_load(layout, argument_value.into_pointer_value(), "load")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                    }

                    if argument_value.is_pointer_value() && kind.is_int_type() {
                        argument_value = self.builder
                            .build_ptr_to_int(argument_value.into_pointer_value(), kind.into_int_type(), "cast")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                            .into();
                    } else if argument_value.is_int_value() && kind.is_pointer_type() {
                        argument_value = self.builder
                            .build_int_to_ptr(argument_value.into_int_value(), kind.into_pointer_type(), "cast")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                            .into();
                    }
                }

                arguments.push(argument_value.into());
            }

            let result = self.builder
                .build_call(callable, &arguments, "call")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            return if let Some(value) = result.try_as_basic_value().basic() {
                Ok(value)
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

        let callable = self.parent(span)?;

        match value {
            Some(item) => {
                let evaluated = self.analysis(*item)?;
                if callable.get_type().get_return_type().is_none() {
                    self.builder
                        .build_return(None)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                    Ok(self.context.i64_type().const_zero().into())
                } else {
                    let coerced = self.coerce(callable, evaluated, span)?;
                    self.builder
                        .build_return(Some(&coerced))
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                    Ok(coerced)
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
            let evaluated = self.analysis(*item)?;
            if let Some(allocation) = self.current_loop_result() {
                self.builder
                    .build_store(allocation, evaluated)
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
            let evaluated = self.analysis(*item)?;
            if let Some(allocation) = self.current_loop_result() {
                self.builder
                    .build_store(allocation, evaluated)
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            }
        }

        if self.terminated() {
            return Ok(self.context.i64_type().const_zero().into());
        }

        if let Some(heading) = self.current_loop_header() {
            self.builder
                .build_unconditional_branch(heading)
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

    pub fn explicit_cast(
        &mut self,
        value: Box<Analysis<'backend>>,
        layout: Type<'backend>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let evaluated = self.analysis(*value.clone())?;
        let expected = self.to_basic_type(&layout, span)?;

        if evaluated.get_type() == expected {
            return Ok(evaluated);
        }

        let signed = match &layout.kind {
            TypeKind::Integer { signed, .. } => *signed,
            _ => true,
        };

        match (evaluated, expected) {
            (BasicValueEnum::IntValue(integer), BasicTypeEnum::IntType(target)) => {
                let source = integer.get_type().get_bit_width();
                let destination = target.get_bit_width();

                if source > destination {
                    self.builder
                        .build_int_truncate(integer, target, "truncate")
                        .map(Into::into)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                } else if source < destination {
                    if self.infer_signedness(&value).unwrap_or(true) {
                        self.builder
                            .build_int_s_extend(integer, target, "sign_extend")
                            .map(Into::into)
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                    } else {
                        self.builder
                            .build_int_z_extend(integer, target, "zero_extend")
                            .map(Into::into)
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                    }
                } else {
                    Ok(integer.into())
                }
            }

            (BasicValueEnum::FloatValue(float), BasicTypeEnum::FloatType(target)) => self
                .builder
                .build_float_cast(float, target, "cast")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            (BasicValueEnum::IntValue(integer), BasicTypeEnum::FloatType(target)) => {
                if self.infer_signedness(&value).unwrap_or(true) {
                    self.builder
                        .build_signed_int_to_float(integer, target, "cast")
                        .map(Into::into)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                } else {
                    self.builder
                        .build_unsigned_int_to_float(integer, target, "cast")
                        .map(Into::into)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                }
            }

            (BasicValueEnum::FloatValue(float), BasicTypeEnum::IntType(target)) => {
                if signed {
                    self.builder
                        .build_float_to_signed_int(float, target, "cast")
                        .map(Into::into)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                } else {
                    self.builder
                        .build_float_to_unsigned_int(float, target, "cast")
                        .map(Into::into)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                }
            }

            (BasicValueEnum::PointerValue(pointer), BasicTypeEnum::IntType(target)) => self
                .builder
                .build_ptr_to_int(pointer, target, "cast")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            (BasicValueEnum::IntValue(integer), BasicTypeEnum::PointerType(target)) => self
                .builder
                .build_int_to_ptr(integer, target, "cast")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            (BasicValueEnum::PointerValue(pointer), BasicTypeEnum::PointerType(target)) => self
                .builder
                .build_pointer_cast(pointer, target, "cast")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            _ => Err(GenerateError::new(ErrorKind::Cast, span)),
        }
    }

    pub fn negate(
        &mut self,
        value: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let evaluated = self.analysis(*value)?;

        match evaluated {
            BasicValueEnum::IntValue(integer) => self
                .builder
                .build_int_neg(integer, "negate")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            BasicValueEnum::FloatValue(float) => self
                .builder
                .build_float_neg(float, "float_negate")
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
        let expected = self.to_basic_type(&layout, span)?;

        let size = expected
            .size_of()
            .ok_or_else(|| GenerateError::new(ErrorKind::SizeOf, span))?;

        Ok(size.into())
    }
}
