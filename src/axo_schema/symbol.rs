use crate::artifact::Artifact;
use crate::axo_form::form::Form;
use crate::hash::{Hash, Hasher};

#[derive(Eq)]
pub struct Inclusion<Target> {
    target: Target,
}

#[derive(Eq)]
pub struct Formation {
    identifier: Artifact,
    form: Form<Artifact, Artifact, Artifact>,
}

#[derive(Eq)]
pub struct Implementation<Target, Interface, Member> {
    target: Target,
    interface: Option<Interface>,
    members: Vec<Member>,
}

#[derive(Eq)]
pub struct Interface<Target, Member> {
    target: Target,
    members: Vec<Member>,
}

#[derive(Eq)]
pub struct Binding<Target, Value, Type> {
    target: Target,
    value: Option<Value>,
    ty: Option<Type>,
    mutable: bool,
}

#[derive(Eq)]
pub struct Structure<Name, Field> {
    name: Name,
    fields: Vec<Field>,
}

#[derive(Eq)]
pub struct Enumeration<Name, Variant> {
    name: Name,
    variants: Vec<Variant>,
}

#[derive(Eq)]
pub struct Method<Name, Parameter, Body, Output> {
    name: Name,
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

impl Formation {
    #[inline]
    pub fn new(identifier: Artifact, form: Form<Artifact, Artifact, Artifact>) -> Self {
        Formation { identifier, form }
    }

    #[inline]
    pub fn get_identifier(&self) -> &Artifact {
        &self.identifier
    }

    #[inline]
    pub fn get_form(&self) -> &Form<Artifact, Artifact, Artifact> {
        &self.form
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

impl<Name, Field> Structure<Name, Field> {
    #[inline]
    pub fn new(name: Name, fields: Vec<Field>) -> Self {
        Structure { name, fields }
    }

    #[inline]
    pub fn get_name(&self) -> &Name {
        &self.name
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

impl<Name, Variant> Enumeration<Name, Variant> {
    #[inline]
    pub fn new(name: Name, variants: Vec<Variant>) -> Self {
        Enumeration { name, variants }
    }

    #[inline]
    pub fn get_name(&self) -> &Name {
        &self.name
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

impl<Name, Parameter, Body, Output> Method<Name, Parameter, Body, Output> {
    #[inline]
    pub fn new(name: Name, parameters: Vec<Parameter>, body: Body, output: Output) -> Self {
        Method { name, parameters, body, output }
    }

    #[inline]
    pub fn get_name(&self) -> &Name {
        &self.name
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

// Hash implementations
impl<Target: Hash> Hash for Inclusion<Target> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_target().hash(state);
    }
}

impl Hash for Formation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_identifier().hash(state);
        self.get_form().hash(state);
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

impl<Name: Hash, Field: Hash> Hash for Structure<Name, Field> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_name().hash(state);
        self.get_fields().hash(state);
    }
}

impl<Name: Hash, Variant: Hash> Hash for Enumeration<Name, Variant> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_name().hash(state);
        self.get_variants().hash(state);
    }
}

impl<Name: Hash, Parameter: Hash, Body: Hash, Output: Hash> Hash for Method<Name, Parameter, Body, Output> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_name().hash(state);
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

impl PartialEq for Formation {
    fn eq(&self, other: &Self) -> bool {
        self.get_identifier() == other.get_identifier()
            && self.get_form() == other.get_form()
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

impl<Name: PartialEq, Field: PartialEq> PartialEq for Structure<Name, Field> {
    fn eq(&self, other: &Self) -> bool {
        self.get_name() == other.get_name() && self.get_fields() == other.get_fields()
    }
}

impl<Name: PartialEq, Variant: PartialEq> PartialEq for Enumeration<Name, Variant> {
    fn eq(&self, other: &Self) -> bool {
        self.get_name() == other.get_name() && self.get_variants() == other.get_variants()
    }
}

impl<Name: PartialEq, Parameter: PartialEq, Body: PartialEq, Output: PartialEq> PartialEq for Method<Name, Parameter, Body, Output> {
    fn eq(&self, other: &Self) -> bool {
        self.get_name() == other.get_name()
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

impl Clone for Formation {
    fn clone(&self) -> Self {
        Formation::new(self.get_identifier().clone(), self.get_form().clone())
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

impl<Name: Clone, Field: Clone> Clone for Structure<Name, Field> {
    fn clone(&self) -> Self {
        Structure::new(self.get_name().clone(), self.get_fields().clone())
    }
}

impl<Name: Clone, Variant: Clone> Clone for Enumeration<Name, Variant> {
    fn clone(&self) -> Self {
        Enumeration::new(self.get_name().clone(), self.get_variants().clone())
    }
}

impl<Name: Clone, Parameter: Clone, Body: Clone, Output: Clone> Clone for Method<Name, Parameter, Body, Output> {
    fn clone(&self) -> Self {
        Method::new(
            self.get_name().clone(),
            self.get_parameters().clone(),
            self.get_body().clone(),
            self.get_output().clone()
        )
    }
}