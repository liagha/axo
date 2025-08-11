use inkwell::types::AnyType;
use inkwell::values::{AnyValue, AsValueRef, BasicMetadataValueEnum};
use inkwell::{builder::Builder, context::Context, module::Module, types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum}, values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue}, AddressSpace, IntPredicate, FloatPredicate};
use std::collections::HashMap;

use crate::{parser::{Element, ElementKind, Symbol}, scanner::{Operator, Token, TokenKind}, schema::{Binding, Method, Structure, Enumeration}};
use crate::data::{Boolean, Integer, Scale};
use crate::data::float::Float;
use crate::data::string::Str;
use crate::parser::Symbolic;
use crate::scanner::OperatorKind;
use crate::schema::{Binary, Unary};

pub trait Backend {
    type Value;
    type Function;
    type Block;

    fn integer(&mut self, value: Integer, size: Scale, signed: Boolean) -> Self::Value;
    fn float(&mut self, value: Float, size: Scale) -> Self::Value;
    fn boolean(&mut self, value: Boolean) -> Self::Value;
    fn array(&mut self, value: Vec<Element>, size: Scale) -> Self::Value;
    fn function(&mut self, target: Element, body: Element, output: Element) -> Self::Value;

    fn create_function(&mut self, name: &str, return_type: &str) -> Self::Function;
    fn create_block(&mut self, function: &Self::Function, name: &str) -> Self::Block;
    fn position_at_block(&mut self, block: &Self::Block);
    fn set_entry_function(&mut self, function: Self::Function);

    fn build_add(&mut self, left: Self::Value, right: Self::Value, name: &str) -> Self::Value;
    fn build_sub(&mut self, left: Self::Value, right: Self::Value, name: &str) -> Self::Value;
    fn build_mul(&mut self, left: Self::Value, right: Self::Value, name: &str) -> Self::Value;
    fn build_div(&mut self, left: Self::Value, right: Self::Value, name: &str) -> Self::Value;
    fn build_neg(&mut self, operand: Self::Value, name: &str) -> Self::Value;

    fn build_return(&mut self, value: Option<Self::Value>);

    fn finalize(&mut self, filename: &str);

    fn get_zero_value(&mut self, type_name: &str) -> Self::Value;
}

pub struct Inkwell<'backend> {
    context: &'backend Context,
    builder: Builder<'backend>,
    module: Module<'backend>,
    entry: Option<FunctionValue<'backend>>,
}

impl<'backend> Inkwell<'backend> {
    pub fn new(context: &'backend Context) -> Self {
        let module = context.create_module("program");
        let builder = context.create_builder();
        Self {
            context,
            builder,
            module,
            entry: None,
        }
    }
}

impl<'backend> Backend for Inkwell<'backend> {
    type Value = BasicValueEnum<'backend>;
    type Function = FunctionValue<'backend>;
    type Block = inkwell::basic_block::BasicBlock<'backend>;

    fn integer(&mut self, value: Integer, size: Scale, signed: Boolean) -> Self::Value {
        let ty = match size {
            8 => self.context.i8_type(),
            16 => self.context.i16_type(),
            32 => self.context.i32_type(),
            64 => self.context.i64_type(),
            _ => self.context.i32_type(),
        };
        ty.const_int(value as u64, false).into()
    }

    fn float(&mut self, value: Float, size: Scale) -> Self::Value {
        let ty = match size {
            16 => self.context.f16_type(),
            32 => self.context.f32_type(),
            64 => self.context.f64_type(),
            128 => self.context.f128_type(),
            _ => self.context.f32_type(),
        };
        ty.const_float(value.0).into()
    }

    fn boolean(&mut self, value: Boolean) -> Self::Value {
        self.context.bool_type().const_int(value as u64, false).into()
    }

    fn array(&mut self, value: Vec<Element>, size: Scale) -> Self::Value {
        unimplemented!()
    }

    fn function(&mut self, target: Element, body: Element, output: Element) -> Self::Value {
        unimplemented!()
    }

    fn create_function(&mut self, name: &str, return_type: &str) -> Self::Function {
        let fn_type = match return_type {
            "i32" => self.context.i32_type().fn_type(&[], false),
            "f32" => self.context.f32_type().fn_type(&[], false),
            "bool" => self.context.bool_type().fn_type(&[], false),
            _ => self.context.i32_type().fn_type(&[], false),
        };
        self.module.add_function(name, fn_type, None)
    }

    fn create_block(&mut self, function: &Self::Function, name: &str) -> Self::Block {
        self.context.append_basic_block(*function, name)
    }

    fn position_at_block(&mut self, block: &Self::Block) {
        self.builder.position_at_end(*block);
    }

    fn set_entry_function(&mut self, function: Self::Function) {
        self.entry = Some(function);
    }

