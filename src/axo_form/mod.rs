mod test;
use core::fmt::Debug;
use crate::arc::Arc;
use crate::{Lexer, Peekable, Token};
use crate::axo_span::Span;
pub use test::*;

pub type TransformFn<Input, Output> = Arc<dyn Fn(Vec<Formed<Input, Output>>, Span) -> Result<Output, Input> + Send + Sync>;
pub type PredicateFn<Input> = Arc<dyn Fn(&Input) -> bool + Send + Sync>;

#[derive(Debug, Clone, PartialEq)]
pub enum PatternError {
    OffsetTooFar,
    InvalidPattern,
    SequencePartialMatch,
    NoMatch,
    LookupFailed,
    IgnoreFailed,
    PredicateNotSatisfied,
    TransformFailed,
    InvalidResult,
    MinimumRepeatNotMet(usize, usize),
    TerminateConditionMet,
    NegatePatternMatched,
    UnexpectedCharacter(char, Option<char>),
    UnexpectedEndOfInput,
    Custom(String),
}

#[derive(Clone)]
pub enum Pattern<Input, Output>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone,
{
    Precise(Input),
    OneOf(Vec<Pattern<Input, Output>>),
    Sequence(Vec<Pattern<Input, Output>>),
    Repeat {
        pattern: Box<Pattern<Input, Output>>,
        minimum: usize,
        maximum: Option<usize>,
    },
    Optional(Box<Pattern<Input, Output>>),
    Lookup(Box<Pattern<Input, Output>>),
    Ignore(Box<Pattern<Input, Output>>),
    Predicate(PredicateFn<Input>),
    Transform {
        pattern: Box<Pattern<Input, Output>>,
        transform: TransformFn<Input, Output>,
    },
    Negate(Box<Pattern<Input, Output>>),
    Any,
}

#[derive(Clone, Debug)]
pub enum Form<Input, Output> {
    Raw(Input),
    Single(Output),
    Multiple(Vec<Formed<Input, Output>>),
}

#[derive(Clone, Debug)]
pub struct Formed<Input, Output> {
    form: Form<Input, Output>,
    span: Span,
}

pub trait Former<Input: Clone + Debug + PartialEq, Output: Clone, Error>: Peekable<Input> {
    fn matches(&self, pattern: &Pattern<Input, Output>, offset: usize) -> (bool, usize);
    fn form(
        &mut self,
        pattern: Pattern<Input, Output>,
    ) -> Result<Formed<Input, Output>, Error>;
}

impl<Input, Output> Formed<Input, Output> {
    pub fn new(form: Form<Input, Output>, span: Span) -> Self {
        Self { form, span }
    }
}

