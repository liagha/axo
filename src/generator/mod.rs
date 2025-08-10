use inkwell::types::AnyType;
use inkwell::values::{AnyValue, AsValueRef, BasicMetadataValueEnum};
use {
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum},
        values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue},
        AddressSpace, IntPredicate, FloatPredicate,
    },
    std::collections::HashMap,
};

use crate::{
    parser::{Element, ElementKind, Symbol},
    scanner::{Operator, Token, TokenKind},
    schema::{Binding, Method, Structure, Enumeration},
};
use crate::data::string::Str;
use crate::parser::Symbolic;
use crate::scanner::OperatorKind;
use crate::schema::{Binary, Unary};

pub struct Generator<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
    entry: Option<FunctionValue<'ctx>>,
}

impl<'ctx> Generator<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("program");
        let builder = context.create_builder();

        Self {
            context,
            builder,
            module,
            entry: None,
        }
    }

    pub fn execute_pipeline(&mut self, _resolver: &mut dyn std::any::Any, elements: Vec<Element>) {
        self.generate(elements);
    }

    pub fn generate(&mut self, elements: Vec<Element>) {
        let main_ty = self.context.i32_type().fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_ty, None);
        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);
        self.entry = Some(main_fn);

        let mut has_return = false;
        for element in elements {
            self.generate_statement(&element);
            if matches!(element.kind, ElementKind::Binary(_)) {
                has_return = true; // Track if a binary expression generated a return
            }
        }

        if !has_return {
            let zero = self.context.i32_type().const_zero();
            self.builder.build_return(Some(&zero)).unwrap();
        }

        self.module.print_to_file("program.bc").unwrap();
    }

    fn generate_statement(&mut self, elem: &Element) {
        match &elem.kind {
            ElementKind::Binary(bin) => {
                if let Some(value) = self.generate_binary(bin) {
                    self.builder.build_return(Some(&value)).unwrap();
                }
            }
            _ => {}
        }
    }

    fn generate_literal(&self, literal: &TokenKind) -> Option<BasicValueEnum<'ctx>> {
        match literal {
            TokenKind::Integer(integer) => {
                Some(self.context.i32_type().const_int(*integer as u64, false).into())
            },
            TokenKind::Float(float) => {
                Some(self.context.f16_type().const_float(float.0).into())
            }
            TokenKind::Boolean(bool) => {
                Some(self.context.bool_type().const_int(*bool as u64, false).into())
            }
            _ => None,
        }
    }

    fn generate_expr(&mut self, elem: &Element) -> Option<BasicValueEnum<'ctx>> {
        match &elem.kind {
            ElementKind::Literal(lit) => self.generate_literal(lit),
            ElementKind::Binary(bin) => self.generate_binary(bin),
            ElementKind::Unary(un) => self.generate_unary(un),
            _ => None,
        }
    }

    fn generate_binary(&mut self, bin: &Binary<Box<Element>, Token, Box<Element>>) -> Option<BasicValueEnum<'ctx>> {
        let left = self.generate_expr(&bin.get_left());
        let right = self.generate_expr(&bin.get_right());
        if let (Some(l), Some(r)) = (left, right) {
            match bin.get_operator().kind.try_unwrap_operator() {
                Some(op) => match op {
                    OperatorKind::Plus => Some(self.builder.build_int_add(l.into_int_value(), r.into_int_value(), "add").unwrap().into()),
                    OperatorKind::Minus => Some(self.builder.build_int_sub(l.into_int_value(), r.into_int_value(), "sub").unwrap().into()),
                    OperatorKind::Star => Some(self.builder.build_int_mul(l.into_int_value(), r.into_int_value(), "mul").unwrap().into()),
                    OperatorKind::Slash => Some(self.builder.build_int_unsigned_div(l.into_int_value(), r.into_int_value(), "div").unwrap().into()),
                    _ => None,
                },
                _ => None,
            }
        } else {
            None
        }
    }

    fn generate_unary(&mut self, un: &Unary<Token, Box<Element>>) -> Option<BasicValueEnum<'ctx>> {
        let opnd = self.generate_expr(&un.get_operand());
        if let Some(o) = opnd {
            match un.get_operator().kind.try_unwrap_operator() {
                Some(op) => match op {
                    OperatorKind::Minus => Some(self.builder.build_int_neg(o.into_int_value(), "neg").unwrap().into()),
                    _ => None,
                },
                _ => None,
            }
        } else {
            None
        }
    }
}