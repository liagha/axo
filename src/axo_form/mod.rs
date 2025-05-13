mod lexer;
use core::fmt::Debug;
use crate::arc::Arc;
use crate::{Peekable};
use crate::axo_span::Span;

pub type TransformFunction<Input, Output> = Arc<dyn Fn(Vec<Formed<Input, Output>>, Span) -> Result<Output, Input> + Send + Sync>;
pub type PredicateFunction<Input> = Arc<dyn Fn(&Input) -> bool + Send + Sync>;

#[derive(Clone)]
pub enum PatternKind<Input, Output>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone,
{
    Literal(Input),

    Alternative(Vec<Pattern<Input, Output>>),

    Sequence(Vec<Pattern<Input, Output>>),

    Repeat {
        pattern: Box<Pattern<Input, Output>>,
        minimum: usize,
        maximum: Option<usize>,
    },

    Optional(Box<Pattern<Input, Output>>),

    Predicate(PredicateFunction<Input>),

    Negate(Box<Pattern<Input, Output>>),

    Anything,

    _Marker(core::marker::PhantomData<Output>),
}

#[derive(Clone)]
pub enum Action<Input, Output> {
    Transform(TransformFunction<Input, Output>),
    None,
}

#[derive(Clone)]
pub struct Pattern<Input, Output>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone,
{
    kind: PatternKind<Input, Output>,
    action: Action<Input, Output>,
}

impl<Input: Clone + Debug + PartialEq, Output: Clone> Pattern<Input, Output> {
    pub fn literal(value: Input) -> Self {
        Self {
            kind: PatternKind::Literal(value),
            action: Action::None,
        }
    }

    pub fn alternative(patterns: impl Into<Vec<Pattern<Input, Output>>>) -> Self {
        Self {
            kind: PatternKind::Alternative(patterns.into()),
            action: Action::None,
        }
    }

    pub fn sequence(patterns: impl Into<Vec<Pattern<Input, Output>>>) -> Self {
        Self {
            kind: PatternKind::Sequence(patterns.into()),
            action: Action::None,
        }
    }

    pub fn repeat(
        pattern: Box<Pattern<Input, Output>>,
        minimum: usize,
        maximum: Option<usize>,
    ) -> Self {
        Self {
            kind: PatternKind::Repeat {
                pattern,
                minimum,
                maximum,
            },
            action: Action::None,
        }
    }

    pub fn optional(pattern: Box<Pattern<Input, Output>>) -> Self {
        Self {
            kind: PatternKind::Optional(pattern),
            action: Action::None,
        }
    }

    pub fn predicate(predicate: PredicateFunction<Input>) -> Self {
        Self {
            kind: PatternKind::Predicate(predicate),
            action: Action::None,
        }
    }

    pub fn negate(pattern: Box<Pattern<Input, Output>>) -> Self {
        Self {
            kind: PatternKind::Negate(pattern),
            action: Action::None,
        }
    }

    pub fn anything() -> Self {
        Self {
            kind: PatternKind::Anything,
            action: Action::None,
        }
    }

    pub fn transform(
        pattern: Box<Pattern<Input, Output>>,
        transform: TransformFunction<Input, Output>,
    ) -> Self {
        Self {
            kind: PatternKind::Sequence(vec![*pattern]),
            action: Action::Transform(transform),
        }
    }

    pub fn with_action(mut self, action: Action<Input, Output>) -> Self {
        self.action = action;
        self
    }
}

#[derive(Clone, Debug)]
pub enum Form<Input, Output> {
    Empty,
    Raw(Input),
    Single(Output),
    Multiple(Vec<Formed<Input, Output>>),
}

#[derive(Clone, Debug)]
pub struct Formed<Input, Output> {
    pub form: Form<Input, Output>,
    pub span: Span,
}

pub trait Former<Input: Clone + Debug + PartialEq, Output: Clone>: Peekable<Input> {
    fn matches(&self, pattern: &Pattern<Input, Output>, offset: usize) -> (bool, usize);
    fn form(
        &mut self,
        pattern: Pattern<Input, Output>,
    ) -> Formed<Input, Output>;
}

impl<Input, Output> Formed<Input, Output> {
    pub fn new(form: Form<Input, Output>, span: Span) -> Self {
        Self { form, span }
    }
}

