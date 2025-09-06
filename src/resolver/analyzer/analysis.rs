use {
    crate::{
        data,
        schema::{
            Assign, Conditional, Cycle,
            Enumeration, Index, Invoke, Method,
            Structure, While, Binding},
    }
};

#[derive(Clone, Debug)]
pub struct Analysis<'analysis> {
    pub instruction: Instruction<'analysis>,
}

impl<'analysis> Analysis<'analysis> {
    pub fn new(instruction: Instruction<'analysis>) -> Self {
        Analysis { instruction }
    }
}

#[derive(Clone, Debug)]
pub enum Instruction<'analysis> {
    // Primitives
    Integer { value: data::Integer, size: data::Scale, signed: data::Boolean },
    Float { value: data::Float, size: data::Scale },
    Boolean { value: data::Boolean },
    Array(Vec<Box<Analysis<'analysis>>>),
    Tuple(Vec<Box<Analysis<'analysis>>>),

    // Operations
    // Arithmetic
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

    // Comparison
    Equal(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    NotEqual(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Less(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    LessOrEqual(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Greater(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    GreaterOrEqual(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),

    // IDK what these are named
    Index(Index<Box<Analysis<'analysis>>, Box<Analysis<'analysis>>>),
    Invoke(Invoke<Box<Analysis<'analysis>>, Box<Analysis<'analysis>>>),

    // Control Flow Related
    Block(Vec<Box<Analysis<'analysis>>>),
    Conditional(Conditional<Box<Analysis<'analysis>>, Box<Analysis<'analysis>>, Box<Analysis<'analysis>>>),
    While(While<Box<Analysis<'analysis>>, Box<Analysis<'analysis>>>),
    Cycle(Cycle<Box<Analysis<'analysis>>, Box<Analysis<'analysis>>>),
    Return(Option<Box<Analysis<'analysis>>>),
    Break(Option<Box<Analysis<'analysis>>>),
    Continue(Option<Box<Analysis<'analysis>>>),

    // Symbols & Stuff
    Usage(data::Str<'analysis>),
    Access(Box<Analysis<'analysis>>, Box<Analysis<'analysis>>),
    Constructor(Structure<data::Str<'analysis>, Box<Analysis<'analysis>>>),
    Assign(Assign<data::Str<'analysis>, Box<Analysis<'analysis>>>),
    Binding(Binding<data::Str<'analysis>, Box<Analysis<'analysis>>, Box<Analysis<'analysis>>>),
    Structure(Structure<data::Str<'analysis>, Box<Analysis<'analysis>>>),
    Enumeration(Enumeration<data::Str<'analysis>, Box<Analysis<'analysis>>>),
    Method(Method<data::Str<'analysis>, Box<Analysis<'analysis>>, Box<Analysis<'analysis>>, Option<Box<Analysis<'analysis>>>>),
    Module(data::Str<'analysis>, Vec<Analysis<'analysis>>),
}

impl<'analysis> Instruction<'analysis> {
    pub fn is_value(&self) -> bool {
        matches!(
            self,
            Instruction::Integer { .. }
            | Instruction::Float { .. }
            | Instruction::Boolean { .. }
        )
    }
}