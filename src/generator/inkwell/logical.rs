use {
    super::{
        Backend,
        Inkwell,
    },
    crate::{
        analyzer::{
            Analysis,
        },
        generator::{
            ErrorKind,
            GenerateError,
        },
        tracker::{
            Span,
        },
    },
    inkwell::{
        values::{
            BasicValueEnum,
            IntValue,
        },
    },
};
use crate::generator::BuilderError;

impl<'backend> Inkwell<'backend> {
    fn check_boolean(
        &self,
        value: BasicValueEnum<'backend>,
        span: Span<'backend>,
    ) -> Result<IntValue<'backend>, GenerateError<'backend>> {
        if !value.is_int_value() || value.into_int_value().get_type().get_bit_width() != 1 {
            return Err(GenerateError::new(ErrorKind::Boolean, span));
        }
        
        Ok(value.into_int_value())
    }

    pub fn logical_and(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let primary = self.check_boolean(alpha, span)?;

        let block = self.builder.get_insert_block().ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::BlockInsertion), span))?;
        let parent = block.get_parent().ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::Parent), span))?;

        let evaluate = self.context.append_basic_block(parent, "evaluate");
        let merge = self.context.append_basic_block(parent, "merge");

        self.builder.build_conditional_branch(primary, evaluate, merge)
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder.position_at_end(evaluate);
        let beta = self.analysis(*right)?;
        let secondary = self.check_boolean(beta, span)?;

        let current = self.builder.get_insert_block().unwrap();
        self.builder.build_unconditional_branch(merge)
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder.position_at_end(merge);
        let phi = self.builder.build_phi(self.context.bool_type(), "result")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        let zero = self.context.bool_type().const_zero();
        phi.add_incoming(&[
            (&zero, block),
            (&secondary, current),
        ]);

        Ok(phi.as_basic_value())
    }

    pub fn logical_or(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let primary = self.check_boolean(alpha, span)?;

        let block = self.builder.get_insert_block().ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::BlockInsertion), span))?;
        let parent = block.get_parent().ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::Parent), span))?;

        let evaluate = self.context.append_basic_block(parent, "evaluate");
        let merge = self.context.append_basic_block(parent, "merge");

        self.builder.build_conditional_branch(primary, merge, evaluate)
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder.position_at_end(evaluate);
        let beta = self.analysis(*right)?;
        let secondary = self.check_boolean(beta, span)?;

        let current = self.builder.get_insert_block().unwrap();
        self.builder.build_unconditional_branch(merge)
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder.position_at_end(merge);
        let phi = self.builder.build_phi(self.context.bool_type(), "result")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        let one = self.context.bool_type().const_int(1, false);
        phi.add_incoming(&[
            (&one, block),
            (&secondary, current),
        ]);

        Ok(phi.as_basic_value())
    }

    pub fn logical_not(
        &mut self,
        operand: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let alpha = self.analysis(*operand)?;

        let primary = self.check_boolean(alpha, span)?;

        Ok(BasicValueEnum::from(
            self.builder.build_not(primary, "not")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
        ))
    }

    pub fn logical_xor(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let primary = self.check_boolean(alpha, span)?;
        let secondary = self.check_boolean(beta, span)?;

        Ok(BasicValueEnum::from(
            self.builder.build_xor(primary, secondary, "xor")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
        ))
    }
}