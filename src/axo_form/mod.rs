#![allow(dead_code)]

mod fmt;
mod lexer;
mod parser;
mod pattern;
mod action;
use core::fmt::Debug;
use crate::arc::Arc;
use crate::axo_span::Span;
use crate::Peekable;
use crate::axo_form::action::Action;
use crate::axo_form::pattern::{Pattern, PatternKind};

pub type TransformFunction<Input, Output, Error> =
Arc<dyn Fn(Vec<Form<Input, Output, Error>>, Span) -> Result<Output, Error> + Send + Sync>;
pub type PredicateFunction<Input> = Arc<dyn Fn(&Input) -> bool + Send + Sync>;
pub type ErrorFunction<Error> = Arc<dyn Fn(Span) -> Error>;

#[derive(Clone, Debug)]
pub enum FormKind<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    Empty,
    Raw(Input),
    Single(Output),
    Multiple(Vec<Form<Input, Output, Error>>),
    Error(Error),
}

#[derive(Clone, Debug)]
pub struct Form<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    pub kind: FormKind<Input, Output, Error>,
    pub span: Span,
}

pub trait Former<Input, Output, Error>: Peekable<Input>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    fn expand(form: Form<Input, Output, Error>) -> Vec<Form<Input, Output, Error>>;
    fn catch(forms: Vec<Form<Input, Output, Error>>) -> Option<Form<Input, Output, Error>>;
    fn action(
        action: &Action<Input, Output, Error>,
        formed_items: Vec<Form<Input, Output, Error>>,
        span: Span,
    ) -> Form<Input, Output, Error>;
    fn matches(&mut self, pattern: &Pattern<Input, Output, Error>, offset: usize) -> (bool, usize);
    fn form(&mut self, pattern: Pattern<Input, Output, Error>) -> Form<Input, Output, Error>;
}

impl<Input, Output, Error> Form<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    pub fn new(form: FormKind<Input, Output, Error>, span: Span) -> Self {
        Self { kind: form, span }
    }
}

