use {
    crate::{
        data::{Offset, Scale},
        internal::hash::{Hash, Hasher}
    }
};

#[derive(Debug, Eq)]
pub struct Procedural<Body> {
    pub body: Body,
}

#[derive(Debug, Eq)]
pub struct Group<Item> {
    pub items: Vec<Item>,
}

#[derive(Debug, Eq)]
pub struct Sequence<Item> {
    pub items: Vec<Item>,
}

#[derive(Debug, Eq)]
pub struct Collection<Item> {
    pub items: Vec<Item>,
}

#[derive(Debug, Eq)]
pub struct Series<Item> {
    pub items: Vec<Item>,
}

#[derive(Debug, Eq)]
pub struct Bundle<Item> {
    pub items: Vec<Item>,
}

#[derive(Debug, Eq)]
pub struct Block<Item> {
    pub items: Vec<Item>,
}

#[derive(Debug, Eq)]
pub struct Binary<Left, Operator, Right> {
    pub left: Left,
    pub operator: Operator,
    pub right: Right,
}

#[derive(Debug, Eq)]
pub struct Unary<Operator, Operand> {
    pub operator: Operator,
    pub operand: Operand,
}

#[derive(Debug, Eq)]
pub struct Index<Target, Value> {
    pub target: Target,
    pub indexes: Vec<Value>,
}

#[derive(Debug, Eq)]
pub struct Invoke<Target, Argument> {
    pub target: Target,
    pub arguments: Vec<Argument>,
}

#[derive(Debug, Eq)]
pub struct Conditional<Condition, Then, Alternate> {
    pub condition: Condition,
    pub then: Then,
    pub alternate: Option<Alternate>,
}

#[derive(Debug, Eq)]
pub struct While<Condition, Body> {
    pub condition: Option<Condition>,
    pub body: Body,
}

#[derive(Debug, Eq)]
pub struct Cycle<Clause, Body> {
    pub clause: Clause,
    pub body: Body,
}

#[derive(Debug, Eq)]
pub struct Label<Value, Element> {
    pub label: Value,
    pub element: Element,
}

#[derive(Debug, Eq)]
pub struct Access<Target, Member> {
    pub target: Target,
    pub member: Member,
}

#[derive(Debug, Eq)]
pub struct Assign<Target, Value> {
    pub target: Target,
    pub value: Value,
}

impl<Body> Procedural<Body> {
    #[inline]
    pub fn new(body: Body) -> Self {
        Procedural { body }
    }
}

impl<Item> Group<Item> {
    #[inline]
    pub fn new(items: Vec<Item>) -> Self {
        Group { items }
    }
}

impl<Item> Sequence<Item> {
    #[inline]
    pub fn new(items: Vec<Item>) -> Self {
        Sequence { items }
    }
}

impl<Item> Collection<Item> {
    #[inline]
    pub fn new(items: Vec<Item>) -> Self {
        Collection { items }
    }
}

impl<Item> Series<Item> {
    #[inline]
    pub fn new(items: Vec<Item>) -> Self {
        Series { items }
    }
}

impl<Item> Bundle<Item> {
    #[inline]
    pub fn new(items: Vec<Item>) -> Self {
        Bundle { items }
    }
}

impl<Item> Block<Item> {
    #[inline]
    pub fn new(items: Vec<Item>) -> Self {
        Block { items }
    }
}

impl<Left, Operator, Right> Binary<Left, Operator, Right> {
    #[inline]
    pub fn new(left: Left, operator: Operator, right: Right) -> Self {
        Binary {
            left,
            operator,
            right,
        }
    }
}

impl<Operator, Operand> Unary<Operator, Operand> {
    #[inline]
    pub fn new(operator: Operator, operand: Operand) -> Self {
        Unary { operator, operand }
    }
}

impl<Target, Value> Index<Target, Value> {
    #[inline]
    pub fn new(target: Target, indexes: Vec<Value>) -> Self {
        Index { target, indexes }
    }
}

impl<Target, Argument> Invoke<Target, Argument> {
    #[inline]
    pub fn new(target: Target, arguments: Vec<Argument>) -> Self {
        Invoke { target, arguments }
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
}

impl<Condition, Body> While<Condition, Body> {
    #[inline]
    pub fn new(condition: Option<Condition>, body: Body) -> Self {
        While { condition, body }
    }
}

impl<Clause, Body> Cycle<Clause, Body> {
    #[inline]
    pub fn new(clause: Clause, body: Body) -> Self {
        Cycle { clause, body }
    }
}

impl<Value, Element> Label<Value, Element> {
    #[inline]
    pub fn new(label: Value, element: Element) -> Self {
        Label { label, element }
    }
}

impl<Target, Member> Access<Target, Member> {
    #[inline]
    pub fn new(target: Target, member: Member) -> Self {
        Access { target, member }
    }
}

impl<Target, Value> Assign<Target, Value> {
    #[inline]
    pub fn new(target: Target, value: Value) -> Self {
        Assign { target, value }
    }
}

impl<Body: Hash> Hash for Procedural<Body> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.body.hash(state);
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

impl<Item: Hash> Hash for Block<Item> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.items.hash(state);
    }
}

impl<Left: Hash, Operator: Hash, Right: Hash> Hash for Binary<Left, Operator, Right> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.left.hash(state);
        self.operator.hash(state);
        self.right.hash(state);
    }
}

impl<Operator: Hash, Operand: Hash> Hash for Unary<Operator, Operand> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.operator.hash(state);
        self.operand.hash(state);
    }
}

