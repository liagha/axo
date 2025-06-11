use {
    super::{
        action::Action,
        pattern::{Pattern, PatternKind},
    },
    crate::{
        artifact::Artifact,
        axo_form::form::{Form, FormKind},
        axo_parser::{Item, ItemKind},
        axo_span::{Position, Span},
        compiler::Marked,
        format::Debug,
        hash::Hash,
        Peekable,
    },
};

#[derive(Clone, Debug)]
pub struct Draft<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pattern: Pattern<Input, Output, Failure>,
    children: Vec<Draft<Input, Output, Failure>>,
    form: Form<Input, Output, Failure>,
}

impl<Input, Output, Failure> Draft<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn new(pattern: Pattern<Input, Output, Failure>, start: &Position) -> Self {
        Self {
            pattern,
            children: Vec::new(),
            form: Form::new(FormKind::Empty, Span::point(start.clone())),
        }
    }

    fn build<Source>(&mut self, source: &mut Source, offset: usize) -> (bool, usize)
    where
        Source: Peekable<Input> + Marked,
    {
        let start = source.position();

        let result = match self.pattern.kind.clone() {
            PatternKind::Deferred(evaluator) => {
                let mut guard = evaluator.lock().unwrap();
                let resolved = guard();

                let mut child = Draft::new(resolved, &start);

                let (matches, consumed) = child.build(source, offset);

                if matches {
                    self.children.push(child.clone());

                    self.form = Form::new(
                        FormKind::Multiple(vec![child.form.clone()]),
                        Span::point(start),
                    );
                }

                (matches, consumed)
            }

            PatternKind::Wrap(pattern) => {
                let mut child = Draft::new(*pattern.clone(), &start);
                let (matches, consumed) = child.build(source, offset);

                if matches {
                    self.children.push(child);

                    self.form = self.children[0].form.clone();
                }

                (matches, consumed)
            }

            PatternKind::Guard { predicate, pattern } => {
                let mut guard = predicate.lock().unwrap();

                if guard(source) {
                    let mut child = Draft::new(*pattern, &start);
                    let (matches, consumed) = child.build(source, offset);

                    if matches {
                        self.children.push(child);
                        self.form = self.children[0].form.clone();
                    } else {
                    }

                    (matches, consumed)
                } else {
                    (false, offset)
                }
            }

            PatternKind::Literal(ref expect) => {
                if let Some(peek) = source.peek_ahead(offset) {
                    let matches = *peek == *expect;

                    if matches {
                        self.form = Form::new(FormKind::Input(expect.clone()), Span::point(start));
                    }

                    (matches, if matches { offset + 1 } else { offset })
                } else {
                    (false, offset)
                }
            }

            PatternKind::Alternative(ref patterns) => {
                for inner in patterns {
                    let mut child = Draft::new(inner.clone(), &start);
                    let (matches, consumed) = child.build(source, offset);

                    if matches {
                        self.children.push(child);
                        self.form = self.children[0].form.clone();

                        return (true, consumed);
                    }
                }

                (false, offset)
            }

            PatternKind::Sequence(ref sequence) => {
                let mut current = offset;
                let mut forms = Vec::new();

                for pattern in sequence {
                    let mut child = Draft::new(pattern.clone(), &start);
                    let (matches, consumed) = child.build(source, current);

                    if matches {
                        current = consumed;
                        forms.push(child.form.clone());
                        self.children.push(child);
                    } else {
                        return (false, offset);
                    }
                }

                if !forms.is_empty() {
                    self.form = Form::new(FormKind::Multiple(forms), Span::point(start));
                }

                (true, current)
            }

            PatternKind::Repetition {
                pattern,
                minimum,
                maximum,
            } => {
                let mut count = 0;
                let mut current = offset;
                let mut forms = Vec::new();

                while source.peek_ahead(current).is_some() {
                    let mut child = Draft::new(*pattern.clone(), &start);
                    let (matches, new) = child.build(source, current);

                    if !matches || new == current {
                        break;
                    }

                    count += 1;
                    current = new;

                    forms.push(child.form.clone());
                    self.children.push(child);

                    if let Some(max) = maximum {
                        if count >= max {
                            break;
                        }
                    }
                }

                let success = count >= minimum;

                if success && !forms.is_empty() {
                    self.form = Form::new(FormKind::Multiple(forms), Span::point(start));
                }

                (success, if success { current } else { offset })
            }

            PatternKind::Optional(pattern) => {
                let mut child = Draft::new(*pattern.clone(), &start);
                let (matches, new) = child.build(source, offset);

                if matches {
                    self.children.push(child);
                    self.form = self.children[0].form.clone();
                }

                (true, if matches { new } else { offset })
            }

            PatternKind::Condition(function) => {
                if let Some(peek) = source.peek_ahead(offset) {
                    let mut guard = function.lock().unwrap();

                    let result = guard(&peek);

                    if result {
                        self.form = Form::new(FormKind::Input(peek.clone()), Span::point(start));
                    }

                    (result, if result { offset + 1 } else { offset })
                } else {
                    (false, offset)
                }
            }

            PatternKind::Negation(pattern) => {
                if source.peek_ahead(offset).is_none() {
                    return (false, offset);
                }

                let mut child = Draft::new(*pattern.clone(), &start);
                let (matches, _) = child.build(source, offset);
                let result = !matches;

                if result {
                    if let Some(peek) = source.peek_ahead(offset) {
                        self.form = Form::new(FormKind::Input(peek.clone()), Span::point(start));
                    }
                }

                (result, if result { offset + 1 } else { offset })
            }

            PatternKind::WildCard => {
                if let Some(peek) = source.peek_ahead(offset) {
                    self.form = Form::new(FormKind::Input(peek.clone()), Span::point(start));

                    (true, offset + 1)
                } else {
                    (false, offset)
                }
            }
        };

        result
    }

    fn realize<Source>(&mut self, source: &mut Source) -> Form<Input, Output, Failure>
    where
        Source: Peekable<Input> + Marked,
    {
        let start = source.position();

        match self.pattern.kind.clone() {
            PatternKind::Literal(_) => {
                if let Some(input) = source.next() {
                    let end = source.position();

                    self.form = Form::new(FormKind::Input(input), Span::new(start, end));
                }
            }

            PatternKind::Condition(_) => {
                if let Some(input) = source.next() {
                    let end = source.position();

                    self.form = Form::new(FormKind::Input(input), Span::new(start, end));
                }
            }

            PatternKind::Negation(_) => {
                if let Some(input) = source.next() {
                    let end = source.position();

                    self.form = Form::new(FormKind::Input(input), Span::new(start, end));
                }
            }

            PatternKind::WildCard => {
                if let Some(input) = source.next() {
                    let end = source.position();

                    self.form = Form::new(FormKind::Input(input), Span::new(start, end));
                }
            }

            _ => {
                for child in self.children.iter_mut() {
                    child.realize(source);
                }

                let end = source.position();
                let span = Span::new(start, end);

                match self.pattern.kind.clone() {
                    PatternKind::Sequence(_) | PatternKind::Repetition { .. } => {
                        let forms: Vec<_> = self.children.iter().map(|c| c.form.clone()).collect();

                        if !forms.is_empty() {
                            self.form = Form::new(FormKind::Multiple(forms), span);
                        }
                    }

                    PatternKind::Alternative(_)
                    | PatternKind::Optional(_)
                    | PatternKind::Guard { .. }
                    | PatternKind::Deferred(_)
                    | PatternKind::Wrap(_) => {
                        if !self.children.is_empty() {
                            self.form = self.children[0].form.clone();
                        }
                    }

                    _ => {}
                }
            }
        }

        if let Some(ref action) = self.pattern.action {
            let span = self.form.span.clone();

            self.form = source.action(action, self.form.clone(), span);
        }

        self.form.clone()
    }
}

