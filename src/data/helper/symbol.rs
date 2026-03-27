use crate::{
    data::Boolean,
    internal::{
        cache::{Decode, Encode},
        hash::{Hash, Hasher},
    },
};

#[derive(Debug, Eq)]
pub struct Binding<Target, Value, Type> {
    pub target: Target,
    pub value: Option<Value>,
    pub annotation: Type,
    pub kind: BindingKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BindingKind {
    Static,
    Constant,
    Variable,
    Meta,
}

#[derive(Debug, Eq)]
pub struct Aggregate<Target, Field> {
    pub target: Target,
    pub members: Vec<Field>,
}

#[derive(Debug)]
pub struct Module<Target> {
    pub target: Target,
}

#[derive(Debug, Eq)]
pub struct Function<Target, Parameter, Body, Output> {
    pub target: Target,
    pub members: Vec<Parameter>,
    pub body: Body,
    pub output: Output,
    pub interface: Interface,
    pub entry: Boolean,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    ) -> Self {
        Function {
            target,
            members,
            body,
            output,
            interface,
            entry,
        }
    }
}

impl<Target> Module<Target> {
    #[inline]
    pub fn new(target: Target) -> Self {
        Module { target }
    }
}

impl<Target: Encode, Value: Encode, Type: Encode> Encode for Binding<Target, Value, Type> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.target.encode(buffer);
        self.value.encode(buffer);
        self.annotation.encode(buffer);
        self.kind.encode(buffer);
    }
}

impl<'element, Target: Decode<'element>, Value: Decode<'element>, Type: Decode<'element>>
    Decode<'element> for Binding<Target, Value, Type>
{
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        Binding {
            target: Target::decode(buffer, cursor),
            value: Option::decode(buffer, cursor),
            annotation: Type::decode(buffer, cursor),
            kind: BindingKind::decode(buffer, cursor),
        }
    }
}

impl Encode for BindingKind {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            BindingKind::Static => buffer.push(0),
            BindingKind::Constant => buffer.push(1),
            BindingKind::Variable => buffer.push(2),
            BindingKind::Meta => buffer.push(3),
        }
    }
}

impl<'element> Decode<'element> for BindingKind {
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        let tag = buffer[*cursor];
        *cursor += 1;
        match tag {
            0 => BindingKind::Static,
            1 => BindingKind::Constant,
            2 => BindingKind::Variable,
            3 => BindingKind::Meta,
            _ => panic!(),
        }
    }
}

impl<Target: Encode, Field: Encode> Encode for Aggregate<Target, Field> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.target.encode(buffer);
        self.members.encode(buffer);
    }
}

impl<'element, Target: Decode<'element>, Field: Decode<'element>> Decode<'element>
    for Aggregate<Target, Field>
{
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        Aggregate {
            target: Target::decode(buffer, cursor),
            members: Vec::decode(buffer, cursor),
        }
    }
}

impl<Target: Encode> Encode for Module<Target> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.target.encode(buffer);
    }
}

impl<'element, Target: Decode<'element>> Decode<'element> for Module<Target> {
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        Module {
            target: Target::decode(buffer, cursor),
        }
    }
}

impl<Target: Encode, Parameter: Encode, Body: Encode, Output: Encode> Encode
    for Function<Target, Parameter, Body, Output>
{
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.target.encode(buffer);
        self.members.encode(buffer);
        self.body.encode(buffer);
        self.output.encode(buffer);
        self.interface.encode(buffer);
        self.entry.encode(buffer);
    }
}

impl<
        'element,
        Target: Decode<'element>,
        Parameter: Decode<'element>,
        Body: Decode<'element>,
        Output: Decode<'element>,
    > Decode<'element> for Function<Target, Parameter, Body, Output>
{
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        Function {
            target: Target::decode(buffer, cursor),
            members: Vec::decode(buffer, cursor),
            body: Body::decode(buffer, cursor),
            output: Output::decode(buffer, cursor),
            interface: Interface::decode(buffer, cursor),
            entry: Boolean::decode(buffer, cursor),
        }
    }
}

impl Encode for Interface {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            Interface::C => buffer.push(0),
            Interface::Rust => buffer.push(1),
            Interface::Axo => buffer.push(2),
            Interface::Compiler => buffer.push(3),
        }
    }
}

impl<'element> Decode<'element> for Interface {
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        let tag = buffer[*cursor];
        *cursor += 1;
        match tag {
            0 => Interface::C,
            1 => Interface::Rust,
            2 => Interface::Axo,
            3 => Interface::Compiler,
            _ => panic!(),
        }
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
        )
    }
}

impl<Target: Clone> Clone for Module<Target> {
    fn clone(&self) -> Self {
        Module::new(self.target.clone())
    }
}
