use {
    crate::{
        hash::{Hash, Hasher},
    }
};

#[derive(Eq)]
pub struct Group<Item> {
    pub items: Vec<Item>,
}

#[derive(Eq)]
pub struct Sequence<Item> {
    pub items: Vec<Item>,
}

#[derive(Eq)]
pub struct Collection<Item> {
    pub items: Vec<Item>,
}

#[derive(Eq)]
pub struct Series<Item> {
    pub items: Vec<Item>,
}

#[derive(Eq)]
pub struct Bundle<Item> {
    pub items: Vec<Item>,
}

#[derive(Eq)]
pub struct Scope<Item> {
    pub items: Vec<Item>,
}

#[derive(Eq)]
pub struct Binary<Left, Operator, Right> {
    left: Left,
    operator: Operator,
    right: Right,
}

#[derive(Eq)]
pub struct Unary<Operator, Operand> {
    operator: Operator,
    operand: Operand,
}

#[derive(Eq)]
pub struct Index<Target, Value> {
    target: Target,
    indexes: Vec<Value>,
}

#[derive(Eq)]
pub struct Invoke<Target, Argument> {
    target: Target,
    arguments: Vec<Argument>,
}

#[derive(Eq)]
pub struct Construct<Target, Field> {
    target: Target,
    fields: Vec<Field>,
}

#[derive(Eq)]
pub struct Conditional<Condition, Then, Alternate> {
    condition: Condition,
    then: Then,
    alternate: Option<Alternate>,
}

#[derive(Eq)]
pub struct Repeat<Condition, Body> {
    condition: Option<Condition>,
    body: Body,
}

#[derive(Eq)]
pub struct Iterate<Clause, Body> {
    clause: Clause,
    body: Body,
}

#[derive(Eq)]
pub struct Label<Value, Element> {
    label: Value,
    element: Element,
}

#[derive(Eq)]
pub struct Access<Object, Member> {
    object: Object,
    member: Member,
}

#[derive(Eq)]
pub struct Assign<Target, Value> {
    target: Target,
    value: Value,
}

