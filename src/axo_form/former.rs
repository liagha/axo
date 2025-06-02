use std::hash::Hash;
use {
    super::{
        action::Action,
        pattern::{Pattern, PatternKind},
    },

    crate::{
        Peekable,
        format::Debug,
        axo_span::Span,
        axo_parser::{Item, ItemKind},
    },
};
use crate::compiler::{Marked};

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum FormKind<Input, Output, Error>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Error: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    Empty,
    Input(Input),
    Output(Output),
    Multiple(Vec<Form<Input, Output, Error>>),
    Error(Error),
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Form<Input, Output, Error>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Error: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub kind: FormKind<Input, Output, Error>,
    pub span: Span,
}

impl<Input, Output, Error> Form<Input, Output, Error>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Error: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub fn new(form: FormKind<Input, Output, Error>, span: Span) -> Self {
        Self { kind: form, span, }
    }

    fn catch(&self) -> Option<Form<Input, Output, Error>> {
        match self.kind.clone() {
            FormKind::Multiple(forms) => {
                for form in forms {
                    Self::catch(&form)?;
                }
            }

            FormKind::Error(_) => {
                return Some(self.clone());
            }

            _ => {},
        }

        None
    }

    pub fn unwrap(&self) -> Vec<Form<Input, Output, Error>> {
        match self.kind.clone() {
            FormKind::Multiple(forms) => forms,
            _ => vec![self.clone()],
        }
    }

    pub fn unwrap_input(&self) -> Option<Input> {
        match self.kind.clone() {
            FormKind::Input(input) => Some(input.clone()),
            _ => None
        }
    }

    pub fn unwrap_output(&self) -> Option<Output> {
        match self.kind.clone() {
            FormKind::Output(output) => Some(output.clone()),
            _ => None
        }
    }

    pub fn expand(&self) -> Vec<Form<Input, Output, Error>> {
        let mut expanded: Vec<Form<Input, Output, Error>> = Vec::new();

        match self {
            Form { kind: FormKind::Multiple(forms), .. } => {
                expanded.extend(Self::expand_forms(forms.clone()));
            }

            form => {
                expanded.push(form.clone());
            }
        }

        expanded
    }

    pub fn inputs(&self) -> Vec<Input> {
        let mut inputs: Vec<Input> = Vec::new();

        for form in self.unwrap() {
            match form.kind {
                FormKind::Input(input) => {
                    inputs.push(input);
                }
                FormKind::Multiple(sub) => {
                    let sub = Self::expand_inputs(sub);

                    inputs.extend(sub);
                }
                _ => {}
            }
        }

        inputs
    }

    pub fn outputs(&self) -> Vec<Output> {
        let mut outputs: Vec<Output> = Vec::new();

        for form in self.unwrap() {
            match form.kind {
                FormKind::Output(output) => {
                    outputs.push(output);
                }
                FormKind::Multiple(sub) => {
                    let sub = Self::expand_outputs(sub);

                    outputs.extend(sub);
                }
                _ => {}
            }
        }

        outputs
    }

    pub fn expand_forms(forms: Vec<Form<Input, Output, Error>>) -> Vec<Form<Input, Output, Error>> {
        let mut expanded: Vec<Form<Input, Output, Error>> = Vec::new();

        for form in forms {
            match form {
                Form { kind: FormKind::Multiple(sub), .. } => {
                    let sub = Self::expand_forms(sub);

                    expanded.extend(sub);
                }
                form => {
                    expanded.push(form)
                }
            }
        }

        expanded
    }

    pub fn expand_inputs(forms: Vec<Form<Input, Output, Error>>) -> Vec<Input> {
        let mut inputs: Vec<Input> = Vec::new();

        for form in forms {
            match form.kind {
                FormKind::Input(input) => {
                    inputs.push(input);
                }
                FormKind::Multiple(sub) => {
                    let sub = Self::expand_inputs(sub);

                    inputs.extend(sub);
                }
                _ => {}
            }
        }

        inputs
    }

    pub fn expand_outputs(forms: Vec<Form<Input, Output, Error>>) -> Vec<Output> {
        let mut outputs: Vec<Output> = Vec::new();

        for form in forms {
            match form.kind {
                FormKind::Output(output) => {
                    outputs.push(output);
                }
                FormKind::Multiple(sub) => {
                    let sub = Self::expand_outputs(sub);

                    outputs.extend(sub);
                }
                _ => {}
            }
        }

        outputs
    }

    pub fn map_types<NewInput, NewOutput, NewError, F, G, H>(
        self,
        input_mapper: F,
        output_mapper: G,
        error_mapper: H,
    ) -> Form<NewInput, NewOutput, NewError>
    where
        NewInput: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
        NewOutput: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
        NewError: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
        F: Fn(Input) -> NewInput + Clone,
        G: Fn(Output) -> NewOutput + Clone,
        H: Fn(Error) -> NewError + Clone,
    {
        let mapped_kind = match self.kind {
            FormKind::Empty => FormKind::Empty,
            FormKind::Input(input) => FormKind::Input(input_mapper(input)),
            FormKind::Output(output) => FormKind::Output(output_mapper(output)),
            FormKind::Multiple(forms) => {
                let mapped_forms = forms
                    .into_iter()
                    .map(|form| form.map_types(input_mapper.clone(), output_mapper.clone(), error_mapper.clone()))
                    .collect();
                FormKind::Multiple(mapped_forms)
            }
            FormKind::Error(error) => FormKind::Error(error_mapper(error)),
        };

        Form::new(mapped_kind, self.span)
    }
}

