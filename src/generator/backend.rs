use {
    inkwell::{
        values::{BasicValueEnum, FunctionValue}
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
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend>;
}
