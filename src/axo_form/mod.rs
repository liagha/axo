use core::fmt::Debug;

#[derive(Debug, Clone)]
pub enum PatternKind<Input, Output, Name, Error>
where
    Input: Clone + PartialEq + Debug,
    Name: Clone + PartialEq + Debug,
    Error: Clone + Debug,
{
    Precise(Input),

    OneOf(Vec<PatternKind<Input, Output, Name, Error>>),

    Sequence(Vec<PatternKind<Input, Output, Name, Error>>),

    Named {
        name: Name,
        pattern: Box<PatternKind<Input, Output, Name, Error>>,
    },

    Repeat(Box<PatternKind<Input, Output, Name, Error>>, usize, usize),

    Optional(Box<PatternKind<Input, Output, Name, Error>>),

    Continuous {
        until: Box<PatternKind<Input, Output, Name, Error>>,
        inclusive: bool,
    },

    Separated {
        item: Box<PatternKind<Input, Output, Name, Error>>,
        separator: Box<PatternKind<Input, Output, Name, Error>>,
        allow_trailing: bool,
    },

    Ignore(Box<PatternKind<Input, Output, Name, Error>>),

    Predicate(fn(&Input) -> bool),

    Terminate(Input),

    Not(Box<PatternKind<Input, Output, Name, Error>>),

    #[doc(hidden)]
    _Marker(core::marker::PhantomData<(Output, Error)>),
}