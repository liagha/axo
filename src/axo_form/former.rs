use {
    super::{
        pattern::{Pattern, PatternKind},
    },
    crate::{
        axo_cursor::{
            Position, Peekable, 
            Span, Spanned,
        },
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
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub marker: usize,
    pub position: Position,
    pub consumed: Vec<Input>,
    pub record: Record,
    pub pattern: Pattern<Input, Output, Failure>,
    pub form: Form<Input, Output, Failure>,
}

impl<Input, Output, Failure> Draft<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    #[inline]
    pub fn new(index: usize, position: Position, pattern: Pattern<Input, Output, Failure>) -> Self {
        Self {
            marker: index,
            position,
            consumed: Vec::new(),
            record: Record::Blank,
            pattern,
            form: Form::new(FormKind::Blank, Span::point(position)),
        }
    }

    pub fn build<Source>(&mut self, source: &mut Source)
    where
        Source: Peekable<Input> + Marked,
    {
        match self.pattern.kind.clone() {
            // Consumers
            PatternKind::Literal { value } => {
                if let Some(peek) = source.get(self.marker).cloned() {
                    if peek == value {
                        source.next(&mut self.marker, &mut self.position);

                        self.consumed.push(peek.clone());
                        self.record.align();
                        self.form = Form::input(peek);
                    } else {
                        self.record.empty();
                    }
                } else {
                    self.record.empty();
                }
            }
            
            PatternKind::Twin { value } => {
                if let Some(peek) = source.get(self.marker).cloned() {
                    if value.eq(&peek) {
                        source.next(&mut self.marker, &mut self.position);

                        self.consumed.push(peek.clone());
                        self.record.align();
                        self.form = Form::input(peek);
                    } else {
                        self.record.empty();
                    }
                } else {
                    self.record.empty();
                } 
            }

            PatternKind::Negation { pattern } => {
                if let Some(peek) = source.get(self.marker).cloned() {
                    let mut draft = Draft::new(self.marker, self.position, *pattern);
                    draft.build(source);

                    if !draft.record.is_aligned() {
                        source.next(&mut self.marker, &mut self.position);

                        self.consumed.push(peek.clone());
                        self.record.align();
                        self.form = Form::input(peek);
                    } else {
                        self.record.empty();
                    }
                } else {
                    self.record.empty();
                }
            }

            PatternKind::Predicate { function } => {
                if let Some(peek) = source.get(self.marker).cloned() {
                    let predicate = function.lock().map_or(false, |mut guard| {
                        let predicate = guard(&peek);
                        drop(guard);
                        predicate
                    });

                    if predicate {
                        source.next(&mut self.marker, &mut self.position);

                        self.consumed.push(peek.clone());
                        self.record.align();
                        self.form = Form::input(peek);
                    } else {
                        self.record.empty();
                    }
                } else {
                    self.record.empty();
                }
            }

            // Parents
            PatternKind::Alternative { patterns } => {
                let mut fallback = None;

                for pattern in patterns {
                    let mut draft = Draft::new(self.marker, self.position, pattern);
                    draft.build(source);

                    match draft.record {
                        Record::Aligned => {
                            self.marker = draft.marker;
                            self.position = draft.position;
                            self.consumed = draft.consumed;
                            self.record.align();
                            self.form = draft.form;
                            return;
                        }
                        Record::Skipped => {
                            self.marker = draft.marker;
                            self.position = draft.position;
                        }
                        Record::Failed => {
                            if fallback.is_none() {
                                fallback = Some(draft);
                            }
                        }
                        Record::Blank => {
                            continue;
                        }
                    }
                }

                if let Some(fallback) = fallback {
                    self.marker = fallback.marker;
                    self.position = fallback.position;
                    self.consumed = fallback.consumed;
                    self.record.fail();
                    self.form = fallback.form;
                } else {
                    self.record.empty();
                }
            }

            PatternKind::Deferred { function } => {
                let resolved = if let Ok(mut guard) = function.lock() {
                    let resolved = guard();
                    drop(guard);
                    resolved
                } else {
                    self.record.empty();
                    return;
                };

                let mut draft = Draft::new(self.marker, self.position, resolved);
                draft.build(source);

                self.marker = draft.marker;
                self.position = draft.position;
                self.consumed = draft.consumed;
                self.record = draft.record;
                self.form = draft.form;
            }

            PatternKind::Optional { pattern } => {
                let mut draft = Draft::new(self.marker, self.position, *pattern);
                draft.build(source);

                if draft.record.is_effected() {
                    self.marker = draft.marker;
                    self.position = draft.position;
                    self.consumed = draft.consumed;
                    self.form = draft.form;
                }

                self.record.align();
            }

            PatternKind::Wrapper { pattern } => {
                let mut draft = Draft::new(self.marker, self.position, *pattern);
                draft.build(source);

                self.marker = draft.marker;
                self.position = draft.position;
                self.consumed = draft.consumed;
                self.record = draft.record;
                self.form = draft.form;
            }

            // Chains
            PatternKind::Sequence { patterns } => {
                let mut index = self.marker;
                let mut position = self.position;
                let mut consumed = Vec::new();
                let mut forms = Vec::with_capacity(patterns.len());

                for pattern in patterns {
                    let mut child = Draft::new(index, position, pattern);
                    child.build(source);

                    match child.record {
                        Record::Aligned => {
                            self.record.align();
                            index = child.marker;
                            position = child.position;
                            consumed.extend(child.consumed);
                            forms.push(child.form);
                        }
                        Record::Failed => {
                            self.record.fail();
                            index = child.marker;
                            position = child.position;
                            consumed.extend(child.consumed);
                            forms.push(child.form);
                            break;
                        }
                        Record::Blank => {
                            self.record.empty();
                            break;
                        }
                        Record::Skipped => {}
                    }
                }

                self.marker = index;
                self.position = position;

                if forms.is_empty() {
                    self.consumed.clear();
                    self.form = Form::blank(Span::point(self.position));
                } else {
                    self.consumed = consumed;
                    self.form = Form::multiple(forms.clone());
                }
            }

            PatternKind::Repetition {
                pattern,
                minimum,
                maximum,
            } => {
                let mut index = self.marker;
                let mut position = self.position;
                let mut consumed = Vec::new();
                let mut forms = Vec::new();

                while source.peek_ahead(index).is_some() {
                    let mut child = Draft::new(index, position, *pattern.clone());
                    child.build(source);

                    if child.marker == index {
                        break;
                    }

                    match child.record {
                        Record::Aligned | Record::Failed => {
                            index = child.marker;
                            position = child.position;
                            consumed.extend(child.consumed);
                            forms.push(child.form);
                        }
                        Record::Skipped => {}
                        Record::Blank => {
                            break;
                        }
                    }

                    if let Some(max) = maximum {
                        if forms.len() >= max {
                            break;
                        }
                    }
                }

                if forms.len() >= minimum {
                    self.marker = index;
                    self.position = position;
                    self.consumed = consumed;
                    self.record.align();

                    if forms.is_empty() {
                        self.form = Form::blank(Span::point(self.position));
                    } else {
                        self.form = Form::multiple(forms.clone());
                    }
                } else {
                    self.record.empty();
                }
            }
        }

        if let Some(action) = &self.pattern.action.clone() {
            action.execute(source, self);
        }
    }
}

