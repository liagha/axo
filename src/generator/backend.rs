use {
    inkwell::{
        values::{BasicValueEnum}
    },
    crate::{
        analyzer::{Analysis},
        generator::GenerateError,
    }
};

pub trait Backend<'backend> {
    fn generate(&mut self, analyses: Vec<Analysis<'backend>>);

    fn analysis(
        &mut self,
        instruction: Analysis<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>>;
}