impl<Item> Group<Item> {
    #[inline]
    pub fn new(items: Vec<Item>) -> Self {
        Group { items }
    }
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Item> {
        self.items.get_mut(index)
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<Item> Sequence<Item> {
    #[inline]
    pub fn new(items: Vec<Item>) -> Self {
        Sequence { items }
    }
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Item> {
        self.items.get_mut(index)
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<Item> Collection<Item> {
    #[inline]
    pub fn new(items: Vec<Item>) -> Self {
        Collection { items }
    }
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Item> {
        self.items.get_mut(index)
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<Item> Series<Item> {
    #[inline]
    pub fn new(items: Vec<Item>) -> Self {
        Series { items }
    }
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Item> {
        self.items.get_mut(index)
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<Item> Bundle<Item> {
    #[inline]
    pub fn new(items: Vec<Item>) -> Self {
        Bundle { items }
    }
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Item> {
        self.items.get_mut(index)
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<Item> Scope<Item> {
    #[inline]
    pub fn new(items: Vec<Item>) -> Self {
        Scope { items }
    }
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Item> {
        self.items.get_mut(index)
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<Left, Operator, Right> Binary<Left, Operator, Right> {
    pub fn new(left: Left, operator: Operator, right: Right) -> Self {
        Binary {
            left,
            operator,
            right,
        }
    }
    #[inline]
    pub fn get_left(&self) -> &Left {
        &self.left
    }
    #[inline]
    pub fn get_right(&self) -> &Right {
        &self.right
    }
    #[inline]
    pub fn get_operator(&self) -> &Operator {
        &self.operator
    }
}

impl<Operator, Operand> Unary<Operator, Operand> {
    #[inline]
    pub fn new(operator: Operator, operand: Operand) -> Self {
        Unary { operator, operand }
    }
    #[inline]
    pub fn get_operand(&self) -> &Operand {
        &self.operand
    }
    #[inline]
    pub fn get_operator(&self) -> &Operator {
        &self.operator
    }
}

impl<Target, Value> Index<Target, Value> {
    #[inline]
    pub fn new(target: Target, indexes: Vec<Value>) -> Self {
        Index { target, indexes }
    }
    #[inline]
    pub fn get_target(&self) -> &Target {
        &self.target
    }
    #[inline]
    pub fn get_indexes(&self) -> &Vec<Value> {
        &self.indexes
    }
    #[inline]
    pub fn get_index(&self, index: usize) -> Option<&Value> {
        self.indexes.get(index)
    }
}

impl<Target, Argument> Invoke<Target, Argument> {
    #[inline]
    pub fn new(target: Target, arguments: Vec<Argument>) -> Self {
        Invoke { target, arguments }
    }
    #[inline]
    pub fn get_target(&self) -> &Target {
        &self.target
    }
    #[inline]
    pub fn get_arguments(&self) -> &Vec<Argument> {
        &self.arguments
    }
    #[inline]
    pub fn get_argument(&self, index: usize) -> Option<&Argument> {
        self.arguments.get(index)
    }
}

impl<Target, Field> Construct<Target, Field> {
    #[inline]
    pub fn new(target: Target, fields: Vec<Field>) -> Self {
        Construct { target, fields }
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
}

impl<Condition, Then, Alternate> Conditional<Condition, Then, Alternate> {
    #[inline]
    pub fn new(condition: Condition, then: Then, alternate: Option<Alternate>) -> Self {
        Conditional {
            condition,
            then,
            alternate,
        }
    }
    #[inline]
    pub fn get_condition(&self) -> &Condition {
        &self.condition
    }
    #[inline]
    pub fn get_then(&self) -> &Then {
        &self.then
    }
    #[inline]
    pub fn get_alternate(&self) -> Option<&Alternate> {
        self.alternate.as_ref()
    }
}

impl<Condition, Body> Repeat<Condition, Body> {
    #[inline]
    pub fn new(condition: Option<Condition>, body: Body) -> Self {
        Repeat { condition, body }
    }
    #[inline]
    pub fn get_condition(&self) -> Option<&Condition> {
        self.condition.as_ref()
    }
    #[inline]
    pub fn get_body(&self) -> &Body {
        &self.body
    }
}

impl<Clause, Body> Iterate<Clause, Body> {
    #[inline]
    pub fn new(clause: Clause, body: Body) -> Self {
        Iterate { clause, body }
    }
    #[inline]
    pub fn get_clause(&self) -> &Clause {
        &self.clause
    }
    #[inline]
    pub fn get_body(&self) -> &Body {
        &self.body
    }
}

impl<Value, Element> Label<Value, Element> {
    #[inline]
    pub fn new(label: Value, element: Element) -> Self {
        Label { label, element }
    }
    #[inline]
    pub fn get_label(&self) -> &Value {
        &self.label
    }
    #[inline]
    pub fn get_element(&self) -> &Element {
        &self.element
    }
}

impl<Object, Member> Access<Object, Member> {
    #[inline]
    pub fn new(object: Object, member: Member) -> Self {
        Access { object, member }
    }
    #[inline]
    pub fn get_object(&self) -> &Object {
        &self.object
    }
    #[inline]
    pub fn get_member(&self) -> &Member {
        &self.member
    }
}

impl<Target, Value> Assign<Target, Value> {
    #[inline]
    pub fn new(target: Target, value: Value) -> Self {
        Assign { target, value }
    }
    #[inline]
    pub fn get_target(&self) -> &Target {
        &self.target
    }
    #[inline]
    pub fn get_value(&self) -> &Value {
        &self.value
    }
}

impl<Item: Hash> Hash for Group<Item> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.items.hash(state);
    }
}

impl<Item: Hash> Hash for Sequence<Item> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.items.hash(state);
    }
}

impl<Item: Hash> Hash for Collection<Item> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.items.hash(state);
    }
}

impl<Item: Hash> Hash for Series<Item> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.items.hash(state);
    }
}

impl<Item: Hash> Hash for Bundle<Item> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.items.hash(state);
    }
}

impl<Item: Hash> Hash for Scope<Item> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.items.hash(state);
    }
}

impl<Left: Hash, Operator: Hash, Right: Hash> Hash for Binary<Left, Operator, Right> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_left().hash(state);
        self.get_operator().hash(state);
        self.get_right().hash(state);
    }
}

