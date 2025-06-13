use {
    super::{
        pattern::{Pattern, PatternKind},
    },
    crate::{
        axo_form::form::{Form, FormKind},
        axo_span::{Position, Span},
        compiler::Marked,
        format::Debug,
        hash::Hash,
        Peekable,
    },
};

#[derive(Clone, PartialEq, Debug)]
pub enum Record {
    Aligned,
    Skipped,
    Failed,
    Blank,
}

impl Record {
    pub fn is_aligned(&self) -> bool {
        self == &Record::Aligned
    }

    pub fn is_skipped(&self) -> bool {
        self == &Record::Skipped
    }

    pub fn is_failed(&self) -> bool {
        self == &Record::Failed
    }

    pub fn is_blank(&self) -> bool {
        self == &Record::Blank
    }
}

#[derive(Clone, Debug)]
pub struct Draft<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub pattern: Pattern<Input, Output, Failure>,
    pub form: Form<Input, Output, Failure>,
    pub record: Record,
    pub children: Vec<Draft<Input, Output, Failure>>,
}

impl<Input, Output, Failure> Draft<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub fn new(pattern: Pattern<Input, Output, Failure>, start: &Position) -> Self {
        Self {
            pattern,
            form: Form::new(FormKind::Empty, Span::point(start.clone())),
            record: Record::Blank,
            children: Vec::new(),
        }
    }

    pub fn build<Source>(&mut self, source: &mut Source, offset: usize) -> usize
    where
        Source: Peekable<Input> + Marked,
    {
        let start = source.position();

        let consumed = match self.pattern.kind.clone() {
            PatternKind::Deferred(evaluator) => {
                let mut guard = evaluator.lock().unwrap();
                let resolved = guard();

                let mut child = Draft::new(resolved, &start);

                let consumed = child.build(source, offset);

                if child.record.is_aligned() {
                    self.record = Record::Aligned;
                    self.children.push(child.clone());

                    self.form = Form::new(
                        FormKind::Multiple(vec![child.form.clone()]),
                        Span::point(start),
                    );

                    consumed
                } else if child.record.is_failed() {
                    self.record = Record::Failed;
                    self.children.push(child.clone());

                    self.form = Form::new(
                        FormKind::Multiple(vec![child.form.clone()]),
                        Span::point(start),
                    );

                    consumed
                } else {
                    self.record = Record::Blank;
                    offset
                }
            }

            PatternKind::Wrapper(pattern) => {
                let mut child = Draft::new(*pattern, &start);
                let consumed = child.build(source, offset);

                if child.record.is_aligned() {
                    self.record = Record::Aligned;
                    self.children.push(child);
                    self.form = self.children[0].form.clone();
                    consumed
                } else if child.record.is_failed() {
                    self.record = Record::Failed;
                    self.children.push(child);
                    self.form = self.children[0].form.clone();
                    consumed
                } else {
                    self.record = Record::Blank;
                    offset
                }
            }

            PatternKind::Guard { predicate, pattern } => {
                let mut guard = predicate.lock().unwrap();

                if guard(source) {
                    let mut child = Draft::new(*pattern, &start);
                    let consumed = child.build(source, offset);

                    if child.record.is_aligned() {
                        self.record = Record::Aligned;
                        self.children.push(child);
                        self.form = self.children[0].form.clone();

                        consumed
                    } else if child.record.is_failed() {
                        self.record = Record::Failed;
                        self.children.push(child);
                        self.form = self.children[0].form.clone();

                        consumed
                    } else {
                        self.record = Record::Blank;
                        offset
                    }
                } else {
                    self.record = Record::Blank;
                    offset
                }
            }

            PatternKind::Literal(ref expect) => {
                if let Some(peek) = source.peek_ahead(offset) {
                    let matches = *peek == *expect;

                    if matches {
                        self.record = Record::Aligned;
                        self.form = Form::new(FormKind::Input(expect.clone()), Span::point(start));
                        offset + 1
                    } else {
                        self.record = Record::Blank;
                        offset
                    }
                } else {
                    self.record = Record::Blank;
                    offset
                }
            }

            PatternKind::Alternative(ref patterns) => {
                self.record = Record::Blank;

                let mut stack = None;

                for inner in patterns {
                    let mut child = Draft::new(inner.clone(), &start);
                    let consumed = child.build(source, offset);

                    if child.record.is_aligned() {
                        self.record = Record::Aligned;
                        self.children.push(child);
                        self.form = self.children[0].form.clone();

                        return consumed;
                    } else if child.record.is_failed() {
                        if stack.is_none() {
                            stack = Some((child, consumed))   
                        }
                    }
                }

                if let Some((child, consumed)) = stack {
                    self.record = Record::Failed;
                    self.children.push(child);
                    self.form = self.children[0].form.clone();

                    consumed
                } else {
                    offset
                }
            }

            PatternKind::Sequence(ref sequence) => {
                let mut current = offset;
                let mut forms = Vec::new();
                self.record = Record::Aligned;

                for pattern in sequence {
                    let mut child = Draft::new(pattern.clone(), &start);
                    let consumed = child.build(source, current);

                    if child.record.is_aligned() {
                        current = consumed;
                        forms.push(child.form.clone());
                        self.children.push(child);
                    } else if child.record.is_failed() {
                        current = consumed;
                        forms.push(child.form.clone());
                        self.children.push(child);

                        self.record = Record::Failed;
                    } else {
                        current = offset;
                        self.record = Record::Blank;

                        if !forms.is_empty() {
                            self.form = Form::new(FormKind::Multiple(forms.clone()), Span::point(start.clone()));
                        }

                        break;
                    }
                }

                current
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
                    let consumed = child.build(source, current);

                    if consumed == current 
                        || child.record.is_blank() 
                    {
                        break;
                    }

                    count += 1;
                    current = consumed;

                    forms.push(child.form.clone());
                    self.children.push(child);

                    if let Some(max) = maximum {
                        if count >= max {
                            break;
                        }
                    }
                }

                let success = count >= minimum;

                if success {
                    self.record = Record::Aligned;
                } else {
                    self.record = Record::Blank;
                }

                if success {
                    self.form = Form::new(FormKind::Multiple(forms), Span::point(start));
                    current
                } else {
                    offset
                }
            }

            PatternKind::Optional(pattern) => {
                let mut child = Draft::new(*pattern.clone(), &start);
                let consumed = child.build(source, offset);

                if child.record.is_aligned() {
                    self.record = Record::Aligned;
                    self.children.push(child);
                    self.form = self.children[0].form.clone();

                    consumed
                } else if child.record.is_failed() {
                    self.record = Record::Failed;
                    self.children.push(child);
                    self.form = self.children[0].form.clone();

                    consumed
                } else {
                    self.record = Record::Aligned;

                    offset
                }
            }

            PatternKind::Condition(function) => {
                if let Some(peek) = source.peek_ahead(offset) {
                    let mut guard = function.lock().unwrap();

                    let result = guard(&peek);

                    if result {
                        self.record = Record::Aligned;
                        self.form = Form::new(FormKind::Input(peek.clone()), Span::point(start));
                        offset + 1
                    } else {
                        self.record = Record::Blank;
                        offset
                    }
                } else {
                    self.record = Record::Blank;
                    offset
                }
            }

            PatternKind::Negation(pattern) => {
                if source.peek_ahead(offset).is_some() {
                    let mut child = Draft::new(*pattern.clone(), &start);
                    child.build(source, offset);

                    let success = child.record != Record::Aligned;

                    if success {
                        self.record = Record::Aligned;
                    } else {
                        self.record = Record::Blank;
                    }

                    if success {
                        if let Some(peek) = source.peek_ahead(offset) {
                            self.form = Form::new(FormKind::Input(peek.clone()), Span::point(start));
                        }
                        offset + 1
                    } else {
                        offset
                    }
                } else {
                    self.record = Record::Blank;
                    offset
                }
            }

            PatternKind::WildCard => {
                if let Some(peek) = source.peek_ahead(offset) {
                    self.record = Record::Aligned;
                    self.form = Form::new(FormKind::Input(peek.clone()), Span::point(start));

                    offset + 1
                } else {
                    self.record = Record::Blank;

                    offset
                }
            }
        };

        if let Some(action) = &self.pattern.action.clone() {
            if action.is_executable() {
                action.execute(source, self);
            }
        }

        consumed
    }

    pub fn realize<Source>(&mut self, source: &mut Source)
    where
        Source: Peekable<Input> + Marked,
    {
        let start = source.position();

        match self.pattern.kind.clone() {
            PatternKind::Literal(_)
            | PatternKind::Condition(_)
            | PatternKind::Negation(_)
            | PatternKind::WildCard => {
                source.next();
            }

            PatternKind::Alternative(_)
            | PatternKind::Optional(_)
            | PatternKind::Guard { .. }
            | PatternKind::Deferred(_)
            | PatternKind::Wrapper(_) => {
                for child in self.children.iter_mut() {
                    child.realize(source);
                }

                if !self.children.is_empty() {
                    self.form = self.children[0].form.clone();
                }
            }

            PatternKind::Sequence(_) | PatternKind::Repetition { .. } => {
                for child in self.children.iter_mut() {
                    child.realize(source);
                }

                let forms: Vec<_> = self.children.iter().map(|c| c.form.clone()).collect();

                let end = source.position();
                let span = Span::new(start.clone(), end);

                if !forms.is_empty() {
                    self.form = Form::new(FormKind::Multiple(forms), span);
                }
            }
        }

        let errors = self.form.catch();

        if errors.is_empty() {
            if let Some(action) = self.pattern.action.clone() {
                if action.is_applicable() {
                    action.apply(source, self);
                }
            }
        }
    }
}

pub trait Former<Input, Output, Failure>: Peekable<Input> + Marked
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn form(&mut self, pattern: Pattern<Input, Output, Failure>) -> Form<Input, Output, Failure>;
}

impl<Source, Input, Output, Failure> Former<Input, Output, Failure> for Source
where
    Source: Peekable<Input> + Marked,
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn form(&mut self, pattern: Pattern<Input, Output, Failure>) -> Form<Input, Output, Failure> {
        let start = self.position();

        let mut draft = Draft::new(pattern, &start);
        draft.build(self, 0);
        draft.realize(self);

        draft.form
    }
}