impl Former<char, Token, PatternError> for Lexer {
    fn matches(&self, pattern: &Pattern<char, Token>, offset: usize) -> (bool, usize) {
        match pattern {
            Pattern::Precise(expect) => {
                if let Some(c) = self.peek_ahead(offset) {
                    (c == expect, offset + 1)
                } else {
                    (false, offset)
                }
            }
            Pattern::OneOf(patterns) => {
                for pattern in patterns {
                    let (matches, new_offset) = self.matches(pattern, offset);
                    if matches {
                        return (true, new_offset);
                    }
                }
                (false, offset)
            }
            Pattern::Sequence(sequence) => {
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

            Pattern::Repeat { pattern, minimum, maximum } => {
                let mut current_offset = offset;
                let mut count = 0;

                while let Some(_) = self.peek() {
                    let (matches, new_offset) = self.matches(pattern, current_offset);

                    if matches {
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

            Pattern::Optional(pattern) => {
                let (matches, new_offset) = self.matches(pattern, offset);

                (true, if matches { new_offset } else { offset })
            }

            Pattern::Lookup(pattern) => {
                let (matches, _) = self.matches(pattern, offset);
                (matches, offset)
            }
            Pattern::Ignore(pattern) => {
                let (matches, _) = self.matches(pattern, offset);
                (matches, offset)
            }
            Pattern::Predicate(predicate_fn) => {
                if let Some(c) = self.peek_ahead(offset) {
                    let result = predicate_fn(&c);
                    (result, if result { offset + 1 } else { offset })
                } else {
                    (false, offset)
                }
            }
            Pattern::Transform { pattern, .. } => self.matches(pattern, offset),
            Pattern::Negate(pattern) => {
                let (matches, _) = self.matches(pattern, offset);
                (!matches, offset)
            }
            Pattern::Any => {
                if self.peek_ahead(offset).is_some() {
                    (true, offset + 1)
                } else {
                    (false, offset)
                }
            }
        }
    }

    fn form(
        &mut self,
        pattern: Pattern<char, Token>,
    ) -> Result<Formed<char, Token>, PatternError> {
        let (matches, _) = self.matches(&pattern, 0);

        if matches {
            match pattern {
                Pattern::Precise(c) => {
                    self.next();

                    Ok(Formed::new(Form::Raw(c), Span::default()))
                }

                Pattern::OneOf(patterns) => {
                    for pattern in patterns {
                        if self.matches(&pattern, 0).0 {
                            return self.form(pattern)
                        } else {
                            continue;
                        }
                    }

                    Err(PatternError::NoMatch)
                }

                Pattern::Sequence(sequence) => {
                    let mut formed_sequence = Vec::new();

                    for pattern in sequence {
                        let formed = self.form(pattern)?;

                        formed_sequence.push(formed);
                    }

                    Ok(Formed::new(Form::Multiple(formed_sequence), Span::default()))
                }

                Pattern::Repeat {
                    pattern,
                    minimum,
                    maximum
                } => {
                    let mut formed_repeat = Vec::new();

                    loop {
                        if self.matches(&pattern, 0).0 {
                            let formed = self.form(*pattern.clone())?;

                            formed_repeat.push(formed);

                            if let Some(max) = maximum {
                                if formed_repeat.len() >= max {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    }

                    if formed_repeat.len() < minimum {
                        return Err(PatternError::MinimumRepeatNotMet(minimum, formed_repeat.len()));
                    }

                    Ok(Formed::new(Form::Multiple(formed_repeat), Span::default()))
                }

                Pattern::Optional(pattern) => {
                    if self.matches(&pattern, 0).0 {
                        self.form(*pattern)
                    } else {
                        Ok(Formed::new(Form::Multiple(Vec::new()), Span::default()))
                    }
                }

                Pattern::Lookup(pattern) => {
                    if self.matches(&pattern, 0).0 {
                        Ok(Formed::new(Form::Multiple(Vec::new()), Span::default()))
                    } else {
                        Err(PatternError::LookupFailed)
                    }
                }

                Pattern::Ignore(pattern) => {
                    if self.matches(&pattern, 0).0 {
                        self.form(*pattern)?;
                        Ok(Formed::new(Form::Multiple(Vec::new()), Span::default()))
                    } else {
                        Err(PatternError::IgnoreFailed)
                    }
                }

                Pattern::Predicate(predicate) => {
                    if let Some(c) = self.peek() {
                        if predicate(&c) {
                            let character = self.next().unwrap();
                            Ok(Formed::new(Form::Raw(character), Span::default()))
                        } else {
                            Err(PatternError::PredicateNotSatisfied)
                        }
                    } else {
                        Err(PatternError::UnexpectedEndOfInput)
                    }
                }

                Pattern::Transform { pattern, transform } => {
                    let formed = self.form(*pattern)?;

                    match formed.form {
                        Form::Raw(_) | Form::Single(_) => {
                            match transform(vec![formed], Span::default()) {
                                Ok(token) => Ok(Formed::new(Form::Single(token), Span::default())),
                                Err(_) => Err(PatternError::TransformFailed)
                            }
                        },
                        Form::Multiple(items) => {
                            match transform(items, Span::default()) {
                                Ok(token) => Ok(Formed::new(Form::Single(token), Span::default())),
                                Err(_) => Err(PatternError::TransformFailed)
                            }
                        },
                    }
                }

                Pattern::Negate(pattern) => {
                    if !self.matches(&pattern, 0).0 {
                        if let Some(c) = self.next() {
                            Ok(Formed::new(Form::Raw(c), Span::default()))
                        } else {
                            Err(PatternError::UnexpectedEndOfInput)
                        }
                    } else {
                        Err(PatternError::NegatePatternMatched)
                    }
                }

                Pattern::Any => {
                    if let Some(c) = self.next() {
                        Ok(Formed::new(Form::Raw(c), Span::default()))
                    } else {
                        Err(PatternError::UnexpectedEndOfInput)
                    }
                }
            }
        } else {
            Err(PatternError::NoMatch)
        }
    }
}