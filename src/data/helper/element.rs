use crate::internal::{
    cache::{Decode, Encode},
    hash::{Hash, Hasher},
};

#[derive(Debug, Eq)]
pub struct Delimited<Delimiter, Item> {
    pub start: Delimiter,
    pub members: Vec<Item>,
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

impl<Delimiter, Item> Delimited<Delimiter, Item> {
    #[inline]
    pub fn new(
        start: Delimiter,
        items: Vec<Item>,
        separator: Option<Delimiter>,
        end: Delimiter,
    ) -> Self {
        Delimited {
            start,
            members: items,
            separator,
            end,
        }
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
        Index {
            target,
            members: indexes,
        }
    }
}

impl<Target, Argument> Invoke<Target, Argument> {
    #[inline]
    pub fn new(target: Target, arguments: Vec<Argument>) -> Self {
        Invoke {
            target,
            members: arguments,
        }
    }
}

impl<Delimiter: Encode, Item: Encode> Encode for Delimited<Delimiter, Item> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.start.encode(buffer);
        self.members.encode(buffer);
        self.separator.encode(buffer);
        self.end.encode(buffer);
    }
}

impl<'element, Delimiter: Decode<'element>, Item: Decode<'element>> Decode<'element>
    for Delimited<Delimiter, Item>
{
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        Delimited {
            start: Delimiter::decode(buffer, cursor),
            members: Vec::decode(buffer, cursor),
            separator: Option::decode(buffer, cursor),
            end: Delimiter::decode(buffer, cursor),
        }
    }
}

impl<Left: Encode, Operator: Encode, Right: Encode> Encode for Binary<Left, Operator, Right> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.left.encode(buffer);
        self.operator.encode(buffer);
        self.right.encode(buffer);
    }
}

impl<'element, Left: Decode<'element>, Operator: Decode<'element>, Right: Decode<'element>>
    Decode<'element> for Binary<Left, Operator, Right>
{
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        Binary {
            left: Left::decode(buffer, cursor),
            operator: Operator::decode(buffer, cursor),
            right: Right::decode(buffer, cursor),
        }
    }
}

impl<Operator: Encode, Operand: Encode> Encode for Unary<Operator, Operand> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.operator.encode(buffer);
        self.operand.encode(buffer);
    }
}

impl<'element, Operator: Decode<'element>, Operand: Decode<'element>> Decode<'element>
    for Unary<Operator, Operand>
{
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        Unary {
            operator: Operator::decode(buffer, cursor),
            operand: Operand::decode(buffer, cursor),
        }
    }
}

impl<Target: Encode, Value: Encode> Encode for Index<Target, Value> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.target.encode(buffer);
        self.members.encode(buffer);
    }
}

impl<'element, Target: Decode<'element>, Value: Decode<'element>> Decode<'element>
    for Index<Target, Value>
{
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        Index {
            target: Target::decode(buffer, cursor),
            members: Vec::decode(buffer, cursor),
        }
    }
}

impl<Target: Encode, Argument: Encode> Encode for Invoke<Target, Argument> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.target.encode(buffer);
        self.members.encode(buffer);
    }
}

impl<'element, Target: Decode<'element>, Argument: Decode<'element>> Decode<'element>
    for Invoke<Target, Argument>
{
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        Invoke {
            target: Target::decode(buffer, cursor),
            members: Vec::decode(buffer, cursor),
        }
    }
}

impl<Delimiter: Hash, Item: Hash> Hash for Delimited<Delimiter, Item> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.members.hash(state);
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

impl<Delimiter: PartialEq, Item: PartialEq> PartialEq for Delimited<Delimiter, Item> {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
            && self.members == other.members
            && self.separator == other.separator
            && self.end == other.end
    }
}

impl<Left: PartialEq, Operator: PartialEq, Right: PartialEq> PartialEq
    for Binary<Left, Operator, Right>
{
    fn eq(&self, other: &Self) -> bool {
        self.left == other.left && self.operator == other.operator && self.right == other.right
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
            self.members.clone(),
            self.separator.clone(),
            self.end.clone(),
        )
    }
}

impl<Left: Clone, Operator: Clone, Right: Clone> Clone for Binary<Left, Operator, Right> {
    fn clone(&self) -> Self {
        Binary::new(self.left.clone(), self.operator.clone(), self.right.clone())
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
