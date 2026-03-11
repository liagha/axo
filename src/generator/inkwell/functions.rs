use inkwell::types::BasicTypeEnum;
use inkwell::values::IntValue;
use {
    super::{Backend, Entity},
    crate::{
        data::*,
        analyzer::Analysis,
        checker::TypeKind,
        internal::hash::Map,
        generator::{ErrorKind, GenerateError},
        tracker::Span,
    },
    inkwell::{
        types::BasicType,
        values::{BasicValueEnum, FunctionValue},
        FloatPredicate, IntPredicate,
    },
};
use crate::analyzer::AnalysisKind;
use crate::checker::Type;
use crate::generator::inkwell::error::{ControlFlowError, FunctionError};

impl<'backend> super::Inkwell<'backend> {
    fn terminated(&self) -> bool {
        self.builder
            .get_insert_block()
            .and_then(|block| block.get_terminator())
            .is_some()
    }

    fn coerce(
        &mut self,
        function: FunctionValue<'backend>,
        value: BasicValueEnum<'backend>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let target = match function.get_type().get_return_type() {
            Some(kind) => kind,
            None => return Ok(value),
        };

        if value.get_type() == target {
            return Ok(value);
        }

        match (value, target) {
            (BasicValueEnum::IntValue(integer), target) if target.is_int_type() => self
                .builder
                .build_int_cast(integer, target.into_int_type(), "cast")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),
            (BasicValueEnum::FloatValue(float), target) if target.is_float_type() => self
                .builder
                .build_float_cast(float, target.into_float_type(), "cast")
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
        if value.is_int_value() {
            let integer = value.into_int_value();
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
        } else if value.is_float_value() {
            let float = value.into_float_value();
            self.builder
                .build_float_compare(
                    FloatPredicate::ONE,
                    float,
                    float.get_type().const_zero(),
                    "condition",
                )
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
        } else {
            Ok(self.context.bool_type().const_zero())
        }
    }

    fn cast(
        &mut self,
        name: &str,
        arguments: &[Analysis<'backend>],
        span: Span<'backend>,
    ) -> Result<Option<BasicValueEnum<'backend>>, GenerateError<'backend>> {
        if !matches!(name, "Int64" | "Int32" | "Float" | "Boolean" | "Character" | "Char") {
            return Ok(None);
        }

        let argument = if let Some(passed) = arguments.first() {
            Some(self.analysis(passed.clone())?)
        } else {
            None
        };

        match name {
            "Int64" => Ok(Some(match argument {
                Some(value) if value.is_int_value() => self
                    .builder
                    .build_int_cast(value.into_int_value(), self.context.i64_type(), "cast")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                    .into(),
                Some(value) if value.is_float_value() => self
                    .builder
                    .build_float_to_signed_int(
                        value.into_float_value(),
                        self.context.i64_type(),
                        "cast",
                    )
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                    .into(),
                _ => self.context.i64_type().const_zero().into(),
            })),
            "Int32" => Ok(Some(match argument {
                Some(value) if value.is_int_value() => self
                    .builder
                    .build_int_cast(value.into_int_value(), self.context.i32_type(), "cast")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                    .into(),
                Some(value) if value.is_float_value() => self
                    .builder
                    .build_float_to_signed_int(
                        value.into_float_value(),
                        self.context.i32_type(),
                        "cast",
                    )
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                    .into(),
                _ => self.context.i32_type().const_zero().into(),
            })),
            "Float" => Ok(Some(match argument {
                Some(value) if value.is_float_value() => self
                    .builder
                    .build_float_cast(
                        value.into_float_value(),
                        self.context.f64_type(),
                        "cast",
                    )
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                    .into(),
                Some(value) if value.is_int_value() => self
                    .builder
                    .build_signed_int_to_float(
                        value.into_int_value(),
                        self.context.f64_type(),
                        "cast",
                    )
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                    .into(),
                _ => self.context.f64_type().const_zero().into(),
            })),
            "Boolean" => Ok(Some(match argument {
                Some(value) if value.is_int_value() => {
                    let integer = value.into_int_value();
                    self.builder
                        .build_int_compare(
                            IntPredicate::NE,
                            integer,
                            integer.get_type().const_zero(),
                            "cast",
                        )
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                        .into()
                }
                Some(value) if value.is_float_value() => {
                    let float = value.into_float_value();
                    self.builder
                        .build_float_compare(
                            FloatPredicate::ONE,
                            float,
                            float.get_type().const_zero(),
                            "cast",
                        )
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                        .into()
                }
                _ => self.context.bool_type().const_zero().into(),
            })),
            "Character" | "Char" => Ok(Some(match argument {
                Some(value) if value.is_int_value() => self
                    .builder
                    .build_int_cast(value.into_int_value(), self.context.i32_type(), "cast")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                    .into(),
                Some(value) if value.is_float_value() => self
                    .builder
                    .build_float_to_signed_int(
                        value.into_float_value(),
                        self.context.i32_type(),
                        "cast",
                    )
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                    .into(),
                _ => self.context.i32_type().const_zero().into(),
            })),
            _ => Ok(None),
        }
    }

    pub fn module(
        &mut self,
        name: Str<'backend>,
        analyses: Vec<Analysis<'backend>>,
        _span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let string = name.as_str().unwrap_or("module");
        self.modules.insert(name, self.context.create_module(string));

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

        Ok(BasicValueEnum::from(self.context.i64_type().const_zero()))
    }

    pub fn function(
        &mut self,
        method: Function<
            Str<'backend>,
            Analysis<'backend>,
            Box<Analysis<'backend>>,
            Option<Type<'backend>>,
        >,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let mut parameters = vec![];

        for member in &method.members {
            if let AnalysisKind::Binding(bind) = &member.kind {
                let kind = if let Some(annotation) = bind.annotation.as_ref() {
                    let resolved = self.llvm_type(annotation, member.span)?;

                    if matches!(method.interface, Interface::C) {
                        if let TypeKind::String = annotation.kind {
                            self.context.ptr_type(inkwell::AddressSpace::default()).into()
                        } else if let TypeKind::Character = annotation.kind {
                            self.context.i8_type().into()
                        } else {
                            resolved
                        }
                    } else {
                        resolved
                    }
                } else {
                    self.context.i64_type().into()
                };

                parameters.push(kind.into());
            }
        }

        let output = if let Some(annotation) = method.output {
            Some(self.llvm_type(&annotation, span)?)
        } else {
            None
        };

        let signature = match output {
            Some(kind) => kind.fn_type(&parameters, false),
            None => self.context.void_type().fn_type(&parameters, false),
        };

        let identifier = method.target.as_str().unwrap_or("function");

        let function = if matches!(method.interface, Interface::C) {
            let callable = self.current_module().add_function(
                identifier,
                signature,
                Some(inkwell::module::Linkage::External),
            );
            callable.set_section(Some("text"));
            self.insert_entity(method.target.clone(), Entity::Function(callable));
            callable
        } else {
            let linkage = if method.entry {
                Some(inkwell::module::Linkage::External)
            } else {
                Some(inkwell::module::Linkage::Internal)
            };

            let callable = self.current_module().add_function(identifier, signature, linkage);

            self.insert_entity(method.target.clone(), Entity::Function(callable));

            self.entities.push(Map::default());

            let entry = self.context.append_basic_block(callable, "entry");
            self.builder.position_at_end(entry);
            callable
        };

        if !matches!(method.interface, Interface::C) {
            for (parameter, member) in function.get_param_iter().zip(method.members.iter()) {
                if let AnalysisKind::Binding(bind) = &member.kind {
                    let allocate = self.build_entry(function, parameter.get_type(), bind.target.clone());

                    self.builder.build_store(allocate, parameter)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                    let signed = if parameter.get_type().is_int_type() {
                        Some(true)
                    } else {
                        None
                    };
                    self.insert_entity(
                        bind.target.clone(),
                        Entity::Variable {
                            pointer: allocate,
                            kind: parameter.get_type(),
                            pointee: None,
                            signed,
                        },
                    );
                }
            }

            self.loop_headers.clear();
            self.loop_exits.clear();
            self.loop_results.clear();

            let result = self.analysis(*method.body.clone())?;

            if !self.terminated() {
                if output.is_none() {
                    self.builder.build_return(None)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                } else {
                    let value = self.coerce(function, result, span)?;
                    self.builder.build_return(Some(&value))
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                }
            }

            self.entities.pop();
        }

        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn block(
        &mut self,
        analyses: Vec<Analysis<'backend>>,
        _span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let mut last = self.context.i64_type().const_zero().into();
        for analysis in analyses {
            if self.terminated() {
                break;
            }
            last = self.analysis(analysis)?;
        }
        Ok(last)
    }

    pub fn conditional(
        &mut self,
        condition: Box<Analysis<'backend>>,
        positive: Box<Analysis<'backend>>,
        negative: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if self.terminated() {
            return Ok(self.context.i64_type().const_zero().into());
        }

        let evaluated = self.analysis(*condition)?;
        let truth = self.truth(evaluated, span)?;

        let function = self.parent(span)?;
        let consequence = self.context.append_basic_block(function, "consequence");
        let alternative = self.context.append_basic_block(function, "alternative");
        let merge = self.context.append_basic_block(function, "merge");

        self.builder
            .build_conditional_branch(truth, consequence, alternative)
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder.position_at_end(consequence);
        let leftwards = self.analysis(*positive)?;
        let persists = !self.terminated();

        if persists {
            self.builder
                .build_unconditional_branch(merge)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
        }

        self.builder.position_at_end(alternative);
        let rightwards = self.analysis(*negative)?;
        let continues = !self.terminated();

        if continues {
            self.builder
                .build_unconditional_branch(merge)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
        }

        self.builder.position_at_end(merge);

        if persists && continues && leftwards.get_type() == rightwards.get_type() {
            let result_alloca = self.build_entry(function, leftwards.get_type(), "cond_res".into());

            if let Some(left_block) = consequence.get_terminator().and_then(|t| t.get_parent()) {
                self.builder.position_before(&left_block.get_terminator().unwrap());
                self.builder.build_store(result_alloca, leftwards)
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            }

            if let Some(right_block) = alternative.get_terminator().and_then(|t| t.get_parent()) {
                self.builder.position_before(&right_block.get_terminator().unwrap());
                self.builder.build_store(result_alloca, rightwards)
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            }

            self.builder.position_at_end(merge);
            let value = self.builder.build_load(leftwards.get_type(), result_alloca, "cond_val")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            Ok(value)
        } else if persists {
            Ok(leftwards)
        } else if continues {
            Ok(rightwards)
        } else {
            Ok(self.context.i64_type().const_zero().into())
        }
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

        let function = self.parent(span)?;
        let heading = self.context.append_basic_block(function, "heading");
        let core = self.context.append_basic_block(function, "core");
        let end = self.context.append_basic_block(function, "end");

        let result_alloc = self.build_entry(function, self.context.i64_type().into(), "loop_res".into());
        self.builder.build_store(result_alloc, self.context.i64_type().const_zero())
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder
            .build_unconditional_branch(heading)
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder.position_at_end(heading);
        let evaluated = self.analysis(*condition)?;
        let truth = self.truth(evaluated, span)?;

        self.builder
            .build_conditional_branch(truth, core, end)
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder.position_at_end(core);

        self.loop_headers.push(heading);
        self.loop_exits.push(end);
        self.loop_results.push(Some(result_alloc));

        self.analysis(*body)?;

        self.loop_results.pop();
        self.loop_exits.pop();
        self.loop_headers.pop();

        if !self.terminated() {
            self.builder
                .build_unconditional_branch(heading)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
        }

        self.builder.position_at_end(end);

        let final_value = self.builder.build_load(self.context.i64_type(), result_alloc, "loop_val")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        Ok(final_value)
    }

    pub fn invoke(
        &mut self,
        invoke: Invoke<Str<'backend>, Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if let Some(value) = self.cast(&*invoke.target, &invoke.members, span)? {
            return Ok(value);
        }

        let entity = self.get_entity(&invoke.target).and_then(|item| {
            if let Entity::Function(callable) = item {
                Some(*callable)
            } else {
                None
            }
        });

        if let Some(callable) = entity {
            let mut arguments = vec![];

            let expected = callable.get_type().get_param_types();

            for (position, argument) in invoke.members.iter().enumerate() {
                let mut evaluated = self.analysis(argument.clone())?;

                if let Some(kind) = expected.get(position) {
                    if evaluated.is_pointer_value() && kind.is_int_type() {
                        evaluated = self.builder
                            .build_ptr_to_int(
                                evaluated.into_pointer_value(),
                                kind.into_int_type(),
                                "cast"
                            )
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                            .into();
                    } else if evaluated.is_int_value() && kind.is_pointer_type() {
                        evaluated = self.builder
                            .build_int_to_ptr(
                                evaluated.into_int_value(),
                                kind.into_pointer_type(),
                                "cast"
                            )
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                            .into();
                    }
                }

                arguments.push(evaluated.into());
            }

            let result = self.builder.build_call(callable, &arguments, "call")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            return if let Some(value) = result.try_as_basic_value().basic() {
                Ok(value)
            } else {
                Ok(self.context.i64_type().const_zero().into())
            }
        }

        Err(GenerateError::new(
            ErrorKind::Function(FunctionError::Undefined {
                name: invoke.target.to_string(),
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
                let result = self.analysis(*item)?;
                if function.get_type().get_return_type().is_none() {
                    self.builder.build_return(None)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                    Ok(self.context.i64_type().const_zero().into())
                } else {
                    let coerced = self.coerce(function, result, span)?;
                    self.builder.build_return(Some(&coerced))
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                    Ok(coerced)
                }
            }
            None => {
                self.builder.build_return(None)
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
            if let Some(Some(alloc)) = self.loop_results.last() {
                self.builder.build_store(*alloc, evaluated)
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            }
        }

        if self.terminated() {
            return Ok(self.context.i64_type().const_zero().into());
        }

        if let Some(exit) = self.loop_exits.last().copied() {
            self.builder.build_unconditional_branch(exit)
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
            if let Some(Some(alloc)) = self.loop_results.last() {
                self.builder.build_store(*alloc, evaluated)
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            }
        }

        if self.terminated() {
            return Ok(self.context.i64_type().const_zero().into());
        }

        if let Some(header) = self.loop_headers.last().copied() {
            self.builder.build_unconditional_branch(header)
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
            .ok_or_else(|| GenerateError::new(
                ErrorKind::Function(FunctionError::NotInFunctionContext),
                span
            ))
    }

    pub fn explicit_cast(
        &mut self,
        value: Box<Analysis<'backend>>,
        target_type: Type<'backend>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let evaluated = self.analysis(*value)?;
        let llvm_target = self.llvm_type(&target_type, span)?;

        if evaluated.get_type() == llvm_target {
            return Ok(evaluated);
        }

        match (evaluated, llvm_target) {
            (BasicValueEnum::IntValue(integer), BasicTypeEnum::IntType(target)) => self
                .builder
                .build_int_cast(integer, target, "cast")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            (BasicValueEnum::FloatValue(float), BasicTypeEnum::FloatType(target)) => self
                .builder
                .build_float_cast(float, target, "cast")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            (BasicValueEnum::IntValue(integer), BasicTypeEnum::FloatType(target)) => self
                .builder
                .build_signed_int_to_float(integer, target, "cast")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            (BasicValueEnum::FloatValue(float), BasicTypeEnum::IntType(target)) => self
                .builder
                .build_float_to_signed_int(float, target, "cast")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

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

            _ => Err(GenerateError::new(
                ErrorKind::Cast,
                span,
            )),
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
                .build_int_neg(integer, "neg")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            BasicValueEnum::FloatValue(float) => self
                .builder
                .build_float_neg(float, "fneg")
                .map(Into::into)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span)),

            _ => Err(GenerateError::new(
                ErrorKind::Negate,
                span,
            )),
        }
    }

    pub fn size_of(
        &mut self,
        ty: Type<'backend>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let llvm_target = self.llvm_type(&ty, span)?;

        let size = llvm_target.size_of().ok_or_else(|| {
            GenerateError::new(
                ErrorKind::SizeOf,
                span,
            )
        })?;

        Ok(size.into())
    }
}
