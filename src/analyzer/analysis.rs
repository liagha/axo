use crate::data::{Boolean, Char, Float, Integer, Scale, Str};
use crate::checker::TypeKind;
use crate::data::schema::*;

#[derive(Clone, Debug)]
pub struct Analysis<'analysis> {
    pub instruction: Instruction<'analysis>,
}

impl<'analysis> Analysis<'analysis> {
    pub fn new(instruction: Instruction<'analysis>) -> Self {
        Analysis { instruction }
    }

    pub fn unit() -> Self {
        Analysis {
            instruction: Instruction::Tuple(Vec::new()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Instruction<'analysis> {
    Integer {
        value: Integer,
        size: Scale,
        signed: Boolean,
    },
    Float {
        value: Float,
        size: Scale,
    },
    Boolean {
        value: Boolean,
    },
    String {
        value: Str<'analysis>,
    },
    Character {
        value: Char,
    },
    Array(Vec<Box<Analysis<'analysis>>>),
    Tuple(Vec<Box<Analysis<'analysis>>>),

    Add(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Subtract(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Multiply(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Divide(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Modulus(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    LogicalAnd(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    LogicalOr(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    LogicalNot(Box<Analysis<'analysis>>),
    LogicalXOr(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    BitwiseAnd(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    BitwiseOr(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    BitwiseNot(Box<Analysis<'analysis>>),
    BitwiseXOr(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    ShiftLeft(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    ShiftRight(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    AddressOf(Box<Analysis<'analysis>>),
    Dereference(Box<Analysis<'analysis>>),

    Equal(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    NotEqual(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Less(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    LessOrEqual(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Greater(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    GreaterOrEqual(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),

    Index(Index<Box<Analysis<'analysis>>, Box<Analysis<'analysis>>>),
    Invoke(Invoke<Box<Analysis<'analysis>>, Box<Analysis<'analysis>>>),

    Block(Vec<Analysis<'analysis>>),
    Conditional(
        Box<Analysis<'analysis>>,
        Box<Analysis<'analysis>>,
        Box<Analysis<'analysis>>,
    ),
    While(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Cycle(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Return(Option<Box<Analysis<'analysis>>>),
    Break(Option<Box<Analysis<'analysis>>>),
    Continue(Option<Box<Analysis<'analysis>>>),

    Usage(Str<'analysis>),
    Access(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Constructor(Structure<Str<'analysis>, Box<Analysis<'analysis>>>),
    Assign(Str<'analysis>, Box<Analysis<'analysis>>),
    Store(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Binding(Binding<Str<'analysis>, Box<Analysis<'analysis>>, TypeKind<'analysis>>),
    Structure(Structure<Str<'analysis>, Box<Analysis<'analysis>>>),
    Enumeration(Structure<Str<'analysis>, Box<Analysis<'analysis>>>),
    Method(
        Method<
            Str<'analysis>,
            Box<Analysis<'analysis>>,
            Box<Analysis<'analysis>>,
            Option<Box<Analysis<'analysis>>>,
        >,
    ),
    Module(Str<'analysis>, Vec<Analysis<'analysis>>),
}

impl<'analysis> Instruction<'analysis> {
    pub fn is_value(&self) -> bool {
        matches!(
            self,
            Instruction::Integer { .. }
                | Instruction::Float { .. }
                | Instruction::Boolean { .. }
                | Instruction::String { .. }
                | Instruction::Character { .. }
        )
    }
}
