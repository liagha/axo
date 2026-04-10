use orbyte::Orbyte;
use crate::{
    data::Boolean,
    internal::{
        hash::{Hash, Hasher},
    },
};

#[derive(Debug, Eq, Orbyte)]
pub struct Binding<Target, Value, Type> {
    pub target: Target,
    pub value: Option<Value>,
    pub annotation: Type,
    pub kind: BindingKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Orbyte, PartialEq)]
pub enum BindingKind {
    Static,
    Constant,
    Variable,
    Meta,
}

#[derive(Debug, Eq, Orbyte)]
pub struct Aggregate<Target, Field> {
    pub target: Target,
    pub members: Vec<Field>,
}

#[derive(Debug, Orbyte)]
pub struct Module<Target> {
    pub target: Target,
}

#[derive(Debug, Eq, Orbyte)]
pub struct Function<Target, Parameter, Body, Output> {
    pub target: Target,
    pub members: Vec<Parameter>,
    pub body: Body,
    pub output: Output,
    pub interface: Interface,
    pub entry: Boolean,
    pub variadic: Boolean,
}

#[derive(Clone, Copy, Debug, Eq, Orbyte, PartialEq)]
pub enum Interface {
    C,
    Rust,
    Axo,
    Compiler,
}

impl<Target, Value, Type> Binding<Target, Value, Type> {
    #[inline]
    pub fn new(target: Target, value: Option<Value>, annotation: Type, kind: BindingKind) -> Self {
        Binding {
            target,
            value,
            annotation,
            kind,
        }
    }
}

impl<Target, Field> Aggregate<Target, Field> {
    #[inline]
    pub fn new(target: Target, fields: Vec<Field>) -> Self {
        Aggregate {
            target,
            members: fields,
        }
    }
}

impl<Target, Parameter, Body, Output> Function<Target, Parameter, Body, Output> {
    #[inline]
    pub fn new(
        target: Target,
        members: Vec<Parameter>,
        body: Body,
        output: Output,
        interface: Interface,
        entry: Boolean,
        variadic: Boolean,
    ) -> Self {
        Function {
            target,
            members,
            body,
            output,
            interface,
            entry,
            variadic,
        }
    }
}

impl<Target> Module<Target> {
    #[inline]
    pub fn new(target: Target) -> Self {
        Module { target }
    }
}

impl<Target: Hash, Value: Hash, Type: Hash> Hash for Binding<Target, Value, Type> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.value.hash(state);
        self.annotation.hash(state);
        self.kind.hash(state);
    }
}

impl<Target: Hash, Field: Hash> Hash for Aggregate<Target, Field> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.members.hash(state);
    }
}

impl<Target: Hash, Parameter: Hash, Body: Hash, Output: Hash> Hash
    for Function<Target, Parameter, Body, Output>
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.members.hash(state);
        self.body.hash(state);
        self.output.hash(state);
    }
}

impl<Target: Hash> Hash for Module<Target> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
    }
}

impl<Target: PartialEq, Value: PartialEq, Type: PartialEq> PartialEq
    for Binding<Target, Value, Type>
{
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
            && self.value == other.value
            && self.annotation == other.annotation
            && self.kind == other.kind
    }
}

impl<Target: PartialEq, Field: PartialEq> PartialEq for Aggregate<Target, Field> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.members == other.members
    }
}

impl<Target: PartialEq, Parameter: PartialEq, Body: PartialEq, Output: PartialEq> PartialEq
    for Function<Target, Parameter, Body, Output>
{
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
            && self.members == other.members
            && self.body == other.body
            && self.output == other.output
    }
}

impl<Target: PartialEq> PartialEq for Module<Target> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
    }
}

impl<Target: Clone, Value: Clone, Type: Clone> Clone for Binding<Target, Value, Type> {
    fn clone(&self) -> Self {
        Binding::new(
            self.target.clone(),
            self.value.clone(),
            self.annotation.clone(),
            self.kind.clone(),
        )
    }
}

impl<Target: Clone, Field: Clone> Clone for Aggregate<Target, Field> {
    fn clone(&self) -> Self {
        Aggregate::new(self.target.clone(), self.members.clone())
    }
}

impl<Target: Clone, Parameter: Clone, Body: Clone, Output: Clone> Clone
    for Function<Target, Parameter, Body, Output>
{
    fn clone(&self) -> Self {
        Function::new(
            self.target.clone(),
            self.members.clone(),
            self.body.clone(),
            self.output.clone(),
            self.interface.clone(),
            self.entry.clone(),
            self.variadic.clone(),
        )
    }
}

impl<Target: Clone> Clone for Module<Target> {
    fn clone(&self) -> Self {
        Module::new(self.target.clone())
    }
}