pub trait Former<Input, Output, Error>: Peekable<Input> + Marked
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Error: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn action(
        &mut self,
        action: &Action<Input, Output, Error>,
        form: Form<Input, Output, Error>,
        span: Span,
    ) -> Form<Input, Output, Error>;
    fn matches(&mut self, pattern: &Pattern<Input, Output, Error>, offset: usize) -> (bool, usize);
    fn form(&mut self, pattern: Pattern<Input, Output, Error>) -> Form<Input, Output, Error>;
}


impl<Matcher, Input, Output, Error> Former<Input, Output, Error> for Matcher
where
    Matcher: Peekable<Input> + Marked,
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Error: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn action(
        &mut self,
        action: &Action<Input, Output, Error>,
        form: Form<Input, Output, Error>,
        span: Span,
    ) -> Form<Input, Output, Error> {
        if let Some(err) = form.catch() {
            return err;
        }

        let result = match action {
            Action::Map(transform) => {
                let context = &mut self.context_mut();

                match transform(context, form) {
                    Ok(output) => Form::new(FormKind::Output(output), span),
                    Err(_) => Form::new(FormKind::Empty, span),
                }
            },

            Action::Ignore => Form::new(FormKind::Empty, span),

            Action::Error(function) => Form::new(FormKind::Error(function(span.clone())), span),

            Action::Trigger { found, .. } => {
                self.action(found, form, span)
            }
        };

        result
    }

    fn matches(&mut self, pattern: &Pattern<Input, Output, Error>, offset: usize) -> (bool, usize) {
        match pattern.kind.clone() {
            PatternKind::Deferred(factory) => {
                let resolved_pattern = factory();

                self.matches(&resolved_pattern, offset)
            }

            PatternKind::Capture {
                pattern,
                ..
            } => {
                self.matches(&*pattern, offset)
            }

            PatternKind::Guard { predicate, pattern } => {
                if predicate(self) {
                    self.matches(&*pattern, offset)
                } else {
                    (false, offset)
                }
            }

            PatternKind::Literal(expect) => {
                if let Some(peek) = self.peek_ahead(offset) {
                    let matches = *peek == expect;

                    (matches, if matches { offset + 1 } else { offset })
                } else {
                    (false, offset)
                }
            }

            PatternKind::Alternative(patterns) => {
                for subpattern in patterns {
                    let (matches, new_offset) = self.matches(&subpattern, offset);

                    if matches {
                        return (true, new_offset);
                    }
                }

                (false, offset)
            }

            PatternKind::Sequence(sequence) => {
                let mut current = offset;

                for pattern in sequence {
                    let (matches, new) = self.matches(&pattern, current);

                    if matches {
                        current = new;
                    } else {
                        return (false, offset);
                    }
                }

                (true, current)
            }

            PatternKind::Repetition { pattern, minimum, maximum } => {
                let mut current = offset;
                let mut count = 0;

                while self.peek_ahead(current).is_some() {
                    let (matches, new) = self.matches(&*pattern, current);

                    if !matches || new == current {
                        break;
                    }

                    count += 1;
                    current = new;

                    if let Some(max) = maximum {
                        if count >= max {
                            break;
                        }
                    }
                }

                let success = count >= minimum;

                (success, if success { current } else { offset })
            }

            PatternKind::Optional(pattern) => {
                let (matches, new) = self.matches(&*pattern, offset);

                (true, if matches { new } else { offset })
            }

            PatternKind::Condition(function) => {
                if let Some(peek) = self.peek_ahead(offset) {
                    let result = function(&peek);

                    (result, if result { offset + 1 } else { offset })
                } else {
                    (false, offset)
                }
            }

            PatternKind::Negation(pattern) => {
                if self.peek_ahead(offset).is_none() {
                    return (false, offset);
                }

                let (matches, _) = self.matches(&*pattern, offset);

                let result = !matches;

                (result, if result { offset + 1 } else { offset })
            }

            PatternKind::WildCard => {
                if self.peek_ahead(offset).is_some() {
                    (true, offset + 1)
                } else {
                    (false, offset)
                }
            }

            PatternKind::Required { pattern, .. } => {
                let (matches, new) = self.matches(&*pattern, offset);

                if matches {
                    (true, new)
                } else {
                    (true, offset)
                }
            }
        }
    }

    fn form(&mut self, pattern: Pattern<Input, Output, Error>) -> Form<Input, Output, Error> {
        let start = self.position();

        let resolved = match &pattern.kind {
            PatternKind::Deferred(factory) => {
                let resolved = factory();

                let mut form = self.form(resolved);

                if let Some(action) = &pattern.action {
                    let end = self.position();
                    let span = Span::new(start, end);

                    form = self.action(action, form, span);
                }

                return form;
            }

            _ => pattern.clone(),
        };

        let (matches, new) = self.matches(&resolved, 0);

        if !matches && new > 0 {
            return Form::new(FormKind::Empty, Span::point(start.clone()));
        }

        let form = match resolved.kind.clone() {
            PatternKind::Literal(input) => {
                if let Some(actual) = self.next() {
                    if actual == input {
                        let end = self.position();

                        Form::new(FormKind::Input(input.clone()), Span::new(start.clone(), end))
                    } else {
                        Form::new(FormKind::Empty, Span::point(start.clone()))
                    }
                } else {
                    Form::new(FormKind::Empty, Span::point(start.clone()))
                }
            }

            PatternKind::Capture {
                identifier,
                pattern,
            } => {
                let captured = self.form(*pattern.clone());

                let resolver = &mut self.context_mut().resolver;

                let form = captured.clone().map_types(
                    |input| Box::new(input) as Box<dyn crate::compiler::Artifact>,
                    |output| Box::new(output) as Box<dyn crate::compiler::Artifact>,
                    |error| Box::new(error) as Box<dyn crate::compiler::Artifact>,
                );

                let item = Item::new(
                    ItemKind::Formed {
                        identifier,
                        form,
                    },
                    captured.span.clone(),
                );

                println!("everything's fine till here");

                resolver.insert(item);

                println!("inserting causes stackoverflow so this wont be printed");

                Form::new(FormKind::Empty, Span::point(start.clone()))
            }

            PatternKind::Guard { predicate, pattern } => {
                if predicate(self) {
                    self.form(*pattern.clone())
                } else {
                    Form::new(FormKind::Empty, Span::point(start.clone()))
                }
            }

            PatternKind::Sequence(sequence) => {
                let mut formed = Vec::new();

                for subpattern in sequence {
                    let form = self.form(subpattern.clone());

                    formed.push(form);
                }

                let kind = if formed.is_empty() {
                    FormKind::Empty
                } else {
                    FormKind::Multiple(formed)
                };

                let result = Form::new(
                    kind,
                    Span::new(start.clone(), self.position()),
                );

                result
            }

            PatternKind::Alternative(patterns) => {
                for subpattern in patterns {
                    let (matches, offset) = self.matches(&subpattern, 0);

                    if matches && offset != 0 {
                        let result = self.form(subpattern.clone());
                        return result;
                    }
                }

                Form::new(FormKind::Empty, Span::point(self.position()))
            }

            PatternKind::Repetition { pattern: subpattern, maximum, .. } => {
                let mut formed = Vec::new();

                while let Some(_) = self.peek() {
                    let (matches, offset) = self.matches(&*subpattern, 0);

                    if matches {
                        let form = self.form(*subpattern.clone());

                        if offset == 0 {
                            break;
                        }

                        formed.push(form);

                        if let Some(max) = maximum {
                            if formed.len() >= max {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }

                let kind = if formed.is_empty() {
                    FormKind::Empty
                } else {
                    FormKind::Multiple(formed)
                };

                let result = Form::new(
                    kind,
                    Span::new(start.clone(), self.position()),
                );

                result
            }

            PatternKind::Optional(sub) => {
                if let (true, _) = self.matches(&*sub, 0) {
                    self.form(*sub.clone())
                } else {
                    Form::new(FormKind::Empty, Span::point(start.clone()))
                }
            }

            PatternKind::Condition(predicate) => {
                if let Some(input) = self.peek() {
                    if predicate(input) {
                        let input = self.next().unwrap();

                        Form::new(
                            FormKind::Input(input),
                            Span::new(start.clone(), self.position()),
                        )
                    } else {
                        Form::new(FormKind::Empty, Span::point(start.clone()))
                    }
                } else {
                    Form::new(FormKind::Empty, Span::point(start.clone()))
                }
            }

            PatternKind::Negation(sub) => {
                if let (false, _) = self.matches(&*sub, 0) {
                    if let Some(input) = self.next() {
                        Form::new(
                            FormKind::Input(input),
                            Span::new(start.clone(), self.position()),
                        )
                    } else {
                        Form::new(FormKind::Empty, Span::point(self.position()))
                    }
                } else {
                    Form::new(FormKind::Empty, Span::point(start.clone()))
                }
            }

            PatternKind::WildCard => {
                if let Some(input) = self.next() {
                    Form::new(
                        FormKind::Input(input),
                        Span::new(start.clone(), self.position()),
                    )
                } else {
                    Form::new(FormKind::Empty, Span::point(self.position()))
                }
            }

            PatternKind::Required { pattern: subpattern, action } => {
                let (matches, _) = self.matches(&*subpattern, 0);

                if matches {
                    self.form(*subpattern.clone())
                } else {
                    let span = Span::new(start.clone(), self.position());

                    self.action(&action, Form::new(FormKind::Empty, span.clone()), span)
                }
            }

            PatternKind::Deferred(_) => unreachable!("lazy pattern should have been resolved."),
        };

        let end = self.position();
        let span = Span::new(start, end);

        match &pattern.action {
            Some(action) => {
                self.action(action, form, span.clone())
            }

            None => form,
        }
    }
}