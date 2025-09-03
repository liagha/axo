use {
    crate::{
        data::{Offset, Scale},
        internal::hash::{Hash, Hasher},
    },
};

#[derive(Debug, Eq)]
pub struct Inclusion<Target> {
    pub target: Target,
}

#[derive(Debug, Eq)]
pub struct Extension<Target, Interface, Member> {
    pub target: Target,
    pub extension: Option<Interface>,
    pub members: Vec<Member>,
}

#[derive(Debug, Eq)]
pub struct Binding<Target, Value, Type> {
    pub target: Target,
    pub value: Option<Value>,
    pub annotation: Option<Type>,
    pub constant: bool,
}

#[derive(Debug, Eq)]
pub struct Structure<Target, Field> {
    pub target: Target,
    pub fields: Vec<Field>,
}

#[derive(Debug, Eq)]
pub struct Enumeration<Target, Variant> {
    pub target: Target,
    pub variants: Vec<Variant>,
}

#[derive(Debug, Eq)]
pub struct Method<Target, Parameter, Body, Output> {
    pub target: Target,
    pub parameters: Vec<Parameter>,
    pub body: Body,
    pub output: Output,
}

#[derive(Debug)]
pub struct Module<Target> {
    pub target: Target,
}

impl<Target> Inclusion<Target> {
    #[inline]
    pub fn new(target: Target) -> Self {
        Inclusion { target }
    }
}

impl<Target, Interface, Member> Extension<Target, Interface, Member> {
    #[inline]
    pub fn new(target: Target, extension: Option<Interface>, members: Vec<Member>) -> Self {
        Extension { target, extension, members }
    }
}

impl<Target, Value, Type> Binding<Target, Value, Type> {
    #[inline]
    pub fn new(target: Target, value: Option<Value>, annotation: Option<Type>, constant: bool) -> Self {
        Binding { target, value, annotation, constant }
    }
}

impl<Target, Field> Structure<Target, Field> {
    #[inline]
    pub fn new(target: Target, fields: Vec<Field>) -> Self {
        Structure { target, fields }
    }
}

impl<Target, Variant> Enumeration<Target, Variant> {
    #[inline]
    pub fn new(target: Target, variants: Vec<Variant>) -> Self {
        Enumeration { target, variants }
    }
}

impl<Target, Parameter, Body, Output> Method<Target, Parameter, Body, Output> {
    #[inline]
    pub fn new(target: Target, parameters: Vec<Parameter>, body: Body, output: Output) -> Self {
        Method { target, parameters, body, output }
    }
}

impl<Target> Module<Target> {
    #[inline]
    pub fn new(target: Target) -> Self {
        Module { target }
    }
}

impl<Target: Hash> Hash for Inclusion<Target> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
    }
}

impl<Interface: Hash, Target: Hash, Member: Hash> Hash for Extension<Target, Interface, Member> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.members.hash(state);
    }
}

impl<Target: Hash, Value: Hash, Type: Hash> Hash for Binding<Target, Value, Type> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.value.hash(state);
        self.annotation.hash(state);
        self.constant.hash(state);
    }
}

impl<Target: Hash, Field: Hash> Hash for Structure<Target, Field> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.fields.hash(state);
    }
}

impl<Target: Hash, Variant: Hash> Hash for Enumeration<Target, Variant> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.variants.hash(state);
    }
}

impl<Target: Hash, Parameter: Hash, Body: Hash, Output: Hash> Hash for Method<Target, Parameter, Body, Output> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.parameters.hash(state);
        self.body.hash(state);
        self.output.hash(state);
    }
}

impl<Target: Hash> Hash for Module<Target> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
    }
}

impl<Target: PartialEq> PartialEq for Inclusion<Target> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
    }
}

impl<Interface: PartialEq, Target: PartialEq, Member: PartialEq> PartialEq for Extension<Target, Interface, Member> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.members == other.members
    }
}

impl<Target: PartialEq, Value: PartialEq, Type: PartialEq> PartialEq for Binding<Target, Value, Type> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
            && self.value == other.value
            && self.annotation == other.annotation
            && self.constant == other.constant
    }
}

impl<Target: PartialEq, Field: PartialEq> PartialEq for Structure<Target, Field> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.fields == other.fields
    }
}

impl<Target: PartialEq, Variant: PartialEq> PartialEq for Enumeration<Target, Variant> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.variants == other.variants
    }
}

impl<Target: PartialEq, Parameter: PartialEq, Body: PartialEq, Output: PartialEq> PartialEq for Method<Target, Parameter, Body, Output> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
            && self.parameters == other.parameters
            && self.body == other.body
            && self.output == other.output
    }
}

impl<Target: PartialEq> PartialEq for Module<Target> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
    }
}

impl<Target: Clone> Clone for Inclusion<Target> {
    fn clone(&self) -> Self {
        Inclusion::new(self.target.clone())
    }
}

impl<Interface: Clone, Target: Clone, Member: Clone> Clone for Extension<Target, Interface, Member> {
    fn clone(&self) -> Self {
        Extension::new(self.target.clone(), self.extension.clone(), self.members.clone())
    }
}

impl<Target: Clone, Value: Clone, Type: Clone> Clone for Binding<Target, Value, Type> {
    fn clone(&self) -> Self {
        Binding::new(
            self.target.clone(),
            self.value.clone(),
            self.annotation.clone(),
            self.constant,
        )
    }
}

impl<Target: Clone, Field: Clone> Clone for Structure<Target, Field> {
    fn clone(&self) -> Self {
        Structure::new(self.target.clone(), self.fields.clone())
    }
}

impl<Target: Clone, Variant: Clone> Clone for Enumeration<Target, Variant> {
    fn clone(&self) -> Self {
        Enumeration::new(self.target.clone(), self.variants.clone())
    }
}

impl<Target: Clone, Parameter: Clone, Body: Clone, Output: Clone> Clone for Method<Target, Parameter, Body, Output> {
    fn clone(&self) -> Self {
        Method::new(
            self.target.clone(),
            self.parameters.clone(),
            self.body.clone(),
            self.output.clone()
        )
    }
}

impl<Target: Clone> Clone for Module<Target> {
    fn clone(&self) -> Self {
        Module::new(self.target.clone())
    }
}