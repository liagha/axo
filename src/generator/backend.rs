use {
    inkwell::{
        values::{BasicValueEnum, FunctionValue}
    },
    crate::{
        analyzer::{Analysis},
        internal::platform::Error as IOError,
    }
};

pub trait Backend<'backend> {
    fn generate(&mut self, analyses: Vec<Analysis<'backend>>);

    fn analysis(
        &mut self,
        instruction: Analysis<'backend>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend>;

    fn print(&self);

    fn write(&self, filename: &str) -> Result<(), IOError>;

    fn take_errors(&mut self) -> Vec<crate::generator::GenerateError<'backend>>;
}
