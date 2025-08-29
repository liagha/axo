use crate::data;

pub struct Analysis<'analysis> {
    pub instruction: Instruction<'analysis>, 
}

pub enum Instruction<'analysis> {
    Integer(data::Integer),
    Float(data::Float),
    Add(Box<Instruction<'analysis>>, Box<Instruction<'analysis>>),
}