impl<Operator: Hash, Operand: Hash> Hash for Unary<Operator, Operand> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_operator().hash(state);
        self.get_operand().hash(state);
    }
}

impl<Target: Hash, Value: Hash> Hash for Index<Target, Value> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_target().hash(state);
        self.get_indexes().hash(state);
    }
}

impl<Target: Hash, Argument: Hash> Hash for Invoke<Target, Argument> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_target().hash(state);
        self.get_arguments().hash(state);
    }
}

impl<Target: Hash, Field: Hash> Hash for Construct<Target, Field> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_target().hash(state);
        self.get_fields().hash(state);
    }
}

impl<Condition: Hash, Then: Hash, Alternate: Hash> Hash
    for Conditional<Condition, Then, Alternate>
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_condition().hash(state);
        self.get_then().hash(state);
        self.get_alternate().hash(state);
    }
}

impl<Condition: Hash, Body: Hash> Hash for Repeat<Condition, Body> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_condition().hash(state);
        self.get_body().hash(state);
    }
}

impl<Clause: Hash, Body: Hash> Hash for Iterate<Clause, Body> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_clause().hash(state);
        self.get_body().hash(state);
    }
}

impl<Value: Hash, Element: Hash> Hash for Label<Value, Element> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_label().hash(state);
        self.get_element().hash(state);
    }
}

impl<Object: Hash, Member: Hash> Hash for Access<Object, Member> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_object().hash(state);
        self.get_member().hash(state);
    }
}

impl<Target: Hash, Value: Hash> Hash for Assign<Target, Value> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_target().hash(state);
        self.get_value().hash(state);
    }
}

impl<Item: PartialEq> PartialEq for Group<Item> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<Item: PartialEq> PartialEq for Sequence<Item> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<Item: PartialEq> PartialEq for Collection<Item> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<Item: PartialEq> PartialEq for Series<Item> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<Item: PartialEq> PartialEq for Bundle<Item> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<Item: PartialEq> PartialEq for Scope<Item> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<Left: PartialEq, Operator: PartialEq, Right: PartialEq> PartialEq
    for Binary<Left, Operator, Right>
{
    fn eq(&self, other: &Self) -> bool {
        self.get_left() == other.get_left()
            && self.get_operator() == other.get_operator()
            && self.get_right() == other.get_right()
    }
}

impl<Operator: PartialEq, Operand: PartialEq> PartialEq for Unary<Operator, Operand> {
    fn eq(&self, other: &Self) -> bool {
        self.get_operator() == other.get_operator() && self.get_operand() == other.get_operand()
    }
}

impl<Target: PartialEq, Value: PartialEq> PartialEq for Index<Target, Value> {
    fn eq(&self, other: &Self) -> bool {
        self.get_target() == other.get_target() && self.get_indexes() == other.get_indexes()
    }
}

impl<Target: PartialEq, Argument: PartialEq> PartialEq for Invoke<Target, Argument> {
    fn eq(&self, other: &Self) -> bool {
        self.get_target() == other.get_target() && self.get_arguments() == other.get_arguments()
    }
}

impl<Target: PartialEq, Field: PartialEq> PartialEq for Construct<Target, Field> {
    fn eq(&self, other: &Self) -> bool {
        self.get_target() == other.get_target() && self.get_fields() == other.get_fields()
    }
}

impl<Condition: PartialEq, Then: PartialEq, Alternate: PartialEq> PartialEq
    for Conditional<Condition, Then, Alternate>
{
    fn eq(&self, other: &Self) -> bool {
        self.get_condition() == other.get_condition()
            && self.get_then() == other.get_then()
            && self.get_alternate() == other.get_alternate()
    }
}

impl<Condition: PartialEq, Body: PartialEq> PartialEq for Repeat<Condition, Body> {
    fn eq(&self, other: &Self) -> bool {
        self.get_condition() == other.get_condition() && self.get_body() == other.get_body()
    }
}

impl<Clause: PartialEq, Body: PartialEq> PartialEq for Iterate<Clause, Body> {
    fn eq(&self, other: &Self) -> bool {
        self.get_clause() == other.get_clause() && self.get_body() == other.get_body()
    }
}

