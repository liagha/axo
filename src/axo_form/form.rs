use {
    crate::{
        hash::Hash,
        format::Debug,
    }
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Form<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    Blank,
    Input(Input),
    Output(Output),
    Multiple(Vec<Form<Input, Output, Failure>>),
    Failure(Failure),
}

impl<Input, Output, Failure> Form<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    pub fn blank() -> Self {
        Form::Blank
    }

    pub fn input(input: Input) -> Self {
        Form::Input(input.clone())
    }

    pub fn output(output: Output) -> Self {
        Form::Output(output.clone())
    }

    pub fn multiple(forms: Vec<Form<Input, Output, Failure>>) -> Self {
        if forms.is_empty() {
            Form::Blank
        } else {
            Form::Multiple(forms.clone())
        }
    }

    pub fn get_input(&self) -> Option<Input> {
        match self.clone() {
            Form::Input(input) => Some(input.clone()),
            _ => None
        }
    }

    pub fn get_output(&self) -> Option<Output> {
        match self.clone() {
            Form::Output(output) => Some(output.clone()),
            _ => None
        }
    }

    pub fn unwrap(&self) -> Vec<Form<Input, Output, Failure>> {
        match self.clone() {
            Form::Multiple(forms) => forms,
            _ => vec![self.clone()],
        }
    }

    #[track_caller]
    pub fn unwrap_input(&self) -> Input {
        match self.clone() {
            Form::Input(input) => input.clone(),
            _ => panic!("the form isn't an input!")
        }
    }

    #[track_caller]
    pub fn unwrap_output(&self) -> Output {
        match self.clone() {
            Form::Output(output) => output.clone(),
            _ => panic!("the form isn't an output!")
        }
    }

    pub fn expand(&self) -> Vec<Form<Input, Output, Failure>> {
        let mut expanded: Vec<Form<Input, Output, Failure>> = Vec::new();

        match self {
            Form::Multiple(forms) => {
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
            match form {
                Form::Input(input) => {
                    inputs.push(input);
                }
                Form::Multiple(sub) => {
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
            match form {
                Form::Output(output) => {
                    outputs.push(output);
                }
                Form::Multiple(sub) => {
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
                Form::Multiple(sub) => {
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
            match form {
                Form::Input(input) => {
                    inputs.push(input);
                }
                Form::Multiple(sub) => {
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
            match form {
                Form::Output(output) => {
                    outputs.push(output);
                }
                Form::Multiple(sub) => {
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
        MappedI: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
        MappedO: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
        MappedF: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
        F: Fn(Input) -> MappedI + Clone,
        G: Fn(Output) -> MappedO + Clone,
        H: Fn(Failure) -> MappedF + Clone,
    {
        let mapped = match self {
            Form::Blank => Form::Blank,
            Form::Input(input) => Form::Input(input_mapper(input)),
            Form::Output(output) => Form::Output(output_mapper(output)),
            Form::Multiple(forms) => {
                let forms = forms
                    .into_iter()
                    .map(|form| form.map(input_mapper.clone(), output_mapper.clone(), error_mapper.clone()))
                    .collect();
                
                Form::Multiple(forms)
            }
            Form::Failure(error) => Form::Failure(error_mapper(error)),
        };

        mapped
    }
}