impl<Matcher, Input, Output, Error> Former<Input, Output, Error> for Matcher
where
    Matcher: Peekable<Input>,
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    fn expand(form: Form<Input, Output, Error>) -> Vec<Form<Input, Output, Error>> {
        match form.kind {
            FormKind::Empty => vec![],
            FormKind::Raw(_) | FormKind::Single(_) | FormKind::Error(_) => vec![form],
            FormKind::Multiple(items) => items,
        }
    }

    fn catch(forms: Vec<Form<Input, Output, Error>>) -> Option<Form<Input, Output, Error>> {
        for form in forms {
            match form.kind.clone() {
                FormKind::Multiple(forms) => {
                    if let Some(error) = Self::catch(forms) {
                        return Some(error);
                    } else {
                        continue;
                    }
                }
                FormKind::Error(_) => {
                    println!("error");
                    
                    return Some(form);
                }
                _ => continue,
            }
        }

        None
    }

    fn action(
        action: &Action<Input, Output, Error>,
        items: Vec<Form<Input, Output, Error>>,
        span: Span,
    ) -> Form<Input, Output, Error> {
        if let Some(err) = Self::catch(items.clone()) {
            return err;
        }

        let result = match action {
            Action::Transform(transform) => match transform(items, span.clone()) {
                Ok(token) => Form::new(FormKind::Single(token), span),
                Err(_) => Form::new(FormKind::Empty, span),
            },

            Action::Ignore => Form::new(FormKind::Empty, span),

            Action::Error(function) => Form::new(FormKind::Error(function(span.clone())), span),

            Action::Conditional { found, missing } => {
                if !items.is_empty() {
                    Self::action(found, items, span)
                } else {
                    Self::action(missing, Vec::new(), span)
                }
            }
        };

        result
    }

    fn matches(&mut self, pattern: &Pattern<Input, Output, Error>, offset: usize) -> (bool, usize) {
        match &pattern.kind {
            PatternKind::Exact(expect) => {
                if let Some(peek) = self.peek_ahead(offset) {
                    let matches = peek == expect;
                    (matches, if matches { offset + 1 } else { offset })
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
                let mut current_offset = offset;

                for pattern in sequence {
                    let (matches, pattern_offset) = self.matches(pattern, current_offset);
                    
                    if matches {
                        current_offset = pattern_offset;
                    } else {
                        return (false, offset);
                    }
                }

                (true, current_offset)
            }

            PatternKind::Repeat { pattern, minimum, maximum } => {
                let mut current_offset = offset;
                let mut count = 0;
                let mut last_successful_offset = offset;

                loop {
                    if self.peek_ahead(current_offset).is_none() {
                        break;
                    }

                    let (matches, new_offset) = self.matches(pattern, current_offset);

                    if matches && new_offset > current_offset {
                        count += 1;
                        current_offset = new_offset;
                        last_successful_offset = new_offset;

                        if let Some(max) = maximum {
                            if count >= *max {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }

                let result = count >= *minimum;
                (result, if result { last_successful_offset } else { offset })
            }

            PatternKind::Optional(pattern) => {
                let (matches, new_offset) = self.matches(pattern, offset);
                (true, if matches { new_offset } else { offset })
            }

            PatternKind::Predicate(function) => {
                if let Some(peek) = self.peek_ahead(offset) {
                    let result = function(&peek);
                    (result, if result { offset + 1 } else { offset })
                } else {
                    (false, offset)
                }
            }

            PatternKind::Negate(pattern) => {
                if self.peek_ahead(offset).is_none() {
                    return (false, offset);
                }

                let (matches, _) = self.matches(pattern, offset);
                let result = !matches;
                (result, if result { offset + 1 } else { offset })
            }

            PatternKind::Anything => {
                if self.peek_ahead(offset).is_some() {
                    (true, offset + 1)
                } else {
                    (false, offset)
                }
            }

            PatternKind::Required { pattern, .. } => {
                let (matches, new_offset) = self.matches(pattern, offset);
                if matches {
                    (true, new_offset)
                } else {
                    (true, offset)
                }
            }
        }
    }

    fn form(&mut self, pattern: Pattern<Input, Output, Error>) -> Form<Input, Output, Error> {
        let start = self.position();
        let (matches, _) = self.matches(&pattern, 0);

        if !matches {
            return Form::new(FormKind::Empty, Span::point(start.clone()));
        }

        let form = match &pattern.kind {
            PatternKind::Exact(input) => {
                if let Some(actual) = self.next() {
                    if actual == *input {
                        let end = self.position();
                        Form::new(FormKind::Raw(input.clone()), Span::new(start.clone(), end))
                    } else {
                        Form::new(FormKind::Empty, Span::point(start.clone()))
                    }
                } else {
                    Form::new(FormKind::Empty, Span::point(start.clone()))
                }
            }

            PatternKind::Alternative(patterns) => {
                for subpattern in patterns {
                    if let (true, _) = self.matches(subpattern, 0) {
                        return self.form(subpattern.clone());
                    }
                }
                Form::new(FormKind::Empty, Span::point(self.position()))
            }

            PatternKind::Sequence(sequence) => {
                let mut formed_sequence = Vec::new();

                for subpattern in sequence {
                    let form = self.form(subpattern.clone());
                    formed_sequence.push(form);
                }

                Form::new(
                    FormKind::Multiple(formed_sequence),
                    Span::new(start.clone(), self.position()),
                )
            }

            PatternKind::Repeat { pattern: subpattern, maximum, .. } => {
                let mut formed_repeat = Vec::new();

                loop {
                    if self.peek().is_none() {
                        break;
                    }

                    let current_position = self.position();
                    let (matches, _) = self.matches(subpattern, 0);

                    if matches {
                        let form = self.form(*subpattern.clone());

                        if self.position() == current_position {
                            break;
                        }

                        formed_repeat.push(form);

                        if let Some(max) = maximum {
                            if formed_repeat.len() >= *max {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }

                Form::new(
                    FormKind::Multiple(formed_repeat),
                    Span::new(start.clone(), self.position()),
                )
            }

            PatternKind::Optional(subpattern) => {
                if let (true, _) = self.matches(subpattern, 0) {
                    self.form(*subpattern.clone())
                } else {
                    Form::new(FormKind::Empty, Span::point(start.clone()))
                }
            }

            PatternKind::Predicate(predicate) => {
                if let Some(input) = self.peek() {
                    if predicate(&input) {
                        let character = self.next().unwrap();
                        Form::new(
                            FormKind::Raw(character),
                            Span::new(start.clone(), self.position()),
                        )
                    } else {
                        Form::new(FormKind::Empty, Span::point(start.clone()))
                    }
                } else {
                    Form::new(FormKind::Empty, Span::point(start.clone()))
                }
            }

            PatternKind::Negate(subpattern) => {
                if let (false, _) = self.matches(subpattern, 0) {
                    if let Some(input) = self.next() {
                        Form::new(
                            FormKind::Raw(input),
                            Span::new(start.clone(), self.position()),
                        )
                    } else {
                        Form::new(FormKind::Empty, Span::point(self.position()))
                    }
                } else {
                    Form::new(FormKind::Empty, Span::point(start.clone()))
                }
            }

            PatternKind::Anything => {
                if let Some(input) = self.next() {
                    Form::new(
                        FormKind::Raw(input),
                        Span::new(start.clone(), self.position()),
                    )
                } else {
                    Form::new(FormKind::Empty, Span::point(self.position()))
                }
            }

            PatternKind::Required { pattern: subpattern, action } => {
                let current_position = self.position();
                let (matches, _) = self.matches(subpattern, 0);

                if matches {
                    self.form(*subpattern.clone())
                } else {
                    let span = Span::new(current_position, self.position());
                    Self::action(action, Vec::new(), span)
                }
            }
        };

        let end = self.position();
        let span = Span::new(start, end);

        match &pattern.action {
            Some(action) => {
                let items = Self::expand(form.clone());
                Self::action(action, items, span.clone())
            }
            None => form,
        }
    }
}