impl<Value: PartialEq, Element: PartialEq> PartialEq for Label<Value, Element> {
    fn eq(&self, other: &Self) -> bool {
        self.get_label() == other.get_label() && self.get_element() == other.get_element()
    }
}

impl<Object: PartialEq, Member: PartialEq> PartialEq for Access<Object, Member> {
    fn eq(&self, other: &Self) -> bool {
        self.get_object() == other.get_object() && self.get_member() == other.get_member()
    }
}

impl<Target: PartialEq, Value: PartialEq> PartialEq for Assign<Target, Value> {
    fn eq(&self, other: &Self) -> bool {
        self.get_target() == other.get_target() && self.get_value() == other.get_value()
    }
}

impl<Item: Clone> Clone for Group<Item> {
    fn clone(&self) -> Self {
        Group::new(self.items.clone())
    }
}

impl<Item: Clone> Clone for Sequence<Item> {
    fn clone(&self) -> Self {
        Sequence::new(self.items.clone())
    }
}

impl<Item: Clone> Clone for Collection<Item> {
    fn clone(&self) -> Self {
        Collection::new(self.items.clone())
    }
}

impl<Item: Clone> Clone for Series<Item> {
    fn clone(&self) -> Self {
        Series::new(self.items.clone())
    }
}

impl<Item: Clone> Clone for Bundle<Item> {
    fn clone(&self) -> Self {
        Bundle::new(self.items.clone())
    }
}

impl<Item: Clone> Clone for Scope<Item> {
    fn clone(&self) -> Self {
        Scope::new(self.items.clone())
    }
}

impl<Left: Clone, Operator: Clone, Right: Clone> Clone for Binary<Left, Operator, Right> {
    fn clone(&self) -> Self {
        Binary::new(
            self.get_left().clone(),
            self.get_operator().clone(),
            self.get_right().clone(),
        )
    }
}

impl<Operator: Clone, Operand: Clone> Clone for Unary<Operator, Operand> {
    fn clone(&self) -> Self {
        Unary::new(self.get_operator().clone(), self.get_operand().clone())
    }
}

impl<Target: Clone, Value: Clone> Clone for Index<Target, Value> {
    fn clone(&self) -> Self {
        Index::new(self.get_target().clone(), self.get_indexes().clone())
    }
}

impl<Target: Clone, Argument: Clone> Clone for Invoke<Target, Argument> {
    fn clone(&self) -> Self {
        Invoke::new(self.get_target().clone(), self.get_arguments().clone())
    }
}

impl<Target: Clone, Field: Clone> Clone for Construct<Target, Field> {
    fn clone(&self) -> Self {
        Construct::new(self.get_target().clone(), self.get_fields().clone())
    }
}

impl<Condition: Clone, Then: Clone, Alternate: Clone> Clone
    for Conditional<Condition, Then, Alternate>
{
    fn clone(&self) -> Self {
        Conditional::new(
            self.get_condition().clone(),
            self.get_then().clone(),
            self.get_alternate().cloned(),
        )
    }
}

impl<Condition: Clone, Body: Clone> Clone for Repeat<Condition, Body> {
    fn clone(&self) -> Self {
        Repeat::new(self.get_condition().cloned(), self.get_body().clone())
    }
}

impl<Clause: Clone, Body: Clone> Clone for Iterate<Clause, Body> {
    fn clone(&self) -> Self {
        Iterate::new(self.get_clause().clone(), self.get_body().clone())
    }
}

impl<Value: Clone, Element: Clone> Clone for Label<Value, Element> {
    fn clone(&self) -> Self {
        Label::new(self.get_label().clone(), self.get_element().clone())
    }
}

impl<Object: Clone, Member: Clone> Clone for Access<Object, Member> {
    fn clone(&self) -> Self {
        Access::new(self.get_object().clone(), self.get_member().clone())
    }
}

impl<Target: Clone, Value: Clone> Clone for Assign<Target, Value> {
    fn clone(&self) -> Self {
        Assign::new(self.get_target().clone(), self.get_value().clone())
    }
}