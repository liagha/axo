use {
    log::{debug, trace, warn},

    super::{
        action::Action,
        pattern::{Pattern, PatternKind},
    },

    crate::{
        Peekable,
        hash::Hash,
        format::Debug,
        compiler::Marked,
        artifact::Artifact,

        axo_span::Span,
        axo_parser::{Item, ItemKind},
        axo_form::form::{Form, FormKind},
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
    fn new(pattern: Pattern<Input, Output, Failure>, start: &crate::axo_span::Position) -> Self {
        Self {
            pattern,
            children: Vec::new(),
            form: Form::new(FormKind::Empty, Span::point(start.clone())),
        }
    }

    fn matches(&self) -> bool {
        !matches!(self.form.kind, FormKind::Empty)
    }

    fn consumed(&self) -> usize {
        if self.matches() {
            1 + self.children.iter().map(|child| child.consumed()).sum::<usize>()
        } else {
            0
        }
    }

    fn build<Source>(
        &mut self,
        source: &mut Source,
        offset: usize
    ) -> (bool, usize)
    where
        Source: Peekable<Input> + Marked,
    {
        let start = source.position();

        let result = match self.pattern.kind.clone() {
            PatternKind::Deferred(factory) => {
                let resolved = factory();
                let mut child = Draft::new(resolved, &start);
                let (matches, consumed) = child.build(source, offset);

                if matches {
                    self.children.push(child.clone());
                    self.form = Form::new(FormKind::Multiple(vec![child.form.clone()]), Span::point(start));
                }

                (matches, consumed)
            }

            PatternKind::Guard { predicate, pattern } => {
                if predicate(source) {
                    let mut child = Draft::new(*pattern, &start);
                    let (matches, consumed) = child.build(source, offset);

                    if matches {
                        self.children.push(child);
                        self.form = self.children[0].form.clone();
                    }

                    (matches, consumed)
                } else {
                    (false, offset)
                }
            }

            PatternKind::Literal(expect) => {
                if let Some(peek) = source.peek_ahead(offset) {
                    let matches = *peek == expect;
                    if matches {
                        self.form = Form::new(FormKind::Input(expect), Span::point(start));
                    }
                    (matches, if matches { offset + 1 } else { offset })
                } else {
                    (false, offset)
                }
            }

            PatternKind::Alternative(patterns) => {
                debug!("building alternative tree with {} options at offset {}", patterns.len(), offset);

                for (index, subpattern) in patterns.iter().enumerate() {
                    let mut child = Draft::new(subpattern.clone(), &start);
                    let (matches, new_offset) = child.build(source, offset);

                    if matches {
                        trace!("alternative {} matched successfully", index);
                        self.children.push(child);
                        self.form = self.children[0].form.clone();
                        return (true, new_offset);
                    }
                }

                (false, offset)
            }

            PatternKind::Sequence(sequence) => {
                debug!("building sequence tree with {} elements at offset {}", sequence.len(), offset);
                let mut current = offset;
                let mut forms = Vec::new();

                for (index, pattern) in sequence.iter().enumerate() {
                    let mut child = Draft::new(pattern.clone(), &start);
                    let (matches, new) = child.build(source, current);

                    if matches {
                        current = new;
                        forms.push(child.form.clone());
                        self.children.push(child);
                    } else {
                        trace!("sequence element {} failed at position {}", index, current);
                        return (false, offset);
                    }
                }

                if !forms.is_empty() {
                    self.form = Form::new(FormKind::Multiple(forms), Span::point(start));
                }

                (true, current)
            }

            PatternKind::Repetition { pattern, minimum, maximum } => {
                debug!("building repetition tree (min: {}, max: {:?}) at offset {}", minimum, maximum, offset);
                let mut current = offset;
                let mut count = 0;
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

                trace!("repetition matched {} times, meets minimum requirement: {}", count, success);
                (success, if success { current } else { offset })
            }

            PatternKind::Optional(pattern) => {
                let mut child = Draft::new(*pattern, &start);
                let (matches, new) = child.build(source, offset);

                if matches {
                    self.children.push(child);
                    self.form = self.children[0].form.clone();
                }

                (true, if matches { new } else { offset })
            }

            PatternKind::Condition(function) => {
                if let Some(peek) = source.peek_ahead(offset) {
                    let result = function(&peek);
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

                let mut child = Draft::new(*pattern, &start);
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

            PatternKind::Required { pattern, .. } => {
                let mut child = Draft::new(*pattern, &start);
                let (matches, new) = child.build(source, offset);

                if matches {
                    self.children.push(child);
                    self.form = self.children[0].form.clone();
                    (true, new)
                } else {
                    (true, offset)
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
                // For composite patterns, realize children first
                for child in &mut self.children {
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

                    PatternKind::Alternative(_) | PatternKind::Optional(_) |
                    PatternKind::Guard { .. } | PatternKind::Deferred(_) => {
                        if !self.children.is_empty() {
                            self.form = self.children[0].form.clone();
                        }
                    }

                    PatternKind::Required { action, .. } => {
                        if self.children.is_empty() {
                            warn!("required pattern failed to match, creating error form");
                            self.form = source.action(&action, Form::new(FormKind::Empty, span.clone()), span);
                        } else {
                            self.form = self.children[0].form.clone();
                        }
                    }

                    _ => {}
                }
            }
        }

        if let Some(action) = &self.pattern.action {
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

    fn fit(&mut self, pattern: &Pattern<Input, Output, Failure>, offset: usize) -> (bool, usize);
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
        if let Some(err) = form.catch() {
            warn!("caught error in form before action processing, returning early");
            return err;
        }

        let result = match action {
            Action::Map(transform) => {
                debug!("applying map transformation at span {:?}", span);
                let context = &mut self.context_mut();

                match transform(context, form) {
                    Ok(output) => Form::new(FormKind::Output(output), span),
                    Err(_) => {
                        warn!("transformation failed, returning empty form");
                        Form::new(FormKind::Empty, span)
                    },
                }
            },

            Action::Ignore => {
                debug!("ignoring form content at span {:?}", span);
                Form::new(FormKind::Empty, span)
            },

            Action::Capture { identifier } => {
                debug!("forming capture pattern with identifier");

                let resolver = &mut self.context_mut().resolver;

                let artifact = form.clone().map(
                    |input| Artifact::new(input),
                    |output| Artifact::new(output),
                    |error| Artifact::new(error),
                );
                
                let item = Item::new(
                    ItemKind::Formed { identifier: *identifier, form: artifact },
                    form.span.clone(),
                );
                
                resolver.insert(item);

                form.clone()
            }

            Action::Failure(function) => {
                warn!("creating error form with function at span {:?}", span);

                Form::new(FormKind::Failure(function(span.clone())), span)
            },

            Action::Trigger { found, .. } => {
                debug!("triggering nested action at span {:?}", span);

                self.action(found, form, span)
            }
        };

        result
    }

    fn fit(&mut self, pattern: &Pattern<Input, Output, Failure>, offset: usize) -> (bool, usize) {
        let start = self.position();
        let mut draft = Draft::new(pattern.clone(), &start);
        draft.build(self, offset)
    }

    fn form(&mut self, pattern: Pattern<Input, Output, Failure>) -> Form<Input, Output, Failure> {
        let start = self.position();

        debug!("forming pattern at position {:?}", start);

        let mut draft = Draft::new(pattern, &start);

        let (matches, _) = draft.build(self, 0);

        if !matches {
            debug!("pattern failed to match, returning empty form");

            return Form::new(FormKind::Empty, Span::point(start));
        }

        draft.realize(self)
    }
}