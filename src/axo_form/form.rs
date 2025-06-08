use {
    log::{debug, warn},
    
    crate::{
        hash::Hash,
        format::Debug,
        axo_span::Span,
    }
};

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum FormKind<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    Empty,
    Input(Input),
    Output(Output),
    Multiple(Vec<Form<Input, Output, Failure>>),
    Failure(Failure),
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Form<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub kind: FormKind<Input, Output, Failure>,
    pub span: Span,
}

impl<Input, Output, Failure> Form<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub fn new(form: FormKind<Input, Output, Failure>, span: Span) -> Self {
        debug!("creating form with span {:?}", span);
        Self { kind: form, span, }
    }

    pub fn catch(&self) -> Option<Form<Input, Output, Failure>> {
        match self.kind.clone() {
            FormKind::Multiple(forms) => {
                for form in forms {
                    if let Some(error_form) = Self::catch(&form) {
                        warn!("error form detected in multiple forms, propagating upward");
                        return Some(error_form);
                    }
                }
            }

            FormKind::Failure(_) => {
                warn!("caught error form, returning for propagation");
                return Some(self.clone());
            }

            _ => {},
        }

        None
    }

    pub fn unwrap(&self) -> Vec<Form<Input, Output, Failure>> {
        match self.kind.clone() {
            FormKind::Multiple(forms) => forms,
            _ => vec![self.clone()],
        }
    }

    pub fn unwrap_input(&self) -> Option<Input> {
        match self.kind.clone() {
            FormKind::Input(input) => Some(input.clone()),
            _ => None
        }
    }

    pub fn unwrap_output(&self) -> Option<Output> {
        match self.kind.clone() {
            FormKind::Output(output) => Some(output.clone()),
            _ => None
        }
    }

    pub fn expand(&self) -> Vec<Form<Input, Output, Failure>> {
        let mut expanded: Vec<Form<Input, Output, Failure>> = Vec::new();

        match self {
            Form { kind: FormKind::Multiple(forms), .. } => {
                debug!("expanding {} nested forms recursively", forms.len());
                expanded.extend(Self::expand_forms(forms.clone()));
            }

            form => {
                expanded.push(form.clone());
            }
        }

        expanded
    }

    pub fn inputs(&self) -> Vec<Input> {
        let mut inputs: Vec<Input> = Vec::new();

        for form in self.unwrap() {
            match form.kind {
                FormKind::Input(input) => {
                    inputs.push(input);
                }
                FormKind::Multiple(sub) => {
                    let sub = Self::expand_inputs(sub);
                    inputs.extend(sub);
                }
                _ => {}
            }
        }

        debug!("extracted {} inputs from form structure", inputs.len());
        inputs
    }

    pub fn outputs(&self) -> Vec<Output> {
        let mut outputs: Vec<Output> = Vec::new();

        for form in self.unwrap() {
            match form.kind {
                FormKind::Output(output) => {
                    outputs.push(output);
                }
                FormKind::Multiple(sub) => {
                    let sub = Self::expand_outputs(sub);
                    outputs.extend(sub);
                }
                _ => {}
            }
        }

        debug!("extracted {} outputs from form structure", outputs.len());
        outputs
    }

    pub fn expand_forms(forms: Vec<Form<Input, Output, Failure>>) -> Vec<Form<Input, Output, Failure>> {
        let mut expanded: Vec<Form<Input, Output, Failure>> = Vec::new();

        for form in forms {
            match form {
                Form { kind: FormKind::Multiple(sub), .. } => {
                    let sub = Self::expand_forms(sub);
                    expanded.extend(sub);
                }
                form => {
                    expanded.push(form)
                }
            }
        }

        expanded
    }

    pub fn expand_inputs(forms: Vec<Form<Input, Output, Failure>>) -> Vec<Input> {
        let mut inputs: Vec<Input> = Vec::new();

        for form in forms {
            match form.kind {
                FormKind::Input(input) => {
                    inputs.push(input);
                }
                FormKind::Multiple(sub) => {
                    let sub = Self::expand_inputs(sub);
                    inputs.extend(sub);
                }
                _ => {}
            }
        }

        inputs
    }

    pub fn expand_outputs(forms: Vec<Form<Input, Output, Failure>>) -> Vec<Output> {
        let mut outputs: Vec<Output> = Vec::new();

        for form in forms {
            match form.kind {
                FormKind::Output(output) => {
                    outputs.push(output);
                }
                FormKind::Multiple(sub) => {
                    let sub = Self::expand_outputs(sub);
                    outputs.extend(sub);
                }
                _ => {}
            }
        }

        outputs
    }

    pub fn map<NewInput, NewOutput, NewError, F, G, H>(
        self,
        input_mapper: F,
        output_mapper: G,
        error_mapper: H,
    ) -> Form<NewInput, NewOutput, NewError>
    where
        NewInput: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
        NewOutput: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
        NewError: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
        F: Fn(Input) -> NewInput + Clone,
        G: Fn(Output) -> NewOutput + Clone,
        H: Fn(Failure) -> NewError + Clone,
    {
        let mapped_kind = match self.kind {
            FormKind::Empty => FormKind::Empty,
            FormKind::Input(input) => FormKind::Input(input_mapper(input)),
            FormKind::Output(output) => FormKind::Output(output_mapper(output)),
            FormKind::Multiple(forms) => {
                debug!("mapping {} forms to new types", forms.len());
                let mapped_forms = forms
                    .into_iter()
                    .map(|form| form.map(input_mapper.clone(), output_mapper.clone(), error_mapper.clone()))
                    .collect();
                FormKind::Multiple(mapped_forms)
            }
            FormKind::Failure(error) => FormKind::Failure(error_mapper(error)),
        };

        Form::new(mapped_kind, self.span)
    }
}