    fn build_add(&mut self, left: Self::Value, right: Self::Value, name: &str) -> Self::Value {
        self.builder.build_int_add(left.into_int_value(), right.into_int_value(), name).unwrap().into()
    }

    fn build_sub(&mut self, left: Self::Value, right: Self::Value, name: &str) -> Self::Value {
        self.builder.build_int_sub(left.into_int_value(), right.into_int_value(), name).unwrap().into()
    }

    fn build_mul(&mut self, left: Self::Value, right: Self::Value, name: &str) -> Self::Value {
        self.builder.build_int_mul(left.into_int_value(), right.into_int_value(), name).unwrap().into()
    }

    fn build_div(&mut self, left: Self::Value, right: Self::Value, name: &str) -> Self::Value {
        self.builder.build_int_unsigned_div(left.into_int_value(), right.into_int_value(), name).unwrap().into()
    }

    fn build_neg(&mut self, operand: Self::Value, name: &str) -> Self::Value {
        self.builder.build_int_neg(operand.into_int_value(), name).unwrap().into()
    }

    fn build_return(&mut self, value: Option<Self::Value>) {
        match value {
            Some(val) => { self.builder.build_return(Some(&val)).unwrap(); }
            None => { self.builder.build_return(None).unwrap(); }
        }
    }

    fn finalize(&mut self, filename: &str) {
        self.module.print_to_file(filename).unwrap();
    }

    fn get_zero_value(&mut self, type_name: &str) -> Self::Value {
        match type_name {
            "i32" => self.context.i32_type().const_zero().into(),
            "f32" => self.context.f32_type().const_zero().into(),
            "bool" => self.context.bool_type().const_zero().into(),
            _ => self.context.i32_type().const_zero().into(),
        }
    }
}

pub struct Generator<B: Backend> {
    backend: B,
}

impl<B: Backend> Generator<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
        }
    }

    pub fn execute_pipeline(&mut self, _resolver: &mut dyn std::any::Any, elements: Vec<Element>) {
        self.generate(elements);
    }

    pub fn generate(&mut self, elements: Vec<Element>) {
        let main_fn = self.backend.create_function("main", "i32");
        let entry_block = self.backend.create_block(&main_fn, "entry");
        self.backend.position_at_block(&entry_block);
        self.backend.set_entry_function(main_fn);

        let mut has_return = false;
        for element in elements {
            self.generate_statement(&element);
            if matches!(element.kind, ElementKind::Binary(_)) {
                has_return = true;
            }
        }

        if !has_return {
            let zero = self.backend.get_zero_value("i32");
            self.backend.build_return(Some(zero));
        }

        self.backend.finalize("program.bc");
    }

    fn generate_statement(&mut self, elem: &Element) {
        match &elem.kind {
            ElementKind::Binary(bin) => {
                if let Some(value) = self.generate_binary(bin) {
                    self.backend.build_return(Some(value));
                }
            }
            _ => {}
        }
    }

    fn generate_literal(&mut self, literal: &TokenKind) -> Option<B::Value> {
        match literal {
            TokenKind::Integer(integer) => Some(self.backend.integer(*integer, 32, false)),
            TokenKind::Float(float) => Some(self.backend.float(*float, 32)),
            TokenKind::Boolean(bool) => Some(self.backend.boolean(*bool)),
            _ => None,
        }
    }

    fn generate_expr(&mut self, elem: &Element) -> Option<B::Value> {
        match &elem.kind {
            ElementKind::Literal(lit) => self.generate_literal(lit),
            ElementKind::Binary(bin) => self.generate_binary(bin),
            ElementKind::Unary(un) => self.generate_unary(un),
            _ => None,
        }
    }

    fn generate_binary(&mut self, bin: &Binary<Box<Element>, Token, Box<Element>>) -> Option<B::Value> {
        let left = self.generate_expr(&bin.get_left());
        let right = self.generate_expr(&bin.get_right());
        if let (Some(l), Some(r)) = (left, right) {
            match bin.get_operator().kind.try_unwrap_operator() {
                Some(op) => match op {
                    OperatorKind::Plus => Some(self.backend.build_add(l, r, "add")),
                    OperatorKind::Minus => Some(self.backend.build_sub(l, r, "sub")),
                    OperatorKind::Star => Some(self.backend.build_mul(l, r, "mul")),
                    OperatorKind::Slash => Some(self.backend.build_div(l, r, "div")),
                    _ => None,
                },
                _ => None,
            }
        } else {
            None
        }
    }

    fn generate_unary(&mut self, un: &Unary<Token, Box<Element>>) -> Option<B::Value> {
        let opnd = self.generate_expr(&un.get_operand());
        if let Some(o) = opnd {
            match un.get_operator().kind.try_unwrap_operator() {
                Some(op) => match op {
                    OperatorKind::Minus => Some(self.backend.build_neg(o, "neg")),
                    _ => None,
                },
                _ => None,
            }
        } else {
            None
        }
    }
}