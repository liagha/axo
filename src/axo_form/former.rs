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
    pub fn is_skipped(&self) -> bool {
        matches!(self, &Record::Skipped)
    }

    #[inline]
    pub fn is_failed(&self) -> bool { 
        matches!(self, Record::Failed) 
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

    #[inline]
    pub fn empty(&mut self) {
        *self = Record::Blank;
    }
}

#[derive(Clone, Debug)]
pub struct Draft<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub offset: usize,
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
    pub fn new(offset: usize, pattern: Pattern<Input, Output, Failure>, span: Span) -> Self {
        Self {
            offset,
            pattern,
            form: Form::new(FormKind::Blank, span),
            record: Record::Blank,
            children: Vec::new(),
        }
    }

    pub fn build<Source>(&mut self, source: &mut Source)
    where
        Source: Peekable<Input> + Marked,
    {
        match self.pattern.kind.clone() {
            // Parents
            PatternKind::Alternative(patterns) => {
                let mut result: Option<Draft<Input, Output, Failure>> = None;

                for pattern in patterns {
                    let mut draft = Draft::new(self.offset, pattern, Span::default());
                    draft.build(source);

                    match draft.record {
                        Record::Aligned => {
                            result = Some(draft);
                            
                            break;
                        }
                        Record::Skipped => {
                            if let Some(stack) = &result {
                                if stack.record.is_failed() {
                                    result = Some(draft);
                                }
                            } else { 
                                result = Some(draft);
                            }
                        }
                        Record::Failed => {
                            if result.is_none() {
                                result = Some(draft);
                            }
                        }
                        Record::Blank => {
                            continue;
                        }
                    }
                }

                if let Some(draft) = result {
                    *self = draft;
                } else {
                    self.record.empty();
                }
            }
            
            PatternKind::Deferred(function) => {
                let mut guard = function.lock().unwrap();
                let resolved = guard();
                drop(guard);

                let mut draft = Draft::new(self.offset, resolved, Span::default());
                draft.build(source);

                *self = draft;
            }

            PatternKind::Optional(pattern) => {
                let mut draft = Draft::new(self.offset, *pattern, Span::default());
                draft.build(source);

                if draft.record.is_effected() {
                    *self = draft;
                }

                self.record.align();
            }

            PatternKind::Wrapper(pattern) => {
                let mut child = Draft::new(self.offset, *pattern, Span::default());
                child.build(source);

                *self = child;
            }
            
            // Consumers
            PatternKind::Literal(expect) => {
                if let Some(peek) = source.peek_ahead(self.offset) {
                    if *peek == expect {
                        self.offset += 1;
                        self.record.align();
                        self.form = Form::new(FormKind::Input(expect), Span::default());
                    }
                }
            }

            PatternKind::Predicate(function) => {
                if let Some(peek) = source.peek_ahead(self.offset) {
                    let mut guard = function.lock().unwrap();
                    let result = guard(peek);
                    drop(guard);

                    if result {
                        self.offset += 1;
                        self.record.align();
                        self.form = Form::new(FormKind::Input(peek.clone()), Span::default());
                    }
                }
            }

            PatternKind::Negation(pattern) => {
                if let Some(peek) = source.peek_ahead(self.offset).cloned() {
                    let mut child = Draft::new(self.offset, *pattern, Span::default());
                    child.build(source);

                    if !child.record.is_aligned() {
                        self.offset += 1;
                        self.record.align();
                        self.form = Form::new(FormKind::Input(peek), Span::default());
                    } else {
                        self.record.empty();
                    }
                } else {
                    self.record.empty();
                }
            }

            PatternKind::WildCard => {
                if let Some(peek) = source.peek_ahead(self.offset) {
                    self.offset += 1;
                    self.record.align();
                    self.form = Form::new(FormKind::Input(peek.clone()), Span::default());
                }
            }

            // Chains
            PatternKind::Sequence(sequence) => {
                let mut current = self.offset;
                self.children.reserve(sequence.len());

                for pattern in sequence {
                    let mut child = Draft::new(current, pattern, Span::default());
                    child.build(source);

                    match child.record {
                        Record::Aligned => {
                            current = child.offset;
                            self.record.align();
                            self.children.push(child);
                        }
                        Record::Failed => {
                            current = child.offset;
                            self.record.fail();
                            self.children.push(child);

                            break;
                        }
                        Record::Blank => {
                            current = self.offset;
                            self.record.empty();

                            break;
                        }
                        
                        Record::Skipped => {}
                    }
                }

                self.offset = current;
            }

            PatternKind::Repetition {
                pattern,
                minimum,
                maximum,
            } => {
                let mut count = 0;
                let mut current = self.offset;

                while source.peek_ahead(current).is_some() {
                    let mut child = Draft::new(current, *pattern.clone(), Span::default());
                    child.build(source);

                    if child.offset == current {
                        break;
                    }

                    match child.record {
                        Record::Aligned | Record::Failed => {
                            count += 1;
                            current = child.offset;
                            self.children.push(child);
                        }

                        Record::Blank => break,
                        
                        Record::Skipped => {}
                    }

                    if let Some(max) = maximum {
                        if count >= max {
                            break;
                        }
                    }
                }

                if count >= minimum {
                    self.offset = current;
                    self.record.align();
                } else {
                    self.record.empty();
                }
            }
        }

        if let Some(action) = &self.pattern.action.clone() {
            action.apply(source, self);
        }
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
                if let Some(input) = source.advance() {
                    let end = source.position();

                    self.form = Form::new(FormKind::Input(input), Span::new(start, end));
                }
            }

            PatternKind::Alternative(_)
            | PatternKind::Optional(_)
            | PatternKind::Deferred(_)
            | PatternKind::Wrapper(_) => {
                for child in &mut self.children {
                    child.realize(source);

                    self.form = child.form.clone();
                }
            }

            PatternKind::Sequence(_) | PatternKind::Repetition { .. } => {
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
        let mut draft = Draft::new(0, pattern, Span::default());

        draft.build(self);

        draft.realize(self);

        draft.form
    }
}
