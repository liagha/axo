use {
    crate::{analyzer::Analysis, generator::GenerateError},
    inkwell::values::BasicValueEnum,
};

pub trait Backend<'backend> {
    fn generate(&mut self, analyses: Vec<Analysis<'backend>>);

    fn analysis(
        &mut self,
        instruction: Analysis<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>>;
}
