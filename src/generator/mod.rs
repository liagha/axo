mod error;

use inkwell::types::AnyType;
use inkwell::values::{AnyValue, AsValueRef, BasicMetadataValueEnum};
use inkwell::{builder::Builder, context::Context, module::Module, types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum}, values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue}, AddressSpace, IntPredicate, FloatPredicate};
use std::collections::HashMap;

use crate::{parser::{Element, ElementKind, Symbol}, scanner::{Operator, Token, TokenKind}, schema::{Binding, Method, Structure, Enumeration}};
use crate::analyzer::Analysis;
use crate::analyzer::Instruction;
use crate::data::{Boolean, Integer, Scale};
use crate::data::float::Float;
use crate::data::Str;
use crate::parser::Symbolic;
use crate::scanner::OperatorKind;
use crate::schema::{Binary, Unary};
use error::*;

pub struct Inkwell<'backend> {
    context: &'backend Context,
    builder: Builder<'backend>,
}

impl<'backend> Inkwell<'backend> {
    pub fn new(context: &'backend Context) -> Self {
        let builder = context.create_builder();
        
        Self {
            context,
            builder,
        }
    }

    pub fn instruct(&mut self, analyses: Vec<Analysis<'backend>>) -> Result<(), Error> {
        Ok(())
    }
}