impl<Target: Hash, Value: Hash> Hash for Index<Target, Value> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.indexes.hash(state);
    }
}

impl<Target: Hash, Argument: Hash> Hash for Invoke<Target, Argument> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.arguments.hash(state);
    }
}

impl<Condition: Hash, Then: Hash, Alternate: Hash> Hash
for Conditional<Condition, Then, Alternate>
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.condition.hash(state);
        self.then.hash(state);
        self.alternate.hash(state);
    }
}

impl<Condition: Hash, Body: Hash> Hash for While<Condition, Body> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.condition.hash(state);
        self.body.hash(state);
    }
}

impl<Clause: Hash, Body: Hash> Hash for Cycle<Clause, Body> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.clause.hash(state);
        self.body.hash(state);
    }
}

impl<Value: Hash, Element: Hash> Hash for Label<Value, Element> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.label.hash(state);
        self.element.hash(state);
    }
}

impl<Target: Hash, Member: Hash> Hash for Access<Target, Member> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.member.hash(state);
    }
}

impl<Target: Hash, Value: Hash> Hash for Assign<Target, Value> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.value.hash(state);
    }
}

impl<Body: PartialEq> PartialEq for Procedural<Body> {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body
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

impl<Item: PartialEq> PartialEq for Block<Item> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<Left: PartialEq, Operator: PartialEq, Right: PartialEq> PartialEq
for Binary<Left, Operator, Right>
{
    fn eq(&self, other: &Self) -> bool {
        self.left == other.left
            && self.operator == other.operator
            && self.right == other.right
    }
}

impl<Operator: PartialEq, Operand: PartialEq> PartialEq for Unary<Operator, Operand> {
    fn eq(&self, other: &Self) -> bool {
        self.operator == other.operator && self.operand == other.operand
    }
}

impl<Target: PartialEq, Value: PartialEq> PartialEq for Index<Target, Value> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.indexes == other.indexes
    }
}

impl<Target: PartialEq, Argument: PartialEq> PartialEq for Invoke<Target, Argument> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.arguments == other.arguments
    }
}

impl<Condition: PartialEq, Then: PartialEq, Alternate: PartialEq> PartialEq
for Conditional<Condition, Then, Alternate>
{
    fn eq(&self, other: &Self) -> bool {
        self.condition == other.condition
            && self.then == other.then
            && self.alternate == other.alternate
    }
}

impl<Condition: PartialEq, Body: PartialEq> PartialEq for While<Condition, Body> {
    fn eq(&self, other: &Self) -> bool {
        self.condition == other.condition && self.body == other.body
    }
}

impl<Clause: PartialEq, Body: PartialEq> PartialEq for Cycle<Clause, Body> {
    fn eq(&self, other: &Self) -> bool {
        self.clause == other.clause && self.body == other.body
    }
}

impl<Value: PartialEq, Element: PartialEq> PartialEq for Label<Value, Element> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label && self.element == other.element
    }
}

impl<Target: PartialEq, Member: PartialEq> PartialEq for Access<Target, Member> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.member == other.member
    }
}

impl<Target: PartialEq, Value: PartialEq> PartialEq for Assign<Target, Value> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.value == other.value
    }
}

impl<Body: Clone> Clone for Procedural<Body> {
    fn clone(&self) -> Self {
        Procedural::new(self.body.clone())
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

impl<Item: Clone> Clone for Block<Item> {
    fn clone(&self) -> Self {
        Block::new(self.items.clone())
    }
}

impl<Left: Clone, Operator: Clone, Right: Clone> Clone for Binary<Left, Operator, Right> {
    fn clone(&self) -> Self {
        Binary::new(
            self.left.clone(),
            self.operator.clone(),
            self.right.clone(),
        )
    }
}

impl<Operator: Clone, Operand: Clone> Clone for Unary<Operator, Operand> {
    fn clone(&self) -> Self {
        Unary::new(self.operator.clone(), self.operand.clone())
    }
}

impl<Target: Clone, Value: Clone> Clone for Index<Target, Value> {
    fn clone(&self) -> Self {
        Index::new(self.target.clone(), self.indexes.clone())
    }
}

impl<Target: Clone, Argument: Clone> Clone for Invoke<Target, Argument> {
    fn clone(&self) -> Self {
        Invoke::new(self.target.clone(), self.arguments.clone())
    }
}

impl<Condition: Clone, Then: Clone, Alternate: Clone> Clone
for Conditional<Condition, Then, Alternate>
{
    fn clone(&self) -> Self {
        Conditional::new(
            self.condition.clone(),
            self.then.clone(),
            self.alternate.clone(),
        )
    }
}

impl<Condition: Clone, Body: Clone> Clone for While<Condition, Body> {
    fn clone(&self) -> Self {
        While::new(self.condition.clone(), self.body.clone())
    }
}

impl<Clause: Clone, Body: Clone> Clone for Cycle<Clause, Body> {
    fn clone(&self) -> Self {
        Cycle::new(self.clause.clone(), self.body.clone())
    }
}

impl<Value: Clone, Element: Clone> Clone for Label<Value, Element> {
    fn clone(&self) -> Self {
        Label::new(self.label.clone(), self.element.clone())
    }
}

impl<Target: Clone, Member: Clone> Clone for Access<Target, Member> {
    fn clone(&self) -> Self {
        Access::new(self.target.clone(), self.member.clone())
    }
}

impl<Target: Clone, Value: Clone> Clone for Assign<Target, Value> {
    fn clone(&self) -> Self {
        Assign::new(self.target.clone(), self.value.clone())
    }
}