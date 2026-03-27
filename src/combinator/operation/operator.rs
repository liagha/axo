use crate::{
    combinator::{Formable, Processor},
    data::memory::PhantomData,
};

pub struct Operator<'a, Data, Output, Failure>
where
    Data: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub data: Vec<Data>,
    pub outputs: Vec<Output>,
    pub failures: Vec<Failure>,
    pub phantom: PhantomData<&'a Data>,
}

impl<'a, Data, Output, Failure> Operator<'a, Data, Output, Failure>
where
    Data: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            outputs: Vec::new(),
            failures: Vec::new(),
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn build<'source>(
        &mut self,
        processor: &mut Processor<'a, 'source, Data, Output, Failure>,
    ) {
        let action = processor.action.clone();
        action.action(self, processor);
    }
}
