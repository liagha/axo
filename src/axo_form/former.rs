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
use crate::format_vec;

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

    fn build<Source>(
        &mut self,
        source: &mut Source,
        offset: usize
    ) -> (bool, usize)
    where
        Source: Peekable<Input> + Marked,
    {
        let start = source.position();
        debug!("building pattern {} at offset {} (position: {})", self.pattern, offset, start);

        let result = match self.pattern.kind.clone() {
            PatternKind::Deferred(factory) => {
                debug!("resolving deferred pattern at offset {}", offset);

                let mut guard = factory.lock().unwrap();
                let resolved = guard();

                debug!("deferred pattern resolved to: {}", resolved);

                let mut child = Draft::new(resolved, &start);

                let (matches, consumed) = child.build(source, offset);

                if matches {
                    debug!("deferred pattern matched, child form: {}", child.form);

                    self.children.push(child.clone());
                    self.form = Form::new(FormKind::Multiple(vec![child.form.clone()]), Span::point(start));

                    trace!("deferred pattern final form: {}", self.form);
                } else {
                    debug!("deferred pattern failed to match");
                }

                (matches, consumed)
            }

            PatternKind::Wrap(ref pattern) => {
                debug!("building wrap pattern with inner pattern: {}", pattern);

                let mut child = Draft::new(*pattern.clone(), &start);
                let (matches, consumed) = child.build(source, offset);

                if matches {
                    debug!("wrap inner pattern matched, form: {}", child.form);

                    self.children.push(child);
                    self.form = self.children[0].form.clone();

                    trace!("wrap pattern final form: {}", self.form);
                } else {
                    debug!("wrap inner pattern failed to match");
                }

                (matches, consumed)
            }

            PatternKind::Guard { predicate, pattern } => {
                debug!("evaluating guard pattern with inner pattern: {}", pattern);
                let mut guard = predicate.lock().unwrap();

                if guard(source) {
                    debug!("guard condition passed, proceeding with inner pattern");
                    let mut child = Draft::new(*pattern, &start);
                    let (matches, consumed) = child.build(source, offset);

                    if matches {
                        debug!("guard inner pattern matched, form: {}", child.form);
                        self.children.push(child);
                        self.form = self.children[0].form.clone();
                        trace!("guard pattern final form: {}", self.form);
                    } else {
                        debug!("guard inner pattern failed to match");
                    }

                    (matches, consumed)
                } else {
                    debug!("guard condition failed");
                    (false, offset)
                }
            }

            PatternKind::Literal(ref expect) => {
                debug!("matching literal pattern: {:?}", expect);

                if let Some(peek) = source.peek_ahead(offset) {
                    let matches = *peek == *expect;
                    debug!("literal match attempt: expected {:?}, found {:?}, matches: {}", expect, peek, matches);
                    if matches {
                        self.form = Form::new(FormKind::Input(expect.clone()), Span::point(start));
                        trace!("literal pattern matched, form: {}", self.form);
                    }
                    (matches, if matches { offset + 1 } else { offset })
                } else {
                    debug!("literal pattern failed: no input at offset {}", offset);
                    (false, offset)
                }
            }

            PatternKind::Alternative(ref patterns) => {
                debug!("building alternative tree with {} options at offset {}, patterns: {}", patterns.len(), offset, format_vec(patterns));

                for (index, subpattern) in patterns.iter().enumerate() {
                    debug!("trying alternative {} of {}: {}", index + 1, patterns.len(), subpattern);
                    let mut child = Draft::new(subpattern.clone(), &start);
                    let (matches, new_offset) = child.build(source, offset);

                    if matches {
                        debug!("alternative {} matched successfully, form: {}", index, child.form);
                        self.children.push(child);
                        self.form = self.children[0].form.clone();
                        trace!("alternative pattern final form: {}", self.form);
                        return (true, new_offset);
                    } else {
                        debug!("alternative {} failed", index);
                    }
                }

                debug!("all alternatives failed");
                (false, offset)
            }

            PatternKind::Sequence(ref sequence) => {
                debug!("building sequence tree with {} elements at offset {}, sequence: {}", sequence.len(), offset, format_vec(sequence));
                let mut current = offset;
                let mut forms = Vec::new();

                for (index, pattern) in sequence.iter().enumerate() {
                    debug!("processing sequence element {} of {}: {}", index + 1, sequence.len(), pattern);
                    let mut child = Draft::new(pattern.clone(), &start);
                    let (matches, new) = child.build(source, current);

                    if matches {
                        debug!("sequence element {} matched, form: {}", index, child.form);
                        current = new;
                        forms.push(child.form.clone());
                        self.children.push(child);
                    } else {
                        debug!("sequence element {} failed at position {}, pattern: {}", index, current, pattern);
                        return (false, offset);
                    }
                }

                if !forms.is_empty() {
                    self.form = Form::new(FormKind::Multiple(forms), Span::point(start));
                    trace!("sequence pattern final form: {}", self.form);
                }

                debug!("sequence completed successfully with {} elements", sequence.len());
                (true, current)
            }

            PatternKind::Repetition { ref pattern, minimum, maximum } => {
                debug!("building repetition tree (min: {}, max: {:?}) at offset {}, pattern: {}", minimum, maximum, offset, pattern);
                let mut current = offset;
                let mut count = 0;
                let mut forms = Vec::new();

                while source.peek_ahead(current).is_some() {
                    debug!("repetition attempt {} (current offset: {})", count + 1, current);
                    let mut child = Draft::new(*pattern.clone(), &start);
                    let (matches, new) = child.build(source, current);

                    if !matches || new == current {
                        debug!("repetition stopped: matches={}, progress={}", matches, new != current);
                        break;
                    }

                    count += 1;
                    current = new;
                    debug!("repetition {} succeeded, form: {}", count, child.form);
                    forms.push(child.form.clone());
                    self.children.push(child);

                    if let Some(max) = maximum {
                        if count >= max {
                            debug!("repetition reached maximum count of {}", max);
                            break;
                        }
                    }
                }

                let success = count >= minimum;
                if success && !forms.is_empty() {
                    self.form = Form::new(FormKind::Multiple(forms), Span::point(start));
                    trace!("repetition pattern final form: {}", self.form);
                }

                debug!("repetition matched {} times, meets minimum requirement ({}): {}", count, minimum, success);
                (success, if success { current } else { offset })
            }

            PatternKind::Optional(ref pattern) => {
                debug!("building optional pattern: {}", pattern);
                let mut child = Draft::new(*pattern.clone(), &start);
                let (matches, new) = child.build(source, offset);

                if matches {
                    debug!("optional pattern matched, form: {}", child.form);
                    self.children.push(child);
                    self.form = self.children[0].form.clone();
                    trace!("optional pattern final form: {}", self.form);
                } else {
                    debug!("optional pattern didn't match, but that's okay");
                }

                (true, if matches { new } else { offset })
            }

            PatternKind::Condition(function) => {
                debug!("evaluating condition pattern at offset {}", offset);
                if let Some(peek) = source.peek_ahead(offset) {
                    let mut guard = function.lock().unwrap();

                    let result = guard(&peek);
                    debug!("condition evaluation: input {:?}, result: {}", peek, result);

                    if result {
                        self.form = Form::new(FormKind::Input(peek.clone()), Span::point(start));
                        trace!("condition pattern matched, form: {}", self.form);
                    }
                    (result, if result { offset + 1 } else { offset })
                } else {
                    debug!("condition pattern failed: no input at offset {}", offset);
                    (false, offset)
                }
            }

            PatternKind::Negation(ref pattern) => {
                debug!("evaluating negation pattern: {}", pattern);
                if source.peek_ahead(offset).is_none() {
                    debug!("negation failed: no input to negate");
                    return (false, offset);
                }

                let mut child = Draft::new(*pattern.clone(), &start);
                let (matches, _) = child.build(source, offset);
                let result = !matches;
                debug!("negation: inner pattern matched={}, negation result={}", matches, result);

                if result {
                    if let Some(peek) = source.peek_ahead(offset) {
                        self.form = Form::new(FormKind::Input(peek.clone()), Span::point(start));
                        trace!("negation pattern matched, form: {}", self.form);
                    }
                }

                (result, if result { offset + 1 } else { offset })
            }

            PatternKind::WildCard => {
                debug!("matching wildcard pattern at offset {}", offset);
                if let Some(peek) = source.peek_ahead(offset) {
                    debug!("wildcard matched input: {:?}", peek);
                    self.form = Form::new(FormKind::Input(peek.clone()), Span::point(start));
                    trace!("wildcard pattern form: {}", self.form);
                    (true, offset + 1)
                } else {
                    debug!("wildcard failed: no input at offset {}", offset);
                    (false, offset)
                }
            }

        };

        debug!("pattern build result: matches={}, consumed={}, final form: {}", result.0, result.1, self.form);
        result
    }

    fn realize<Source>(&mut self, source: &mut Source) -> Form<Input, Output, Failure>
    where
        Source: Peekable<Input> + Marked,
    {
        let start = source.position();
        debug!("realizing pattern {} at position {}", self.pattern, start);

        match self.pattern.kind.clone() {
            PatternKind::Literal(_) => {
                if let Some(input) = source.next() {
                    let end = source.position();
                    self.form = Form::new(FormKind::Input(input), Span::new(start, end));
                    debug!("literal realization complete, form: {}", self.form);
                }
            }

            PatternKind::Condition(_) => {
                if let Some(input) = source.next() {
                    let end = source.position();
                    self.form = Form::new(FormKind::Input(input), Span::new(start, end));
                    debug!("condition realization complete, form: {}", self.form);
                }
            }

            PatternKind::Negation(_) => {
                if let Some(input) = source.next() {
                    let end = source.position();
                    self.form = Form::new(FormKind::Input(input), Span::new(start, end));
                    debug!("negation realization complete, form: {}", self.form);
                }
            }

            PatternKind::WildCard => {
                if let Some(input) = source.next() {
                    let end = source.position();
                    self.form = Form::new(FormKind::Input(input), Span::new(start, end));
                    debug!("wildcard realization complete, form: {}", self.form);
                }
            }

            _ => {
                debug!("realizing {} children for composite pattern", self.children.len());

                for (i, child) in self.children.iter_mut().enumerate() {
                    debug!("realizing child {} with pattern: {}", i, child.pattern);
                    child.realize(source);
                    debug!("child {} realized with form: {}", i, child.form);
                }

                let end = source.position();
                let span = Span::new(start, end);

                match self.pattern.kind.clone() {
                    PatternKind::Sequence(_) | PatternKind::Repetition { .. } => {
                        let forms: Vec<_> = self.children.iter().map(|c| c.form.clone()).collect();
                        if !forms.is_empty() {
                            self.form = Form::new(FormKind::Multiple(forms), span);
                            debug!("composite pattern (sequence/repetition) realized, form: {}", self.form);
                        }
                    }

                    PatternKind::Alternative(_) | PatternKind::Optional(_) |
                    PatternKind::Guard { .. } | PatternKind::Deferred(_) | PatternKind::Wrap(_) => {
                        if !self.children.is_empty() {
                            self.form = self.children[0].form.clone();
                            debug!("composite pattern (alternative/optional/guard/deferred/wrap) realized, form: {}", self.form);
                        }
                    }

                    _ => {}
                }
            }
        }

        if let Some(ref action) = self.pattern.action {
            debug!("applying action to form: action={}, current_form={}", action, self.form);
            
            let span = self.form.span.clone();
            
            self.form = source.action(action, self.form.clone(), span);
            
            debug!("action applied, new form: {}", self.form);
        }

        debug!("pattern realization completed, final form: {}", self.form);
        
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
        debug!("processing action: {} on form: {} at span: {}", action, form, span);

        if let Some(err) = form.catch() {
            warn!("caught error in form before action processing: {}, returning early", err);
            return err;
        }

        let result = match action {
            Action::Map(transform) => {
                debug!("applying map transformation at span {} to form: {}", span, form);

                let mut guard = transform.lock().unwrap();
                let context = &mut self.context_mut();

                match guard(context, form.clone()) {
                    Ok(output) => {
                        let new_form = Form::new(FormKind::Output(output), span);
                        debug!("map transformation successful, new form: {}", new_form);
                        new_form
                    },
                    Err(error) => {
                        warn!("transformation failed with error: {:?}, returning failure form", error);
                        Form::new(FormKind::Failure(error), span)
                    },
                }
            },

            Action::Execute(executor) => {
                debug!("executing action at span {} with form: {}", span, form);

                let mut guard = executor.lock().unwrap();
                guard();

                debug!("execution completed, returning original form");
                form.clone()
            },

            Action::Multiple(actions) => {
                debug!("processing multiple actions ({} total) at span {} with form: {}", actions.len(), span, form);

                let mut current_form = form.clone();

                for (index, action) in actions.iter().enumerate() {
                    debug!("processing action {} of {}: {}", index + 1, actions.len(), action);
                    current_form = self.action(action, current_form, span.clone());

                    if let Some(err) = current_form.catch() {
                        warn!("action {} failed with error: {}, short-circuiting multiple actions", index + 1, err);
                        return err;
                    }
                }

                debug!("all {} actions completed successfully, final form: {}", actions.len(), current_form);
                current_form
            },

            Action::Trigger { found, missing } => {
                debug!("evaluating trigger condition at span {} with form: {}", span, form);

                let has_content = match &form.kind {
                    FormKind::Empty => false,
                    FormKind::Failure(_) => false,
                    FormKind::Input(_) | FormKind::Output(_) | FormKind::Multiple(_) => true,
                };

                let chosen_action = if has_content { found } else { missing };
                debug!("trigger choosing {} action based on content presence: {}",
                   if has_content { "found" } else { "missing" }, has_content);

                self.action(chosen_action, form, span)
            },

            Action::Ignore => {
                debug!("ignoring form content: {} at span {}", form, span);
                Form::new(FormKind::Empty, span)
            },

            Action::Capture { identifier } => {
                debug!("forming capture pattern with identifier: {}, form: {}", identifier, form);

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

                debug!("inserting captured item: {}", item);
                resolver.insert(item);

                form.clone()
            },

            Action::Failure(function) => {
                warn!("creating error form with function at span {}, input form: {}", span, form);

                let mut guard = function.lock().unwrap();
                let error_form = Form::new(FormKind::Failure(guard(span.clone())), span);
                debug!("error form created: {}", error_form);
                error_form
            },
            Action::Inspect(inspector) => {
                debug!("inspecting action at span {} with form: {}", span, form);

                let mut guard = inspector.lock().unwrap();
                guard(form.clone());

                debug!("inspection completed, returning original form");
                
                form.clone()
            },
        };

        debug!("action processing complete, result form: {}", result);
        
        result
    }

    fn fit(&mut self, pattern: &Pattern<Input, Output, Failure>, offset: usize) -> (bool, usize) {
        let start = self.position();
        
        debug!("fitting pattern: {} at offset {} (position: {})", pattern, offset, start);

        let mut draft = Draft::new(pattern.clone(), &start);
        let result = draft.build(self, offset);

        debug!("fit result for pattern: matches={}, consumed={}", result.0, result.1);
        
        result
    }

    fn form(&mut self, pattern: Pattern<Input, Output, Failure>) -> Form<Input, Output, Failure> {
        let start = self.position();

        debug!("forming pattern: {} at position {}", pattern, start);

        let mut draft = Draft::new(pattern, &start);
        let (matches, consumed) = draft.build(self, 0);
        
        debug!("pattern build phase: matches={}, consumed={}", matches, consumed);

        if !matches {
            debug!("pattern failed to match, returning empty form");
            
            return Form::new(FormKind::Empty, Span::point(start));
        }

        let form = draft.realize(self);
        
        debug!("pattern formation complete, final form: {}", form);
        
        form
    }
}