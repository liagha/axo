use {
    crate::{
        hash::{Hash, Hasher},
    },
};

#[derive(Debug, Eq)]
pub struct Inclusion<Target> {
    target: Target,
}

#[derive(Debug, Eq)]
pub struct Implementation<Target, Interface, Member> {
    target: Target,
    interface: Option<Interface>,
    members: Vec<Member>,
}

#[derive(Debug, Eq)]
pub struct Interface<Target, Member> {
    target: Target,
    members: Vec<Member>,
}

#[derive(Debug, Eq)]
pub struct Binding<Target, Value, Type> {
    target: Target,
    value: Option<Value>,
    ty: Option<Type>,
    mutable: bool,
}

#[derive(Debug, Eq)]
pub struct Structure<Target, Field> {
    target: Target,
    fields: Vec<Field>,
}

#[derive(Debug, Eq)]
pub struct Enumeration<Target, Variant> {
    target: Target,
    variants: Vec<Variant>,
}

#[derive(Debug, Eq)]
pub struct Method<Target, Parameter, Body, Output> {
    target: Target,
    parameters: Vec<Parameter>,
    body: Body,
    output: Output,
}

impl<Target> Inclusion<Target> {
    #[inline]
    pub fn new(target: Target) -> Self {
        Inclusion { target }
    }

    #[inline]
    pub fn get_target(&self) -> &Target {
        &self.target
    }
}

impl<Target, Interface, Member> Implementation<Target, Interface, Member> {
    #[inline]
    pub fn new(target: Target, interface: Option<Interface>, members: Vec<Member>) -> Self {
        Implementation { interface, target, members }
    }

    #[inline]
    pub fn get_interface(&self) -> &Option<Interface> {
        &self.interface
    }

    #[inline]
    pub fn get_target(&self) -> &Target {
        &self.target
    }

    #[inline]
    pub fn get_members(&self) -> &Vec<Member> {
        &self.members
    }
}

impl<Target, Member> Interface<Target, Member> {
    #[inline]
    pub fn new(target: Target, members: Vec<Member>) -> Self {
        Interface { target, members }
    }

    #[inline]
    pub fn get_target(&self) -> &Target {
        &self.target
    }

    #[inline]
    pub fn get_members(&self) -> &Vec<Member> {
        &self.members
    }
}

impl<Target, Value, Type> Binding<Target, Value, Type> {
    #[inline]
    pub fn new(target: Target, value: Option<Value>, ty: Option<Type>, mutable: bool) -> Self {
        Binding { target, value, ty, mutable }
    }

    #[inline]
    pub fn get_target(&self) -> &Target {
        &self.target
    }

    #[inline]
    pub fn get_value(&self) -> Option<&Value> {
        self.value.as_ref()
    }

    #[inline]
    pub fn get_type(&self) -> Option<&Type> {
        self.ty.as_ref()
    }

    #[inline]
    pub fn is_mutable(&self) -> bool {
        self.mutable
    }
}

impl<Target, Field> Structure<Target, Field> {
    #[inline]
    pub fn new(target: Target, fields: Vec<Field>) -> Self {
        Structure { target, fields }
    }

    #[inline]
    pub fn get_target(&self) -> &Target {
        &self.target
    }

    #[inline]
    pub fn get_fields(&self) -> &Vec<Field> {
        &self.fields
    }

    #[inline]
    pub fn get_field(&self, index: usize) -> Option<&Field> {
        self.fields.get(index)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

impl<Target, Variant> Enumeration<Target, Variant> {
    #[inline]
    pub fn new(target: Target, variants: Vec<Variant>) -> Self {
        Enumeration { target, variants }
    }

    #[inline]
    pub fn get_target(&self) -> &Target {
        &self.target
    }

    #[inline]
    pub fn get_variants(&self) -> &Vec<Variant> {
        &self.variants
    }

    #[inline]
    pub fn get_variant(&self, index: usize) -> Option<&Variant> {
        self.variants.get(index)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.variants.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.variants.is_empty()
    }
}

impl<Target, Parameter, Body, Output> Method<Target, Parameter, Body, Output> {
    #[inline]
    pub fn new(target: Target, parameters: Vec<Parameter>, body: Body, output: Output) -> Self {
        Method { target, parameters, body, output }
    }

    #[inline]
    pub fn get_target(&self) -> &Target {
        &self.target
    }

    #[inline]
    pub fn get_parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    #[inline]
    pub fn get_parameter(&self, index: usize) -> Option<&Parameter> {
        self.parameters.get(index)
    }

    #[inline]
    pub fn get_body(&self) -> &Body {
        &self.body
    }

    #[inline]
    pub fn get_output(&self) -> &Output {
        &self.output
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.parameters.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.parameters.is_empty()
    }
}

impl<Target: Hash> Hash for Inclusion<Target> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_target().hash(state);
    }
}

