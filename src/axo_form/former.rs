use {
    super::{
        pattern::{Pattern, PatternKind},
    },
    crate::{
        axo_form::form::{Form, FormKind},
        axo_cursor::{Peekable, Span},
        compiler::Marked,
        format::Debug,
        memory::drop,
        hash::Hash,
    },
};
use crate::axo_form::action::Action;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Record {
    Aligned,
    Skipped,
    Failed,
    Blank,
}

impl Record {
    #[inline]
    pub fn is_aligned(self) -> bool {
        matches!(self, Record::Aligned)
    }

    #[inline]
    pub fn is_skipped(self) -> bool {
        matches!(self, Record::Skipped)
    }

    #[inline]
    pub fn is_failed(self) -> bool {
        matches!(self, Record::Failed)
    }

    #[inline]
    pub fn is_blank(self) -> bool {
        matches!(self, Record::Blank)
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
    pub fn new(pattern: Pattern<Input, Output, Failure>) -> Self {
        Self {
            pattern,
            form: Form::new(FormKind::Blank, Span::default()),
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

                let mut child = Draft::new(resolved);
                let consumed = child.build(source, offset);

                match child.record {
                    Record::Aligned | Record::Failed => {
                        self.record = child.record;
                        self.form = Form::new(
                            FormKind::Multiple(vec![child.form.clone()]),
                            Span::default(),
                        );
                        self.children.push(child);
                        consumed
                    }
                    _ => {
                        self.record = Record::Blank;
                        offset
                    }
                }
            }

            PatternKind::Wrapper(pattern) => {
                let mut child = Draft::new((**pattern).clone());
                let consumed = child.build(source, offset);

                match child.record {
                    Record::Aligned | Record::Failed => {
                        self.record = child.record;
                        self.form = child.form.clone();
                        self.children.push(child);
                        consumed
                    }
                    _ => {
                        self.record = Record::Blank;
                        offset
                    }
                }
            }

            PatternKind::Guard { predicate: function, pattern } => {
                let mut guard = function.lock().unwrap();
                let predicate = guard(source);

                drop(guard);

                if predicate {
                    let mut child = Draft::new((**pattern).clone());
                    let consumed = child.build(source, offset);

                    match child.record {
                        Record::Aligned | Record::Failed => {
                            self.record = child.record;
                            self.form = child.form.clone();
                            self.children.push(child);
                            consumed
                        }
                        _ => {
                            self.record = Record::Blank;
                            offset
                        }
                    }
                } else {
                    self.record = Record::Blank;
                    offset
                }
            }

            PatternKind::Literal(expect) => {
                if let Some(peek) = source.peek_ahead(offset) {
                    if *peek == *expect {
                        self.record = Record::Aligned;
                        self.form = Form::new(FormKind::Input(expect.clone()), Span::default());
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

            PatternKind::Alternative(patterns) => {
                let mut failed = None;

                for pattern in patterns {
                    let mut child = Draft::new(pattern.clone());
                    let consumed = child.build(source, offset);

                    match child.record {
                        Record::Aligned => {
                            self.record = Record::Aligned;
                            self.form = child.form.clone();
                            self.children.push(child);
                            return consumed;
                        }
                        Record::Failed => {
                            if failed.is_none() {
                                failed = Some((child, consumed));
                            }
                        }
                        _ => {}
                    }
                }

                if let Some((child, consumed)) = failed {
                    self.record = Record::Failed;
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
                let mut forms = Vec::with_capacity(sequence.len());
                self.record = Record::Aligned;
                self.children.reserve(sequence.len());

                for pattern in sequence {
                    let mut child = Draft::new(pattern.clone());
                    let consumed = child.build(source, current);

                    match child.record {
                        Record::Aligned => {
                            current = consumed;
                            forms.push(child.form.clone());
                            self.children.push(child);
                        }
                        Record::Failed => {
                            self.record = Record::Failed;
                            current = consumed;
                            forms.push(child.form.clone());
                            self.children.push(child);
                            break;
                        }
                        _ => {
                            self.record = Record::Blank;
                            current = offset;
                            if !forms.is_empty() {
                                self.form = Form::new(FormKind::Multiple(forms), Span::default());
                            }
                            break;
                        }
                    }
                }

                current
            }

            PatternKind::Repetition { pattern, minimum, maximum } => {
                let mut count = 0;
                let mut current = offset;
                let mut forms = Vec::new();

                while source.peek_ahead(current).is_some() {
                    let mut child = Draft::new((**pattern).clone());
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
                    self.record = Record::Aligned;
                    self.form = Form::new(FormKind::Multiple(forms), Span::default());
                    current
                } else {
                    self.record = Record::Blank;
                    offset
                }
            }

            PatternKind::Optional(pattern) => {
                let mut child = Draft::new((**pattern).clone());
                let consumed = child.build(source, offset);

                match child.record {
                    Record::Aligned | Record::Failed => {
                        self.record = child.record;
                        self.form = child.form.clone();
                        self.children.push(child);
                        consumed
                    }
                    _ => {
                        self.record = Record::Aligned;
                        offset
                    }
                }
            }

            PatternKind::Condition(function) => {
                if let Some(peek) = source.peek_ahead(offset) {
                    let mut guard = function.lock().unwrap();
                    let result = guard(peek);
                    drop(guard);

                    if result {
                        self.record = Record::Aligned;
                        self.form = Form::new(FormKind::Input(peek.clone()), Span::default());
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
                    let mut child = Draft::new((**pattern).clone());
                    child.build(source, offset);

                    if child.record != Record::Aligned {
                        self.record = Record::Aligned;
                        if let Some(peek) = source.peek_ahead(offset) {
                            self.form = Form::new(FormKind::Input(peek.clone()), Span::default());
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
                    self.record = Record::Aligned;
                    self.form = Form::new(FormKind::Input(peek.clone()), Span::default());
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

        match &self.pattern.kind {
            PatternKind::Literal(_)
            | PatternKind::Condition(_)
            | PatternKind::Negation(_)
            | PatternKind::WildCard => {
                if let Some(input) = source.next() {
                    let end = source.position();

                    self.form = Form::new(FormKind::Input(input), Span::new(start, end));
                }
            }

            PatternKind::Alternative(_)
            | PatternKind::Optional(_)
            | PatternKind::Guard { .. }
            | PatternKind::Deferred(_)
            | PatternKind::Wrapper(_) => {
                for child in &mut self.children {
                    child.realize(source);
                }

                if let Some(first) = self.children.first() {
                    self.form = first.form.clone();
                } else {
                    println!("Fuck :{:?}\n---{:?}", self.form, self.pattern);
                }
            }

            PatternKind::Sequence(_) | PatternKind::Repetition { .. } => {
                for child in &mut self.children {
                    child.realize(source);
                }

                let forms: Vec<_> = self.children.iter().map(|draft| draft.form.clone()).collect();

                let end = source.position();

                self.form = Form::new(FormKind::Multiple(forms), Span::new(start, end));
            }
        }
        
        if let Some(action) = &self.pattern.action.clone() {
            match self.record {
                Record::Failed => {
                    if matches!(action, Action::Failure(_)) {
                        action.apply(source, self);
                    }
                }

                _ => {
                    if action.is_applicable() {
                        action.apply(source, self);
                    }
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
        let mut draft = Draft::new(pattern);

        draft.build(self, 0);
        draft.realize(self);

        draft.form
    }
}