impl<Matcher, Input, Output> Former<Input, Output> for Matcher
where
    Matcher: Peekable<Input>,
    Input: Debug + PartialEq + Clone,
    Output: Debug + Clone,
{
    fn matches(&self, pattern: &Pattern<Input, Output>, offset: usize) -> (bool, usize) {
        match &pattern.kind {
            PatternKind::Literal(expect) => {
                if let Some(c) = self.peek_ahead(offset) {
                    (c == expect, offset + 1)
                } else {
                    (false, offset)
                }
            }
            PatternKind::Alternative(patterns) => {
                for pattern in patterns {
                    let (matches, new_offset) = self.matches(pattern, offset);
                    if matches {
                        return (true, new_offset);
                    }
                }
                (false, offset)
            }
            PatternKind::Sequence(sequence) => {
                let mut sequence_offset = offset;

                for pattern in sequence.iter() {
                    let (matches, pattern_offset) = self.matches(pattern, sequence_offset);

                    if matches {
                        sequence_offset = pattern_offset;
                    } else {
                        return (false, offset);
                    }
                }

                (true, sequence_offset)
            }

            PatternKind::Repeat { pattern, minimum, maximum } => {
                let mut current_offset = offset;
                let mut count = 0;

                while let Some(_) = self.peek_ahead(current_offset) {
                    let (matches, new_offset) = self.matches(pattern, current_offset);

                    if matches {
                        if current_offset == new_offset {
                            break;
                        }

                        count += 1;
                        current_offset = new_offset;

                        if let Some(max) = maximum {
                            if count >= *max {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }

                (count >= *minimum, current_offset)
            }

            PatternKind::Optional(pattern) => {
                let (matches, new_offset) = self.matches(pattern, offset);

                (true, if matches { new_offset } else { offset })
            }

            PatternKind::Predicate(predicate_fn) => {
                if let Some(c) = self.peek_ahead(offset) {
                    let result = predicate_fn(&c);
                    (result, if result { offset + 1 } else { offset })
                } else {
                    (false, offset)
                }
            }

            PatternKind::Negate(pattern) => {
                let (matches, _) = self.matches(pattern, offset);

                (!matches, if !matches && self.peek_ahead(offset).is_some() { offset + 1 } else { offset })
            }

            PatternKind::Anything => {
                if self.peek_ahead(offset).is_some() {
                    (true, offset + 1)
                } else {
                    (false, offset)
                }
            }

            PatternKind::_Marker(_) => {
                (false, offset)
            }
        }
    }

    fn form(
        &mut self,
        pattern: Pattern<Input, Output>,
    ) -> Formed<Input, Output> {
        let (matches, _) = self.matches(&pattern, 0);

        if matches {
            let start = self.position();
            let formed = match &pattern.kind {
                PatternKind::Literal(c) => {
                    self.next();
                    Formed::new(Form::Raw(c.clone()), Span::new(start.clone(), self.position()))
                }

                PatternKind::Alternative(patterns) => {
                    for subpattern in patterns {
                        if self.matches(subpattern, 0).0 {
                            return self.form(subpattern.clone());
                        }
                    }
                    Formed::new(Form::Empty, Span::point(self.position()))
                }

                PatternKind::Sequence(sequence) => {
                    let mut formed_sequence = Vec::new();
                    for subpattern in sequence {
                        let formed = self.form(subpattern.clone());
                        formed_sequence.push(formed);
                    }
                    Formed::new(Form::Multiple(formed_sequence), Span::new(start.clone(), self.position()))
                }

                PatternKind::Repeat { pattern: subpattern, maximum, .. } => {
                    let mut formed_repeat = Vec::new();
                    loop {
                        if self.matches(subpattern, 0).0 {
                            let formed = self.form((**subpattern).clone());
                            formed_repeat.push(formed);

                            if let Some(max) = maximum {
                                if formed_repeat.len() >= *max {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    Formed::new(Form::Multiple(formed_repeat), Span::new(start.clone(), self.position()))
                }

                PatternKind::Optional(subpattern) => {
                    if self.matches(subpattern, 0).0 {
                        self.form((**subpattern).clone())
                    } else {
                        Formed::new(Form::Multiple(Vec::new()), Span::new(start.clone(), self.position()))
                    }
                }

                PatternKind::Predicate(predicate) => {
                    if let Some(c) = self.peek() {
                        if predicate(&c) {
                            let character = self.next().unwrap();
                            Formed::new(Form::Raw(character), Span::new(start.clone(), self.position()))
                        } else {
                            Formed::new(Form::Empty, Span::point(start.clone()))
                        }
                    } else {
                        Formed::new(Form::Empty, Span::point(start.clone()))
                    }
                }

                PatternKind::Negate(subpattern) => {
                    if !self.matches(subpattern, 0).0 {
                        if let Some(c) = self.next() {
                            Formed::new(Form::Raw(c), Span::new(start.clone(), self.position()))
                        } else {
                            Formed::new(Form::Empty, Span::point(self.position()))
                        }
                    } else {
                        Formed::new(Form::Empty, Span::point(start.clone()))
                    }
                }

                PatternKind::Anything => {
                    if let Some(c) = self.next() {
                        Formed::new(Form::Raw(c), Span::new(start.clone(), self.position()))
                    } else {
                        Formed::new(Form::Empty, Span::point(self.position()))
                    }
                }

                PatternKind::_Marker(_) => {
                    Formed::new(Form::Empty, Span::point(self.position()))
                }
            };

            match pattern.action {
                Action::Transform(transform) => {
                    let end = self.position();
                    let span = Span::new(start, end);

                    match formed.form {
                        Form::Empty => {
                            match transform(vec![], span.clone()) {
                                Ok(token) => Formed::new(Form::Single(token), span),
                                Err(_) => Formed::new(Form::Empty, Span::point(self.position()))
                            }
                        }
                        Form::Raw(_) | Form::Single(_) => {
                            match transform(vec![formed], span.clone()) {
                                Ok(token) => Formed::new(Form::Single(token), span),
                                Err(_) => Formed::new(Form::Empty, Span::point(self.position()))
                            }
                        }
                        Form::Multiple(items) => {
                            match transform(items, span.clone()) {
                                Ok(token) => Formed::new(Form::Single(token), span),
                                Err(_) => Formed::new(Form::Empty, Span::point(self.position()))
                            }
                        }
                    }
                }
                Action::None => formed
            }
        } else {
            Formed::new(Form::Empty, Span::point(self.position()))
        }
    }
}