pub trait Former<Input, Output, Failure>: Peekable<Input> + Marked
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn strain(&mut self, pattern: Pattern<Input, Output, Failure>);
    fn form(&mut self, pattern: Pattern<Input, Output, Failure>) -> Form<Input, Output, Failure>;
}

impl<Source, Input, Output, Failure> Former<Input, Output, Failure> for Source
where
    Source: Peekable<Input> + Marked,
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn strain(&mut self, pattern: Pattern<Input, Output, Failure>) {
        let mut inputs = Vec::with_capacity(self.len());
        let mut index = 0;
        let mut position = self.position();
        
        while self.get(index).is_some() {
            let mut draft = Draft::new(index, position, pattern.clone());
            draft.build(self);

            if draft.record.is_aligned() {
                index = draft.marker + 1;
                position = draft.position;

                inputs.extend(draft.consumed);
            } else { 
                index = draft.marker + 1;
                position = draft.position;
            }
        }
        
        *self.input_mut() = inputs;
    }

    fn form(&mut self, pattern: Pattern<Input, Output, Failure>) -> Form<Input, Output, Failure> {
        let mut draft = Draft::new(0, self.position(), pattern);

        draft.build(self);

        if draft.record.is_effected() {
            self.set_index(draft.marker);
            self.set_position(draft.position);
        }

        draft.form
    }
}