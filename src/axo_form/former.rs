use {
    super::{
        pattern::{Pattern, PatternKind},
        form::{Form, FormKind},
        order::Pulse,
    },
    crate::{
        axo_cursor::{
            Position, Peekable,
            Span, Spanned,
        },
        compiler::Marked,
        format::Debug,
        hash::Hash,
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
    pub stack: Vec<Draft<Input, Output, Failure>>,
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
            stack: Vec::new(),
            consumed: Vec::new(),
            record: Record::Blank,
            pattern,
            form: Form::new(FormKind::Blank, Span::point(position)),
        }
    }

    pub fn forge(&mut self) {
        if self.stack.len() == 0 {
            self.form = Form::blank(Span::point(self.position));
        } else if self.stack.len() == 1 {
            self.form = self.stack[0].clone().form;
        } else {
            let forms = self.stack.iter().map(|draft| draft.form.clone()).collect::<Vec<_>>();
            self.form = Form::multiple(forms);
        }
    }

    pub fn consumer<Source>(&mut self, source: &mut Source, peek: Input, pulses: Vec<Pulse>)
    where
        Source: Peekable<Input> + Marked,
    {
        for pulse in pulses {
            match pulse {
                Pulse::Escape => return,
                Pulse::Forge => {
                    self.forge()
                },
                Pulse::Imitate => {
                    self.form = Form::input(peek.clone())
                },
                Pulse::Feast => {
                    source.next(&mut self.marker, &mut self.position);
                    self.consumed.push(peek.clone());
                }
                Pulse::Align => self.record.align(),
                Pulse::Skip => {
                    self.record.skip();
                    self.form = Form::blank(Span::point(self.position));
                },
                Pulse::Ignore => {
                    self.form = Form::blank(Span::point(self.position));
                },
                Pulse::Fail => self.record.fail(),
                Pulse::Pardon => self.record.empty(),
                _ => {}
            }
        }
    }

    pub fn build<Source>(&mut self, source: &mut Source)
    where
        Source: Peekable<Input> + Marked,
    {
        match self.pattern.kind.clone() {
            PatternKind::Identical { value, align, miss } => {
                if let Some(peek) = source.get(self.marker).cloned() {
                    if value.eq(&peek) {
                        let pulses = align.execute(source, self);

                        self.consumer(source, peek, pulses);
                    } else {
                        miss.execute(source, self);
                    }
                } else {
                    miss.execute(source, self);
                }
            }

            PatternKind::Reject { pattern, align, miss } => {
                if let Some(peek) = source.get(self.marker).cloned() {
                    let mut draft = Draft::new(self.marker, self.position, *pattern);
                    draft.build(source);

                    if !draft.record.is_aligned() {
                        let pulses = align.execute(source, self);

                        self.consumer(source, peek, pulses);
                    } else {
                        miss.execute(source, self);
                    }
                } else {
                    miss.execute(source, self);
                }
            }

            PatternKind::Predicate { function, align, miss } => {
                if let Some(peek) = source.get(self.marker).cloned() {
                    let predicate = function(&peek);

                    if predicate {
                        let pulses = align.execute(source, self);

                        self.consumer(source, peek, pulses);
                    } else {
                        miss.execute(source, self);
                    }
                } else {
                    miss.execute(source, self);
                }
            }

            PatternKind::Alternative { patterns, order, finish } => {
                'outer : for pattern in patterns {
                    let mut child = Draft::new(self.marker, self.position, pattern);
                    child.build(source);

                    let pulses = order.execute(source, &mut child);

                    for pulse in pulses {
                        match pulse {
                            Pulse::Escape => return,
                            Pulse::Terminate => break 'outer,
                            Pulse::Proceed => continue 'outer,
                            Pulse::Forge => {
                                self.forge()
                            },
                            Pulse::Imitate => {
                                self.form = child.form.clone();
                            },
                            Pulse::Inject => {
                                self.stack.push(child.clone())
                            },
                            Pulse::Feast => {
                                self.marker = child.marker;
                                self.position = child.position;
                                self.consumed = child.consumed.clone();
                            }
                            Pulse::Align => self.record.align(),
                            Pulse::Skip => {
                                self.record.skip();
                                child.form = Form::blank(Span::point(self.position));
                            },
                            Pulse::Ignore => {
                                child.form = Form::blank(Span::point(self.position));
                            },
                            Pulse::Fail => self.record.fail(),
                            Pulse::Pardon => self.record.empty(),
                        }
                    }
                }

                let pulses = finish.execute(source, self);

                for pulse in pulses {
                    match pulse {
                        Pulse::Escape => return,
                        Pulse::Forge => {
                            self.forge()
                        },
                        Pulse::Imitate => {
                            if let Some(fallback) = self.stack.first() {
                                self.form = fallback.form.clone();
                            }
                        },
                        Pulse::Feast => {
                            if let Some(fallback) = self.stack.first() {
                                self.marker = fallback.marker;
                                self.position = fallback.position;
                                self.consumed = fallback.consumed.clone();
                            }
                        }
                        Pulse::Align => self.record.align(),
                        Pulse::Skip => {
                            self.record.skip();

                            self.form = Form::blank(Span::point(self.position));
                        },
                        Pulse::Ignore => {
                            self.form = Form::blank(Span::point(self.position));
                        },
                        Pulse::Fail => self.record.fail(),
                        Pulse::Pardon => self.record.empty(),
                        _ => {}
                    }
                }
            }

            PatternKind::Deferred { function, order } => {
                let resolved = function();

                let mut child = Draft::new(self.marker, self.position, resolved);
                child.build(source);

                let pulses = order.execute(source, &mut child);

                for pulse in pulses {
                    match pulse {
                        Pulse::Escape => return,
                        Pulse::Forge => {
                            self.forge()
                        },
                        Pulse::Imitate => {
                            self.marker = child.marker;
                            self.position = child.position;
                            self.consumed = child.consumed.clone();
                            self.record = child.record;
                            self.form = child.form.clone();
                        },
                        Pulse::Inject => {
                            self.stack.push(child.clone())
                        },
                        Pulse::Feast => {
                            self.marker = child.marker;
                            self.position = child.position;
                            self.consumed = child.consumed.clone();
                        }
                        Pulse::Align => self.record.align(),
                        Pulse::Skip => {
                            self.record.skip();
                            self.form = Form::blank(Span::point(self.position));
                        },
                        Pulse::Ignore => {
                            self.form = Form::blank(Span::point(self.position));
                        },
                        Pulse::Fail => self.record.fail(),
                        Pulse::Pardon => self.record.empty(),
                        _ => {}
                    }
                }
            }

            PatternKind::Wrapper { pattern, order } => {
                let mut child = Draft::new(self.marker, self.position, *pattern);
                child.build(source);

                let pulses = order.execute(source, &mut child);

                for pulse in pulses {
                    match pulse {
                        Pulse::Escape => return,
                        Pulse::Forge => {
                            self.forge()
                        },
                        Pulse::Imitate => {
                            self.marker = child.marker;
                            self.position = child.position;
                            self.consumed = child.consumed.clone();
                            self.record = child.record.clone();
                            self.form = child.form.clone();
                        },
                        Pulse::Inject => {
                            self.stack.push(child.clone())
                        },
                        Pulse::Feast => {
                            self.marker = child.marker;
                            self.position = child.position;
                            self.consumed = child.consumed.clone();
                        }
                        Pulse::Align => self.record.align(),
                        Pulse::Skip => {
                            self.record.skip();
                            self.form = Form::blank(Span::point(self.position));
                        },
                        Pulse::Ignore => {
                            self.form = Form::blank(Span::point(self.position));
                        },
                        Pulse::Fail => self.record.fail(),
                        Pulse::Pardon => self.record.empty(),
                        _ => {}
                    }
                }
            }

            PatternKind::Sequence { patterns, order, finish } => {
                'outer : for pattern in patterns {
                    let mut child = Draft::new(self.marker, self.position, pattern);
                    child.build(source);

                    if child.marker == self.marker {
                        break;
                    }

                    let pulses = order.execute(source, &mut child);

                    for pulse in pulses {
                        match pulse {
                            Pulse::Escape => return,
                            Pulse::Terminate => break 'outer,
                            Pulse::Proceed => continue 'outer,
                            Pulse::Forge => {
                                self.forge()
                            },
                            Pulse::Imitate => {
                                self.form = child.form.clone();
                            },
                            Pulse::Inject => {
                                self.stack.push(child.clone())
                            },
                            Pulse::Feast => {
                                self.marker = child.marker;
                                self.position = child.position;
                                self.consumed = child.consumed.clone();
                            }
                            Pulse::Align => self.record.align(),
                            Pulse::Skip => {
                                self.record.skip();
                                child.form = Form::blank(Span::point(self.position));
                            },
                            Pulse::Ignore => {
                                child.form = Form::blank(Span::point(self.position));
                            },
                            Pulse::Fail => self.record.fail(),
                            Pulse::Pardon => self.record.empty(),
                        }
                    }
                }

                let pulses = finish.execute(source, self);

                for pulse in pulses {
                    match pulse {
                        Pulse::Escape => return,
                        Pulse::Forge => {
                            self.forge()
                        },
                        _ => {}
                    }
                }
            }

            PatternKind::Repetition {
                pattern,
                minimum,
                maximum,
                order, lack, exceed, finish,
            } => {
                let mut index = self.marker;
                let mut position = self.position;
                let mut consumed = Vec::new();

                'outer : while source.peek_ahead(index).is_some() {
                    let mut child = Draft::new(index, position, *pattern.clone());
                    child.build(source);

                    if child.marker == index {
                        break;
                    }

                    let pulses = order.execute(source, &mut child);

                    for pulse in pulses {
                        match pulse {
                            Pulse::Escape => return,
                            Pulse::Terminate => break 'outer,
                            Pulse::Proceed => continue 'outer,
                            Pulse::Forge => {
                                self.forge()
                            },
                            Pulse::Imitate => {
                                self.form = child.form.clone();
                            },
                            Pulse::Inject => {
                                self.stack.push(child.clone())
                            },
                            Pulse::Feast => {
                                index = child.marker;
                                position = child.position;
                                consumed.extend(child.consumed.clone());
                            }
                            Pulse::Align => self.record.align(),
                            Pulse::Skip => {
                                self.record.skip();
                                self.form = Form::blank(Span::point(self.position));
                            },
                            Pulse::Ignore => {
                                self.form = Form::blank(Span::point(self.position));
                            },
                            Pulse::Fail => self.record.fail(),
                            Pulse::Pardon => self.record.empty(),
                        }
                    }

                    if let Some(max) = maximum {
                        if self.stack.len() >= max {
                            let pulses = exceed.execute(source, &mut child);

                            for pulse in pulses {
                                match pulse {
                                    Pulse::Escape => return,
                                    Pulse::Terminate => break 'outer,
                                    Pulse::Proceed => continue 'outer,
                                    Pulse::Forge => {
                                        self.forge()
                                    },
                                    Pulse::Imitate => {
                                        self.form = child.form.clone();
                                    },
                                    Pulse::Inject => {
                                        self.stack.push(child.clone())
                                    },
                                    Pulse::Feast => {
                                        index = child.marker;
                                        position = child.position;
                                        consumed.extend(child.consumed.clone());
                                    }
                                    Pulse::Align => self.record.align(),
                                    Pulse::Skip => {
                                        self.record.skip();
                                        self.form = Form::blank(Span::point(self.position));
                                    },
                                    Pulse::Ignore => {
                                        self.form = Form::blank(Span::point(self.position));
                                    },
                                    Pulse::Fail => self.record.fail(),
                                    Pulse::Pardon => self.record.empty(),
                                }
                            }
                        }
                    }
                }

                if self.stack.len() >= minimum {
                    let pulses = finish.execute(source, self);

                    for pulse in pulses {
                        match pulse {
                            Pulse::Escape => return,
                            Pulse::Forge => {
                                self.forge()
                            },
                            Pulse::Feast => {
                                self.marker = index;
                                self.position = position;
                                self.consumed = consumed.clone();
                            }
                            _ => {}
                        }
                    }
                } else {
                    let pulses = lack.execute(source, self);

                    for pulse in pulses {
                        match pulse {
                            Pulse::Escape => return,
                            Pulse::Forge => {
                                self.forge()
                            },
                            Pulse::Feast => {
                                self.marker = index;
                                self.position = position;
                                self.consumed = consumed.clone();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if let Some(order) = &self.pattern.order.clone() {
            order.execute(source, self);
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