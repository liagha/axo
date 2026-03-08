use {
    inkwell::{
        values::{BasicValueEnum}
    },
    crate::{
        analyzer::{Analysis},
    }
};

pub trait Backend<'backend> {
    fn generate(&mut self, analyses: Vec<Analysis<'backend>>);

    fn analysis(
        &mut self,
        instruction: Analysis<'backend>,
    ) -> BasicValueEnum<'backend>;
}
