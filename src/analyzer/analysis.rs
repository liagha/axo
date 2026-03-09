use {
    crate::{
        data::*,
    }
};
use crate::checker::Type;

#[derive(Clone, Debug)]
pub enum Analysis<'analysis> {
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
    Array(Vec<Analysis<'analysis>>),
    Tuple(Vec<Analysis<'analysis>>),

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

    Index(Index<Box<Analysis<'analysis>>, Analysis<'analysis>>),
    Invoke(Invoke<Box<Analysis<'analysis>>, Analysis<'analysis>>),

    Block(Vec<Analysis<'analysis>>),
    Conditional(
        Box<Analysis<'analysis>>,
        Box<Analysis<'analysis>>,
        Box<Analysis<'analysis>>,
    ),
    While(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Return(Option<Box<Analysis<'analysis>>>),
    Break(Option<Box<Analysis<'analysis>>>),
    Continue(Option<Box<Analysis<'analysis>>>),

    Usage(Str<'analysis>),
    Access(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Constructor(Structure<Str<'analysis>, Analysis<'analysis>>),
    Assign(Str<'analysis>, Box<Analysis<'analysis>>),
    Store(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Binding(Binding<Str<'analysis>, Box<Analysis<'analysis>>, Type<'analysis>>),
    Structure(Structure<Str<'analysis>, Analysis<'analysis>>),
    Function(
        Function<
            Str<'analysis>,
            Analysis<'analysis>,
            Box<Analysis<'analysis>>,
            Option<Type<'analysis>>,
        >,
    ),
    Module(Str<'analysis>, Vec<Analysis<'analysis>>),
}

impl<'analysis> Analysis<'analysis> {
    pub fn unit() -> Self {
        Analysis::Tuple(Vec::new())
    }
}
