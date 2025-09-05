use inkwell::values::{BasicValueEnum, FunctionValue};

use crate::resolver::analyzer::{Analysis, Instruction};

pub trait Backend<'backend> {
    fn generate(&mut self, analyses: Vec<Analysis<'backend>>);

    fn generate_instruction(&mut self, instruction: Instruction<'backend>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend>;

    fn print(&self);

    fn write_to_file(&self, filename: &str) -> std::io::Result<()>;
}