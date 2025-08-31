use {
    crate::{
        data,
        schema::Binding,
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
    Integer(data::Integer),
    Float(data::Float),
    Boolean(data::Boolean),

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

    Usage(data::Str<'analysis>),
    Assign(data::Str<'analysis>, Box<Analysis<'analysis>>),
    Binding(Binding<data::Str<'analysis>, Box<Analysis<'analysis>>, Box<Analysis<'analysis>>>),
    Module(data::Str<'analysis>, Vec<Analysis<'analysis>>),
}

impl<'analysis> Instruction<'analysis> {
    pub fn is_value(&self) -> bool {
        matches!(
            self,
            Instruction::Integer(_)
            | Instruction::Float(_)
            | Instruction::Boolean(_)
        )
    }
}