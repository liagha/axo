use {
    crate::{
        format::Show,
        data::{Boolean, Str},
        internal::hash::{Hash, Hasher},
    },
};

#[derive(Debug, Eq)]
pub struct Binding<Target, Value, Type> {
    pub target: Target,
    pub value: Option<Value>,
    pub annotation: Option<Type>,
    pub kind: BindingKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BindingKind {
    Constant,
    Variable,
    Generic,
}

#[derive(Debug, Eq)]
pub struct Structure<Target, Field> {
    pub target: Target,
    pub members: Vec<Field>,
}

#[derive(Debug)]
pub struct Module<Target> {
    pub target: Target,
}

#[derive(Debug, Eq)]
pub struct Method<Target, Parameter, Body, Output> {
    pub target: Target,
    pub members: Vec<Parameter>,
    pub body: Body,
    pub output: Output,
    pub interface: Interface,
    pub variadic: Boolean,
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
    pub fn new(
        target: Target,
        value: Option<Value>,
        annotation: Option<Type>,
        kind: BindingKind,
    ) -> Self {
        Binding {
            target,
            value,
            annotation,
            kind,
        }
    }
}

impl<Target, Field> Structure<Target, Field> {
    #[inline]
    pub fn new(target: Target, fields: Vec<Field>) -> Self {
        Structure {
            target,
            members: fields,
        }
    }
}

impl<Target, Parameter, Body, Output> Method<Target, Parameter, Body, Output> {
    #[inline]
    pub fn new(
        target: Target,
        members: Vec<Parameter>,
        body: Body,
        output: Output,
        interface: Interface,
        variadic: Boolean,
        entry: Boolean,
    ) -> Self {
        Method {
            target,
            members,
            body,
            output,
            interface,
            variadic,
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

impl<Target: Hash, Value: Hash, Type: Hash> Hash for Binding<Target, Value, Type> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.value.hash(state);
        self.annotation.hash(state);
        self.kind.hash(state);
    }
}

impl<Target: Hash, Field: Hash> Hash for Structure<Target, Field> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.members.hash(state);
    }
}

impl<Target: Hash, Parameter: Hash, Body: Hash, Output: Hash> Hash
    for Method<Target, Parameter, Body, Output>
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

impl<Target: PartialEq, Field: PartialEq> PartialEq for Structure<Target, Field> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.members == other.members
    }
}

impl<Target: PartialEq, Parameter: PartialEq, Body: PartialEq, Output: PartialEq> PartialEq
    for Method<Target, Parameter, Body, Output>
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

impl<Target: Clone, Field: Clone> Clone for Structure<Target, Field> {
    fn clone(&self) -> Self {
        Structure::new(self.target.clone(), self.members.clone())
    }
}

impl<Target: Clone, Parameter: Clone, Body: Clone, Output: Clone> Clone
    for Method<Target, Parameter, Body, Output>
{
    fn clone(&self) -> Self {
        Method::new(
            self.target.clone(),
            self.members.clone(),
            self.body.clone(),
            self.output.clone(),
            self.interface.clone(),
            self.variadic.clone(),
            self.entry.clone(),
        )
    }
}

impl<Target: Clone> Clone for Module<Target> {
    fn clone(&self) -> Self {
        Module::new(self.target.clone())
    }
}

impl<
        'show,
        Target: Show<'show, Verbosity = u8>,
        Value: Show<'show, Verbosity = u8>,
        Type: Show<'show, Verbosity = u8>,
    > Show<'show> for Binding<Target, Value, Type>
{
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => format!(
                "Binding({:?} | {}{}{})",
                self.kind,
                self.target.format(verbosity),
                if let Some(annotation) = &self.annotation {
                    format!(" : {}", annotation.format(verbosity))
                } else {
                    "".to_string()
                },
                if let Some(value) = &self.value {
                    format!(" = {}", value.format(verbosity))
                } else {
                    "".to_string()
                }
            )
            .into(),

            _ => self.format(verbosity - 1),
        }
    }
}

impl<'show, Target: Show<'show, Verbosity = u8>, Member: Show<'show, Verbosity = u8>> Show<'show>
    for Structure<Target, Member>
{
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => format!(
                "Structure({})[{}]",
                self.target.format(verbosity),
                self.members.format(verbosity)
            )
            .into(),

            _ => self.format(verbosity - 1),
        }
    }
}

impl<
        'show,
        Target: Show<'show, Verbosity = u8>,
        Parameter: Show<'show, Verbosity = u8>,
        Body: Show<'show, Verbosity = u8>,
        Output: Show<'show, Verbosity = u8>,
    > Show<'show> for Method<Target, Parameter, Body, Output>
{
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => format!(
                "Method({}{} : {})[{}{}]{{{}}}",
                format!("{:?} | ", self.interface),
                self.target.format(verbosity),
                self.output.format(verbosity),
                if self.variadic { "Variadic | " } else { "" },
                self.members.format(verbosity),
                self.body.format(verbosity)
            )
            .into(),

            _ => self.format(verbosity - 1),
        }
    }
}

impl<'show, Target: Show<'show, Verbosity = u8>> Show<'show>
for Module<Target>
{
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => format!(
                "Module({})",
                self.target.format(verbosity),
            )
                .into(),

            _ => self.format(verbosity - 1),
        }
    }
}