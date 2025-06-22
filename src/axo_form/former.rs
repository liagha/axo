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
use crate::axo_cursor::{Position, Spanned};

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
    pub index: usize,
    pub position: Position,
    pub record: Record,
    pub pattern: Pattern<Input, Output, Failure>,
    pub form: Form<Input, Output, Failure>,
}

impl<Input, Output, Failure> Draft<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    #[inline]
    pub fn new(index: usize, position: Position, pattern: Pattern<Input, Output, Failure>) -> Self {
        Self {
            index,
            position: position.clone(),
            record: Record::Blank,
            pattern,
            form: Form::new(FormKind::Blank, Span::point(position.clone())),
        }
    }

    pub fn build<Source>(&mut self, source: &mut Source)
    where
        Source: Peekable<Input> + Marked,
    {
        match self.pattern.kind.clone() {
            // Consumers
            PatternKind::Literal(expect) => {
                if let Some(peek) = source.get(self.index).cloned() {
                    if peek == expect {
                        let start = self.position.clone();
                        source.next(&mut self.index, &mut self.position);
                        let end = self.position.clone();
                        
                        self.record.align();
                        self.form = Form::new(FormKind::Input(peek), Span::new(start, end));
                    }
                }
            }

            PatternKind::Negation(pattern) => {
                if let Some(peek) = source.get(self.index).cloned() {
                    let mut draft = Draft::new(self.index, self.position.clone(), *pattern);
                    draft.build(source);

                    if !draft.record.is_aligned() {
                        let start = self.position.clone();
                        source.next(&mut self.index, &mut self.position);
                        let end = self.position.clone();

                        self.record.align();
                        self.form = Form::new(FormKind::Input(peek), Span::new(start, end));
                    }
                }
            }

            PatternKind::Predicate(function) => {
                if let Some(peek) = source.get(self.index).cloned() {
                    let mut guard = function.lock().unwrap();
                    let result = guard(&peek);
                    drop(guard);

                    if result {
                        let start = self.position.clone();
                        source.next(&mut self.index, &mut self.position);
                        let end = self.position.clone();

                        self.record.align();
                        self.form = Form::new(FormKind::Input(peek), Span::new(start, end));
                    }
                }
            }

            PatternKind::WildCard => {
                if let Some(peek) = source.get(self.index).cloned() {
                    let start = self.position.clone();
                    source.next(&mut self.index, &mut self.position);
                    let end = self.position.clone();

                    self.record.align();
                    self.form = Form::new(FormKind::Input(peek), Span::new(start, end));
                }
            }

            // Parents
            PatternKind::Alternative(patterns) => {
                let mut fallback = None;
                
                for pattern in patterns {
                    let mut draft = Draft::new(self.index, self.position.clone(), pattern);
                    draft.build(source);

                    match draft.record {
                        Record::Aligned => {
                            self.index = draft.index;
                            self.position = draft.position;
                            self.record.align();
                            self.form = draft.form;

                            break;
                        }
                        Record::Skipped => {
                            self.index = draft.index;
                            self.position = draft.position;
                        }
                        Record::Failed => {
                            if fallback.is_none() {
                                fallback = Some(draft);
                            }
                        }
                        Record::Blank => { continue }
                    }
                }
                
                if let Some(fallback) = fallback {
                    self.index = fallback.index;
                    self.position = fallback.position;
                    self.record.fail();
                    self.form = fallback.form;
                }
            }

            PatternKind::Deferred(function) => {
                let mut guard = function.lock().unwrap();
                let resolved = guard();
                drop(guard);

                let mut draft = Draft::new(self.index, self.position.clone(), resolved);
                draft.build(source);

                self.index = draft.index;
                self.position = draft.position;
                self.record = draft.record;
                self.form = draft.form;
            }

            PatternKind::Optional(pattern) => {
                let mut draft = Draft::new(self.index, self.position.clone(), *pattern);
                draft.build(source);

                if draft.record.is_effected() {
                    self.index = draft.index;
                    self.position = draft.position;
                    self.form = draft.form;
                }

                self.record.align();
            }

            PatternKind::Wrapper(pattern) => {
                let mut draft = Draft::new(self.index, self.position.clone(), *pattern);
                draft.build(source);

                self.index = draft.index;
                self.position = draft.position;
                self.record = draft.record;
                self.form = draft.form;
            }

            // Chains
            PatternKind::Sequence(sequence) => {
                let mut index = self.index;
                let mut position = self.position.clone();
                let mut forms = Vec::with_capacity(sequence.len());

                for pattern in sequence {
                    let mut child = Draft::new(index, position.clone(), pattern);
                    child.build(source);

                    match child.record {
                        Record::Aligned => {
                            index = child.index;
                            position = child.position.clone();
                            self.record.align();
                            forms.push(child.form);
                        }
                        Record::Failed => {
                            index = child.index;
                            position = child.position.clone();
                            self.record.fail();
                            forms.push(child.form);

                            break;
                        }
                        Record::Blank => {
                            index = self.index;
                            position = child.position.clone();
                            self.record.empty();

                            break;
                        }

                        Record::Skipped => {}
                    }
                }

                self.index = index;
                self.position = position;

                self.form = Form::new(FormKind::Multiple(forms.clone()), forms.span());
            }

            PatternKind::Repetition {
                pattern,
                minimum,
                maximum,
            } => {
                let mut index = self.index;
                let mut position = self.position.clone();
                let mut forms = Vec::new();
                
                while source.peek_ahead(index).is_some() {
                    let mut child = Draft::new(index, position.clone(), *pattern.clone());
                    child.build(source);

                    if child.index == index {
                        break;
                    }

                    match child.record {
                        Record::Aligned | Record::Skipped | Record::Failed => {
                            index = child.index;
                            position = child.position.clone();
                            forms.push(child.form);
                        }
                        Record::Blank => {
                            index = self.index;
                            position = child.position.clone();

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
                    self.index = index;
                    self.position = position.clone();
                    self.record.align();
                    self.form = Form::new(FormKind::Multiple(forms.clone()), forms.span());
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
        let mut draft = Draft::new(0, self.position(), pattern);

        draft.build(self);

        if draft.record.is_effected() {
            self.set_index(draft.index);
            self.set_position(draft.position);
        }
        
        draft.form
    }
}
