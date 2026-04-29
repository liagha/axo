use crate::{
    combinator::{Form, Formable, Outcome},
    data::{Identity, Offset},
    tracker::Peekable,
};

#[derive(Clone)]
pub struct Record<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub forms: Box<[Form<'a, Input, Output, Failure>]>,
    pub inputs: Box<[Input]>,
    pub consumed: Box<[Identity]>,
    pub stack: Box<[Identity]>,
    pub form: Identity,
    pub form_base: Offset,
    pub input_base: Offset,
}

#[derive(Clone)]
pub struct Memo<'a, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub outcome: Outcome,
    pub advance: Offset,
    pub state: Source::State,
    pub record: Option<Box<Record<'a, Input, Output, Failure>>>,
}