impl<Interface: Hash, Target: Hash, Member: Hash> Hash for Implementation<Target, Interface, Member> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_target().hash(state);
        self.get_members().hash(state);
    }
}

impl<Target: Hash, Member: Hash> Hash for Interface<Target, Member> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_target().hash(state);
        self.get_members().hash(state);
    }
}

impl<Target: Hash, Value: Hash, Type: Hash> Hash for Binding<Target, Value, Type> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_target().hash(state);
        self.get_value().hash(state);
        self.get_type().hash(state);
        self.is_mutable().hash(state);
    }
}

impl<Target: Hash, Field: Hash> Hash for Structure<Target, Field> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_target().hash(state);
        self.get_fields().hash(state);
    }
}

impl<Target: Hash, Variant: Hash> Hash for Enumeration<Target, Variant> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_target().hash(state);
        self.get_variants().hash(state);
    }
}

impl<Target: Hash, Parameter: Hash, Body: Hash, Output: Hash> Hash for Method<Target, Parameter, Body, Output> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_target().hash(state);
        self.get_parameters().hash(state);
        self.get_body().hash(state);
        self.get_output().hash(state);
    }
}

// PartialEq implementations
impl<Target: PartialEq> PartialEq for Inclusion<Target> {
    fn eq(&self, other: &Self) -> bool {
        self.get_target() == other.get_target()
    }
}

impl<Interface: PartialEq, Target: PartialEq, Member: PartialEq> PartialEq for Implementation<Target, Interface, Member> {
    fn eq(&self, other: &Self) -> bool {
        self.get_target() == other.get_target() && self.get_members() == other.get_members()
    }
}

impl<Target: PartialEq, Member: PartialEq> PartialEq for Interface<Target, Member> {
    fn eq(&self, other: &Self) -> bool {
        self.get_target() == other.get_target() && self.get_members() == other.get_members()
    }
}

impl<Target: PartialEq, Value: PartialEq, Type: PartialEq> PartialEq for Binding<Target, Value, Type> {
    fn eq(&self, other: &Self) -> bool {
        self.get_target() == other.get_target()
            && self.get_value() == other.get_value()
            && self.get_type() == other.get_type()
            && self.is_mutable() == other.is_mutable()
    }
}

impl<Target: PartialEq, Field: PartialEq> PartialEq for Structure<Target, Field> {
    fn eq(&self, other: &Self) -> bool {
        self.get_target() == other.get_target() && self.get_fields() == other.get_fields()
    }
}

impl<Target: PartialEq, Variant: PartialEq> PartialEq for Enumeration<Target, Variant> {
    fn eq(&self, other: &Self) -> bool {
        self.get_target() == other.get_target() && self.get_variants() == other.get_variants()
    }
}

impl<Target: PartialEq, Parameter: PartialEq, Body: PartialEq, Output: PartialEq> PartialEq for Method<Target, Parameter, Body, Output> {
    fn eq(&self, other: &Self) -> bool {
        self.get_target() == other.get_target()
            && self.get_parameters() == other.get_parameters()
            && self.get_body() == other.get_body()
            && self.get_output() == other.get_output()
    }
}

// Clone implementations
impl<Target: Clone> Clone for Inclusion<Target> {
    fn clone(&self) -> Self {
        Inclusion::new(self.get_target().clone())
    }
}

impl<Interface: Clone, Target: Clone, Member: Clone> Clone for Implementation<Target, Interface, Member> {
    fn clone(&self) -> Self {
        Implementation::new(self.get_target().clone(), self.get_interface().clone(), self.get_members().clone())
    }
}

impl<Target: Clone, Member: Clone> Clone for Interface<Target, Member> {
    fn clone(&self) -> Self {
        Interface::new(self.get_target().clone(), self.get_members().clone())
    }
}

impl<Target: Clone, Value: Clone, Type: Clone> Clone for Binding<Target, Value, Type> {
    fn clone(&self) -> Self {
        Binding::new(
            self.get_target().clone(),
            self.get_value().cloned(),
            self.get_type().cloned(),
            self.is_mutable(),
        )
    }
}

impl<Target: Clone, Field: Clone> Clone for Structure<Target, Field> {
    fn clone(&self) -> Self {
        Structure::new(self.get_target().clone(), self.get_fields().clone())
    }
}

impl<Target: Clone, Variant: Clone> Clone for Enumeration<Target, Variant> {
    fn clone(&self) -> Self {
        Enumeration::new(self.get_target().clone(), self.get_variants().clone())
    }
}

impl<Target: Clone, Parameter: Clone, Body: Clone, Output: Clone> Clone for Method<Target, Parameter, Body, Output> {
    fn clone(&self) -> Self {
        Method::new(
            self.get_target().clone(),
            self.get_parameters().clone(),
            self.get_body().clone(),
            self.get_output().clone()
        )
    }
}