use {
    crate::{
        internal::hash::{Hash, Hasher}
    }
};

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
pub struct Closure<Member, Body> {
    pub members: Vec<Member>,
    pub body: Body,
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

impl<Member, Body> Closure<Member, Body> {
    #[inline]
    pub fn new(members: Vec<Member>, body: Body) -> Self {
        Closure { members, body }
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

impl<Member: Hash, Body: Hash> Hash for Closure<Member, Body> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.members.hash(state);
        self.body.hash(state);
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

impl<Member: PartialEq, Body: PartialEq> PartialEq for Closure<Member, Body> {
    fn eq(&self, other: &Self) -> bool {
        self.members == other.members && self.body == other.body
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

impl<Member: Clone, Body: Clone> Clone for Closure<Member, Body> {
    fn clone(&self) -> Self {
        Closure::new(self.members.clone(), self.body.clone())
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