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
pub struct Delimited<Delimiter, Item> {
    pub start: Delimiter,
    pub items: Vec<Item>,
    pub separator: Option<Delimiter>,
    pub end: Delimiter,
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
    pub members: Vec<Value>,
}

#[derive(Debug, Eq)]
pub struct Invoke<Target, Argument> {
    pub target: Target,
    pub members: Vec<Argument>,
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

impl<Body> Procedural<Body> {
    #[inline]
    pub fn new(body: Body) -> Self {
        Procedural { body }
    }
}

impl<Delimiter, Item> Delimited<Delimiter, Item> {
    #[inline]
    pub fn new(start: Delimiter, items: Vec<Item>, separator: Option<Delimiter>, end: Delimiter) -> Self {
        Delimited { start, items, separator, end }
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
        Index { target, members: indexes }
    }
}

impl<Target, Argument> Invoke<Target, Argument> {
    #[inline]
    pub fn new(target: Target, arguments: Vec<Argument>) -> Self {
        Invoke { target, members: arguments }
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

impl<Body: Hash> Hash for Procedural<Body> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.body.hash(state);
    }
}

impl<Delimiter: Hash, Item: Hash> Hash for Delimited<Delimiter, Item> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.items.hash(state);
        self.separator.hash(state);
        self.end.hash(state);
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
        self.members.hash(state);
    }
}

impl<Target: Hash, Argument: Hash> Hash for Invoke<Target, Argument> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.members.hash(state);
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

impl<Body: PartialEq> PartialEq for Procedural<Body> {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body
    }
}

impl<Delimiter: PartialEq, Item: PartialEq> PartialEq for Delimited<Delimiter, Item> {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
            && self.items == other.items
            && self.separator == other.separator
            && self.end == other.end
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
        self.target == other.target && self.members == other.members
    }
}

impl<Target: PartialEq, Argument: PartialEq> PartialEq for Invoke<Target, Argument> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.members == other.members
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

impl<Body: Clone> Clone for Procedural<Body> {
    fn clone(&self) -> Self {
        Procedural::new(self.body.clone())
    }
}

impl<Delimiter: Clone, Item: Clone> Clone for Delimited<Delimiter, Item> {
    fn clone(&self) -> Self {
        Delimited::new(
            self.start.clone(),
            self.items.clone(),
            self.separator.clone(),
            self.end.clone(),
        )
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
        Index::new(self.target.clone(), self.members.clone())
    }
}

impl<Target: Clone, Argument: Clone> Clone for Invoke<Target, Argument> {
    fn clone(&self) -> Self {
        Invoke::new(self.target.clone(), self.members.clone())
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