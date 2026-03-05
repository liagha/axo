use crate::data::Str;
use crate::format::Show;
use crate::internal::hash::{Hash, Hasher};

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

impl<'show, Delimiter: Show<'show, Verbosity = u8>, Member: Show<'show, Verbosity = u8>> Show<'show> for Delimited<Delimiter, Member> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => {
                format!(
                    "Delimited({} | {})[{}]({})",
                    self.start.format(verbosity),
                    self.separator.format(verbosity),
                    self.members.format(verbosity),
                    self.end.format(verbosity),
                ).into()
            }

            _ => {
                self.format(verbosity - 1)
            }
        }
    }
}

impl<'show, Left: Show<'show, Verbosity = u8>, Operator: Show<'show, Verbosity = u8>, Right: Show<'show, Verbosity = u8>> Show<'show> for Binary<Left, Operator, Right> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => {
                format!(
                    "Binary({} {} {})",
                    self.left.format(verbosity),
                    self.operator.format(verbosity),
                    self.right.format(verbosity)
                ).into()
            }

            _ => {
                self.format(verbosity - 1)
            }
        }
    }
}

impl<'show, Operator: Show<'show, Verbosity = u8>, Operand: Show<'show, Verbosity = u8>> Show<'show> for Unary<Operator, Operand> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => {
                format!(
                    "Unary({} {})",
                    self.operator.format(verbosity),
                    self.operand.format(verbosity)
                ).into()
            }

            _ => {
                self.format(verbosity - 1)
            }
        }
    }
}

impl<'show, Target: Show<'show, Verbosity = u8>, Member: Show<'show, Verbosity = u8>> Show<'show> for Index<Target, Member> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => {
                format!(
                    "Index({})[{}]",
                    self.target.format(verbosity),
                    self.members.format(verbosity),
                ).into()
            }

            _ => {
                self.format(verbosity - 1)
            }
        }
    }
}

impl<'show, Target: Show<'show, Verbosity = u8>, Member: Show<'show, Verbosity = u8>> Show<'show> for Invoke<Target, Member> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => {
                format!(
                    "Invoke({})[{}]",
                    self.target.format(verbosity),
                    self.members.format(verbosity),
                ).into()
            }

            _ => {
                self.format(verbosity - 1)
            }
        }
    }
}