pub trait Former<Input, Output, Failure>: Peekable<Input> + Marked
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn action(
        &mut self,
        action: &Action<Input, Output, Failure>,
        form: Form<Input, Output, Failure>,
        span: Span,
    ) -> Form<Input, Output, Failure>;

    fn form(&mut self, pattern: Pattern<Input, Output, Failure>) -> Form<Input, Output, Failure>;
}

impl<Source, Input, Output, Failure> Former<Input, Output, Failure> for Source
where
    Source: Peekable<Input> + Marked,
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn action(
        &mut self,
        action: &Action<Input, Output, Failure>,
        form: Form<Input, Output, Failure>,
        span: Span,
    ) -> Form<Input, Output, Failure> {
        let result = match action {
            Action::Map(transform) => {
                let mut guard = transform.lock().unwrap();
                let context = &mut self.context_mut();

                match guard(context, form.clone()) {
                    Ok(output) => {
                        let mapped = Form::new(FormKind::Output(output), span);

                        mapped
                    }
                    Err(error) => Form::new(FormKind::Failure(error), span),
                }
            }

            Action::Execute(executor) => {
                let mut guard = executor.lock().unwrap();
                guard();

                form.clone()
            }

            Action::Multiple(actions) => {
                let mut current = form.clone();

                for action in actions.iter() {
                    current = self.action(action, current, span.clone());
                }

                current
            }

            Action::Trigger { found, missing } => {
                let has_content = match &form.kind {
                    FormKind::Empty => false,
                    FormKind::Failure(_) => false,
                    FormKind::Input(_) | FormKind::Output(_) | FormKind::Multiple(_) => true,
                };

                let chosen = if has_content { found } else { missing };

                self.action(chosen, form, span)
            }

            Action::Ignore => Form::new(FormKind::Empty, span),

            Action::Capture { identifier } => {
                let resolver = &mut self.context_mut().resolver;

                let artifact = form.clone().map(
                    |input| Artifact::new(input),
                    |output| Artifact::new(output),
                    |error| Artifact::new(error),
                );

                let item = Item::new(
                    ItemKind::Formed {
                        identifier: *identifier,
                        form: artifact,
                    },
                    form.span.clone(),
                );

                resolver.insert(item);

                form.clone()
            }

            Action::Failure(function) => {
                let mut guard = function.lock().unwrap();
                let form = Form::new(FormKind::Failure(guard(span.clone())), span);

                form
            }

            Action::Inspect(inspector) => {
                let mut guard = inspector.lock().unwrap();
                guard(form.clone());

                form.clone()
            }
        };

        result
    }

    fn form(&mut self, pattern: Pattern<Input, Output, Failure>) -> Form<Input, Output, Failure> {
        let start = self.position();

        let mut draft = Draft::new(pattern, &start);
        let (_, _) = draft.build(self, 0);

        let form = draft.realize(self);

        form
    }
}