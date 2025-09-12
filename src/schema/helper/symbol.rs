use {
    crate::{
        data::{Boolean},
        internal::hash::{Hash, Hasher},
    },
};
use crate::resolver::Id;

#[derive(Debug, Eq)]
pub struct Inclusion<Target, Id> {
    pub target: Target,
    pub id: Id,
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
    pub constant: Boolean,
}

#[derive(Debug, Eq)]
pub struct Structure<Target, Field> {
    pub target: Target,
    pub members: Vec<Field>,
}

#[derive(Debug, Eq)]
pub struct Method<Target, Parameter, Body, Output> {
    pub target: Target,
    pub members: Vec<Parameter>,
    pub body: Body,
    pub output: Output,
    pub variadic: Boolean,
}

#[derive(Debug)]
pub struct Module<Target> {
    pub target: Target,
}

impl<Target, Id> Inclusion<Target, Id> {
    #[inline]
    pub fn new(target: Target, id: Id) -> Self {
        Inclusion { target, id }
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
        Structure { target, members: fields }
    }
}

impl<Target, Parameter, Body, Output> Method<Target, Parameter, Body, Output> {
    #[inline]
    pub fn new(target: Target, parameters: Vec<Parameter>, body: Body, output: Output, variadic: bool) -> Self {
        Method { target, members: parameters, body, output, variadic }
    }
}

impl<Target> Module<Target> {
    #[inline]
    pub fn new(target: Target) -> Self {
        Module { target }
    }
}

impl<Target: Hash, Id: Hash> Hash for Inclusion<Target, Id> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.id.hash(state);
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
        self.members.hash(state);
    }
}

impl<Target: Hash, Parameter: Hash, Body: Hash, Output: Hash> Hash for Method<Target, Parameter, Body, Output> {
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

impl<Target: PartialEq, Id> PartialEq for Inclusion<Target, Id> {
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
        self.target == other.target && self.members == other.members
    }
}

impl<Target: PartialEq, Parameter: PartialEq, Body: PartialEq, Output: PartialEq> PartialEq for Method<Target, Parameter, Body, Output> {
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

impl<Target: Clone, Id: Clone> Clone for Inclusion<Target, Id> {
    fn clone(&self) -> Self {
        Inclusion::new(self.target.clone(), self.id.clone())
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
        Structure::new(self.target.clone(), self.members.clone())
    }
}

impl<Target: Clone, Parameter: Clone, Body: Clone, Output: Clone> Clone for Method<Target, Parameter, Body, Output> {
    fn clone(&self) -> Self {
        Method::new(
            self.target.clone(),
            self.members.clone(),
            self.body.clone(),
            self.output.clone(),
            self.variadic.clone(),
        )
    }
}

impl<Target: Clone> Clone for Module<Target> {
    fn clone(&self) -> Self {
        Module::new(self.target.clone())
    }
}