use {
    crate::{
        hash::Hash,
        format::Debug,
        axo_cursor::Span,
    }
};

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum FormKind<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    Blank,
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
        Self { kind: form, span, }
    }

    pub fn catch(&self) -> Vec<Form<Input, Output, Failure>> {
        let mut result = Vec::new();
        
        match self.kind.clone() {
            FormKind::Multiple(forms) => {
                for form in forms {
                    let errs = Self::catch(&form);
                    
                    if !errs.is_empty() {
                        result.extend(errs);
                    }
                }
            }

            FormKind::Failure(_) => {
                result.push(self.clone());
            }

            _ => {},
        }

        result
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

    pub fn map<MappedI, MappedO, MappedF, F, G, H>(
        self,
        input_mapper: F,
        output_mapper: G,
        error_mapper: H,
    ) -> Form<MappedI, MappedO, MappedF>
    where
        MappedI: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
        MappedO: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
        MappedF: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
        F: Fn(Input) -> MappedI + Clone,
        G: Fn(Output) -> MappedO + Clone,
        H: Fn(Failure) -> MappedF + Clone,
    {
        let mapped = match self.kind {
            FormKind::Blank => FormKind::Blank,
            FormKind::Input(input) => FormKind::Input(input_mapper(input)),
            FormKind::Output(output) => FormKind::Output(output_mapper(output)),
            FormKind::Multiple(forms) => {
                let mapped_forms = forms
                    .into_iter()
                    .map(|form| form.map(input_mapper.clone(), output_mapper.clone(), error_mapper.clone()))
                    .collect();
                FormKind::Multiple(mapped_forms)
            }
            FormKind::Failure(error) => FormKind::Failure(error_mapper(error)),
        };

        Form::new(mapped, self.span)
    }
}