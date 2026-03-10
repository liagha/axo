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
use crate::generator::error::{ControlFlowError, FunctionError};

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
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let expected = match function.get_type().get_return_type() {
            Some(kind) => kind,
            None => return Ok(value),
        };

        if value.get_type() == expected {
            return Ok(value);
        }

        match (value, expected) {
            (BasicValueEnum::IntValue(int), expected) if expected.is_int_type() => self
                .builder
                .build_int_cast(int, expected.into_int_type(), "ret_cast_int")
                .map(Into::into)
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span)),
            (BasicValueEnum::FloatValue(float), expected) if expected.is_float_type() => self
                .builder
                .build_float_cast(float, expected.into_float_type(), "ret_cast_float")
                .map(Into::into)
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span)),
            (BasicValueEnum::IntValue(int), expected) if expected.is_float_type() => self
                .builder
                .build_signed_int_to_float(int, expected.into_float_type(), "ret_int_to_float")
                .map(Into::into)
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span)),
            (BasicValueEnum::FloatValue(float), expected) if expected.is_int_type() => self
                .builder
                .build_float_to_signed_int(float, expected.into_int_type(), "ret_float_to_int")
                .map(Into::into)
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span)),
            _ => Err(GenerateError::new(
                ErrorKind::Function(FunctionError::IncompatibleReturnType),
                span,
            )),
        }
    }

    fn truthy(
        &mut self,
        value: BasicValueEnum<'backend>,
        span: Span<'backend>,
    ) -> Result<inkwell::values::IntValue<'backend>, GenerateError<'backend>> {
        if value.is_int_value() {
            let int = value.into_int_value();
            if int.get_type().get_bit_width() == 1 {
                Ok(int)
            } else {
                self.builder
                    .build_int_compare(
                        IntPredicate::NE,
                        int,
                        int.get_type().const_zero(),
                        "if_cond",
                    )
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))
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
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))
        } else {
            Ok(self.context.bool_type().const_zero())
        }
    }

    fn primitive_cast(
        &mut self,
        name: &str,
        arguments: &[Analysis<'backend>],
        span: Span<'backend>,
    ) -> Result<Option<BasicValueEnum<'backend>>, GenerateError<'backend>> {
        let arg = if let Some(argument) = arguments.first() {
            Some(self.analysis(argument.clone())?)
        } else {
            None
        };

        match name {
            "Int64" => Ok(Some(match arg {
                Some(value) if value.is_int_value() => self
                    .builder
                    .build_int_cast(value.into_int_value(), self.context.i64_type(), "cast_int")
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                    .into(),
                Some(value) if value.is_float_value() => self
                    .builder
                    .build_float_to_signed_int(
                        value.into_float_value(),
                        self.context.i64_type(),
                        "cast_float_to_int",
                    )
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                    .into(),
                _ => self.context.i64_type().const_zero().into(),
            })),
            "Int32" => Ok(Some(match arg {
                Some(value) if value.is_int_value() => self
                    .builder
                    .build_int_cast(value.into_int_value(), self.context.i32_type(), "cast_i32")
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                    .into(),
                Some(value) if value.is_float_value() => self
                    .builder
                    .build_float_to_signed_int(
                        value.into_float_value(),
                        self.context.i32_type(),
                        "cast_float_to_i32",
                    )
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                    .into(),
                _ => self.context.i32_type().const_zero().into(),
            })),
            "Float" => Ok(Some(match arg {
                Some(value) if value.is_float_value() => self
                    .builder
                    .build_float_cast(
                        value.into_float_value(),
                        self.context.f64_type(),
                        "cast_float",
                    )
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                    .into(),
                Some(value) if value.is_int_value() => self
                    .builder
                    .build_signed_int_to_float(
                        value.into_int_value(),
                        self.context.f64_type(),
                        "cast_int_to_float",
                    )
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                    .into(),
                _ => self.context.f64_type().const_zero().into(),
            })),
            "Boolean" => Ok(Some(match arg {
                Some(value) if value.is_int_value() => {
                    let int = value.into_int_value();
                    self.builder
                        .build_int_compare(
                            IntPredicate::NE,
                            int,
                            int.get_type().const_zero(),
                            "cast_bool_int",
                        )
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
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
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                        .into()
                }
                _ => self.context.bool_type().const_zero().into(),
            })),
            "Character" | "Char" => Ok(Some(match arg {
                Some(value) if value.is_int_value() => self
                    .builder
                    .build_int_cast(value.into_int_value(), self.context.i32_type(), "cast_char")
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                    .into(),
                Some(value) if value.is_float_value() => self
                    .builder
                    .build_float_to_signed_int(
                        value.into_float_value(),
                        self.context.i32_type(),
                        "cast_float_to_char",
                    )
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
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
        let name_str = name.as_str().unwrap_or("module");
        self.modules.insert(name, self.context.create_module(name_str));

        let caller_block = self.builder.get_insert_block();
        for analysis in analyses {
            if self.has_terminator() {
                break;
            }
            let current_block = self.builder.get_insert_block();
            self.analysis(analysis)?;
            if let Some(block) = current_block {
                self.builder.position_at_end(block);
            }
        }
        if let Some(block) = caller_block {
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
                    let llvm_kind = self.llvm_type(annotation)?;

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
                } else {
                    self.context.i64_type().into()
                };

                parameters.push(kind);
            }
        }

        let parameter_types: Vec<inkwell::types::BasicMetadataTypeEnum<'backend>> =
            parameters.iter().map(|kind| (*kind).into()).collect();

        let return_type = if let Some(return_type) = method.output {
            Some(self.llvm_type(&return_type)?)
        } else {
            None
        };

        let function_type = match return_type {
            Some(kind) => kind.fn_type(&parameter_types, false),
            None => self.context.void_type().fn_type(&parameter_types, false),
        };

        let name = method.target.as_str().unwrap_or("anonymous_function");

        let function = if matches!(method.interface, Interface::C) {
            let function = self.current_module().add_function(
                name,
                function_type,
                Some(inkwell::module::Linkage::External),
            );
            function.set_section(Some(".text"));
            self.entities.insert(method.target.clone(), Entity::Function(function));
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
            self.entities.insert(method.target.clone(), Entity::Function(function));

            let entry_block = self.context.append_basic_block(function, "entry");
            self.builder.position_at_end(entry_block);
            function
        };

        if !matches!(method.interface, Interface::C) {
            for (param_val, member) in function.get_param_iter().zip(method.members.iter()) {
                if let AnalysisKind::Binding(bind) = &member.kind {
                    let allocate = self.build_entry(function, param_val.get_type(), bind.target.clone());

                    self.builder.build_store(allocate, param_val)
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

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
            let body_result = self.analysis(*method.body.clone())?;

            if !self.has_terminator() {
                if return_type.is_none() {
                    self.builder.build_return(None)
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
                } else {
                    let value = self.coerce(function, body_result, span)?;
                    self.builder.build_return(Some(&value))
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
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
        let mut last = self.context.i64_type().const_zero().into();
        for analysis in analyses {
            if self.has_terminator() {
                break;
            }
            last = self.analysis(analysis)?;
        }
        Ok(last)
    }

    pub fn conditional(
        &mut self,
        condition: Box<Analysis<'backend>>,
        then: Box<Analysis<'backend>>,
        otherwise: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if self.has_terminator() {
            return Ok(self.context.i64_type().const_zero().into());
        }

        let condition_val = self.analysis(*condition)?;
        let condition_truthy = self.truthy(condition_val, span)?;

        let current_func = self.current_function(span)?;
        let then_block = self.context.append_basic_block(current_func, "if_then");
        let else_block = self.context.append_basic_block(current_func, "if_else");
        let merge_block = self.context.append_basic_block(current_func, "if_merge");

        self.builder
            .build_conditional_branch(condition_truthy, then_block, else_block)
            .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

        self.builder.position_at_end(then_block);
        let then_value = self.analysis(*then)?;
        let then_end = self.builder.get_insert_block();
        let then_reaches_merge = !self.has_terminator();

        if then_reaches_merge {
            self.builder
                .build_unconditional_branch(merge_block)
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
        }

        self.builder.position_at_end(else_block);
        let else_value = self.analysis(*otherwise)?;
        let else_end = self.builder.get_insert_block();
        let else_reaches_merge = !self.has_terminator();

        if else_reaches_merge {
            self.builder
                .build_unconditional_branch(merge_block)
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
        }

        self.builder.position_at_end(merge_block);

        if then_reaches_merge && else_reaches_merge && then_value.get_type() == else_value.get_type() {
            let phi = self
                .builder
                .build_phi(then_value.get_type(), "if_result")
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

            if let (Some(t_end), Some(e_end)) = (then_end, else_end) {
                phi.add_incoming(&[(&then_value, t_end), (&else_value, e_end)]);
            }
            Ok(phi.as_basic_value())
        } else if then_reaches_merge {
            Ok(then_value)
        } else if else_reaches_merge {
            Ok(else_value)
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
        if self.has_terminator() {
            return Ok(self.context.i64_type().const_zero().into());
        }

        let current_func = self.current_function(span)?;
        let condition_block = self.context.append_basic_block(current_func, "while_condition");
        let body_block = self.context.append_basic_block(current_func, "while_body");
        let end_block = self.context.append_basic_block(current_func, "while_end");

        self.builder
            .build_unconditional_branch(condition_block)
            .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

        self.builder.position_at_end(condition_block);
        let condition_val = self.analysis(*condition)?;
        let condition_truthy = self.truthy(condition_val, span)?;

        self.builder
            .build_conditional_branch(condition_truthy, body_block, end_block)
            .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

        self.builder.position_at_end(body_block);
        self.loop_headers.push(condition_block);
        self.loop_exits.push(end_block);

        self.analysis(*body)?;

        self.loop_exits.pop();
        self.loop_headers.pop();

        if !self.has_terminator() {
            self.builder
                .build_unconditional_branch(condition_block)
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
        }

        self.builder.position_at_end(end_block);
        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn invoke(
        &mut self,
        invoke: Invoke<Str<'backend>, Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if let Some(value) = self.primitive_cast(&*invoke.target, &invoke.members, span)? {
            return Ok(value);
        }

        let function_entity = self.entities.get(&invoke.target).and_then(|entity| {
            if let Entity::Function(func) = entity {
                Some(*func)
            } else {
                None
            }
        });

        if let Some(func_value) = function_entity {
            let mut arguments = vec![];
            for argument in &invoke.members {
                let value = self.analysis(argument.clone())?;
                arguments.push(value.into());
            }

            let result = self.builder.build_call(func_value, &arguments, "call")
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

            return Ok(result.try_as_basic_value().basic().unwrap());
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
        if self.has_terminator() {
            return Ok(self.context.i64_type().const_zero().into());
        }

        let current_func = self.current_function(span)?;

        match value {
            Some(item) => {
                let result = self.analysis(*item)?;
                if current_func.get_type().get_return_type().is_none() {
                    self.builder.build_return(None)
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
                    Ok(self.context.i64_type().const_zero().into())
                } else {
                    let value = self.coerce(current_func, result, span)?;
                    self.builder.build_return(Some(&value))
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
                    Ok(value)
                }
            }
            None => {
                self.builder.build_return(None)
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
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
            self.analysis(*item)?;
        }

        if self.has_terminator() {
            return Ok(self.context.i64_type().const_zero().into());
        }

        if let Some(exit) = self.loop_exits.last().copied() {
            self.builder.build_unconditional_branch(exit)
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
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
            self.analysis(*item)?;
        }

        if self.has_terminator() {
            return Ok(self.context.i64_type().const_zero().into());
        }

        if let Some(header) = self.loop_headers.last().copied() {
            self.builder.build_unconditional_branch(header)
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
        } else {
            return Err(GenerateError::new(
                ErrorKind::ControlFlow(ControlFlowError::ContinueOutsideLoop),
                span,
            ));
        }

        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn current_function(
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
}
