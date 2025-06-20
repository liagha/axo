use {
    super::pattern::{Pattern, PatternKind},
    crate::{
        axo_cursor::{Peekable, Span},
        axo_form::form::{Form, FormKind},
        compiler::Marked,
        format::Debug,
        hash::Hash,
        memory::drop,
    },
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Record {
    Aligned,
    Skipped,
    Failed,
    Blank,
}

impl Record {
    #[inline]
    pub fn is_aligned(&self) -> bool {
        matches!(self, &Record::Aligned)
    }

    #[inline]
    pub fn is_effected(&self) -> bool {
        matches!(self, &Record::Aligned | &Record::Failed)
    }

    #[inline]
    pub fn is_blank(&self) -> bool {
        matches!(self, &Record::Blank)
    }

    #[inline]
    pub fn align(&mut self) {
        *self = Record::Aligned;
    }

    #[inline]
    pub fn skip(&mut self) {
        *self = Record::Skipped;
    }

    #[inline]
    pub fn fail(&mut self) {
        *self = Record::Failed;
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
    #[inline]
    pub fn new(pattern: Pattern<Input, Output, Failure>, span: Span) -> Self {
        Self {
            pattern,
            form: Form::new(FormKind::Blank, span),
            record: Record::Blank,
            children: Vec::new(),
        }
    }

    pub fn build<Source>(&mut self, source: &mut Source, offset: usize) -> usize
    where
        Source: Peekable<Input> + Marked,
    {
        let consumed = match &self.pattern.kind {
            PatternKind::Deferred(function) => {
                let mut guard = function.lock().unwrap();
                let resolved = guard();
                drop(guard);

                let mut child = Draft::new(resolved, Span::default());
                let consumed = child.build(source, offset);

                self.record = child.record;
                self.form = child.form.clone();
                self.children.push(child);

                consumed
            }

            PatternKind::Wrapper(pattern) => {
                let mut child = Draft::new((**pattern).clone(), Span::default());
                let consumed = child.build(source, offset);

                self.record = child.record;
                self.form = child.form.clone();
                self.children.push(child);

                consumed
            }

            PatternKind::Literal(expect) => {
                if let Some(peek) = source.peek_ahead(offset) {
                    if peek == expect {
                        self.record.align();
                        self.form =
                            Form::new(FormKind::Input(expect.clone()), Span::default());

                        offset + 1
                    } else {
                        offset
                    }
                } else {
                    offset
                }
            }

            PatternKind::Alternative(patterns) => {
                let mut failed = None;

                for pattern in patterns {
                    let mut child = Draft::new(pattern.clone(), Span::default());
                    let consumed = child.build(source, offset);

                    match child.record {
                        Record::Aligned => {
                            self.record.align();
                            self.form = child.form.clone();
                            self.children.push(child);

                            return consumed;
                        }
                        Record::Skipped => {
                            self.children.push(child);
                        }
                        Record::Failed => {
                            if failed.is_none() {
                                failed = Some((child, consumed));
                            }
                        }
                        Record::Blank => {
                            continue;
                        }
                    }
                }

                if let Some((child, consumed)) = failed {
                    self.record.fail();
                    self.form = child.form.clone();
                    self.children.push(child);
                    consumed
                } else {
                    self.record = Record::Blank;
                    offset
                }
            }

            PatternKind::Sequence(sequence) => {
                let mut current = offset;
                self.children.reserve(sequence.len());

                for pattern in sequence {
                    let mut child = Draft::new(pattern.clone(), Span::default());
                    let consumed = child.build(source, current);

                    match child.record {
                        Record::Aligned | Record::Skipped => {
                            current = consumed;
                            self.record.align();
                            self.children.push(child);
                        }
                        Record::Failed => {
                            current = consumed;
                            self.record.fail();
                            self.children.push(child);

                            break;
                        }
                        Record::Blank => {
                            current = offset;
                            self.record = Record::Blank;

                            break;
                        }
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
                    let mut child = Draft::new(*pattern.clone(), Span::default());
                    let consumed = child.build(source, current);

                    if consumed == current || child.record.is_blank() {
                        break;
                    }

                    count += 1;
                    current = consumed;
                    forms.push(child.form.clone());

                    self.children.push(child);

                    if let Some(max) = maximum {
                        if count >= *max {
                            break;
                        }
                    }
                }

                if count >= *minimum {
                    self.record.align();
                    self.form = Form::new(FormKind::Multiple(forms), Span::default());
                    current
                } else {
                    self.record = Record::Blank;
                    offset
                }
            }

            PatternKind::Optional(pattern) => {
                let mut child = Draft::new((**pattern).clone(), Span::default());
                let consumed = child.build(source, offset);

                self.record.align();
                
                match child.record {
                    Record::Aligned | Record::Failed => {
                        self.form = child.form.clone();
                        self.children.push(child);

                        consumed
                    }
                    _ => {
                        offset
                    }
                }
            }

            PatternKind::Predicate(function) => {
                if let Some(peek) = source.peek_ahead(offset) {
                    let mut guard = function.lock().unwrap();
                    let result = guard(peek);
                    drop(guard);

                    if result {
                        self.record.align();
                        self.form =
                            Form::new(FormKind::Input(peek.clone()), Span::default());
                        offset + 1
                    } else {
                        offset
                    }
                } else {
                    offset
                }
            }

            PatternKind::Negation(pattern) => {
                if source.peek_ahead(offset).is_some() {
                    let mut child = Draft::new((**pattern).clone(), Span::default());
                    child.build(source, offset);

                    if child.record != Record::Aligned {
                        self.record.align();
                        if let Some(peek) = source.peek_ahead(offset) {
                            self.form = Form::new(
                                FormKind::Input(peek.clone()),
                                Span::default(),
                            );
                        }
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

            PatternKind::WildCard => {
                if let Some(peek) = source.peek_ahead(offset) {
                    self.record.align();
                    self.form =
                        Form::new(FormKind::Input(peek.clone()), Span::default());
                    offset + 1
                } else {
                    offset
                }
            }
        };

        if let Some(action) = &self.pattern.action.clone() {
            action.apply(source, self);
        }

        consumed
    }

    pub fn realize<Source>(&mut self, source: &mut Source)
    where
        Source: Peekable<Input> + Marked,
    {
        let start = source.position();

        match &self.pattern.kind {
            PatternKind::Literal(_)
            | PatternKind::Predicate(_)
            | PatternKind::Negation(_)
            | PatternKind::WildCard => {
                if self.record.is_aligned() {
                    if let Some(input) = source.advance() {
                        let end = source.position();

                        self.form = Form::new(FormKind::Input(input), Span::new(start, end));
                    }
                }
            }

            PatternKind::Alternative(_)
            | PatternKind::Optional(_)
            | PatternKind::Deferred(_)
            | PatternKind::Wrapper(_) => {
                if self.record.is_effected() {
                    for child in &mut self.children {
                        child.realize(source);

                        if child.record.is_effected() {
                            self.form = child.form.clone();
                        }
                    }
                }
            }

            PatternKind::Sequence(_) | PatternKind::Repetition { .. } => {
                if self.record.is_effected() {
                    for child in &mut self.children {
                        child.realize(source);
                    }

                    let forms: Vec<_> = self
                        .children
                        .iter()
                        .map(|draft| draft.form.clone())
                        .collect();

                    let end = source.position();

                    self.form = Form::new(FormKind::Multiple(forms), Span::new(start, end));
                }
            }
        }

        self.children.clear();

        if let Some(action) = &self.pattern.action.clone() {
            action.execute(source, self);
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
        let mut draft = Draft::new(pattern, Span::default());

        draft.build(self, 0);

        if draft.record != Record::Blank {
            draft.realize(self);
        }

        draft.form
    }
}
