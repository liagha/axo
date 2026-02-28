use {
    crate::resolver::analyzer::{Analysis, Instruction},
    inkwell::values::{BasicValueEnum, FunctionValue},
};

pub trait Backend<'backend> {
    fn generate(&mut self, analyses: Vec<Analysis<'backend>>);

    fn instruction(
        &mut self,
        instruction: Instruction<'backend>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend>;

    fn print(&self);

    fn write(&self, filename: &str) -> std::io::Result<()>;

    fn take_errors(&mut self) -> Vec<crate::generator::GenerateError<'backend>>;
}
