// src/generator/cranelift/comparison.rs
use {
    crate::{
        analyzer::Analysis,
        generator::{cranelift::CraneliftGenerator, GenerateError},
        tracker::Span,
    },
    cranelift_codegen::ir::{
        condcodes::{FloatCC, IntCC},
        InstBuilder, Value,
    },
};

impl<'backend> CraneliftGenerator<'backend> {
    pub fn equal(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            Ok(self.builder.ins().fcmp(FloatCC::Equal, primary, secondary))
        } else {
            Ok(self.builder.ins().icmp(IntCC::Equal, primary, secondary))
        }
    }

    pub fn not_equal(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            Ok(self.builder.ins().fcmp(FloatCC::NotEqual, primary, secondary))
        } else {
            Ok(self.builder.ins().icmp(IntCC::NotEqual, primary, secondary))
        }
    }

    pub fn less(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let signed = self.infer_signedness(&left).unwrap_or(true)
            && self.infer_signedness(&right).unwrap_or(true);

        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            Ok(self.builder.ins().fcmp(FloatCC::LessThan, primary, secondary))
        } else if signed {
            Ok(self.builder.ins().icmp(IntCC::SignedLessThan, primary, secondary))
        } else {
            Ok(self.builder.ins().icmp(IntCC::UnsignedLessThan, primary, secondary))
        }
    }

    pub fn less_or_equal(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let signed = self.infer_signedness(&left).unwrap_or(true)
            && self.infer_signedness(&right).unwrap_or(true);

        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            Ok(self.builder.ins().fcmp(FloatCC::LessThanOrEqual, primary, secondary))
        } else if signed {
            Ok(self.builder.ins().icmp(IntCC::SignedLessThanOrEqual, primary, secondary))
        } else {
            Ok(self.builder.ins().icmp(IntCC::UnsignedLessThanOrEqual, primary, secondary))
        }
    }

    pub fn greater(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let signed = self.infer_signedness(&left).unwrap_or(true)
            && self.infer_signedness(&right).unwrap_or(true);

        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            Ok(self.builder.ins().fcmp(FloatCC::GreaterThan, primary, secondary))
        } else if signed {
            Ok(self.builder.ins().icmp(IntCC::SignedGreaterThan, primary, secondary))
        } else {
            Ok(self.builder.ins().icmp(IntCC::UnsignedGreaterThan, primary, secondary))
        }
    }

    pub fn greater_or_equal(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let signed = self.infer_signedness(&left).unwrap_or(true)
            && self.infer_signedness(&right).unwrap_or(true);

        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            Ok(self.builder.ins().fcmp(FloatCC::GreaterThanOrEqual, primary, secondary))
        } else if signed {
            Ok(self.builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, primary, secondary))
        } else {
            Ok(self.builder.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, primary, secondary))
        }
    }
}
