use {
    crate::{
        slice,
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

impl<Input, Output, Failure> Default for Form<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::Blank
    }
}

impl<Input, Output, Failure> Form<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    #[inline(always)]
    pub fn blank() -> Self {
        Form::Blank
    }

    #[inline(always)]
    pub fn input(input: Input) -> Self {
        Form::Input(input.clone())
    }

    #[inline(always)]
    pub fn output(output: Output) -> Self {
        Form::Output(output.clone())
    }

    #[inline(always)]
    pub fn multiple(forms: Vec<Form<Input, Output, Failure>>) -> Self {
        if forms.is_empty() {
            Form::Blank
        } else {
            Form::Multiple(forms.clone())
        }
    }

    #[inline(always)]
    pub fn failure(failure: Failure) -> Self {
        Form::Failure(failure)
    }

    #[inline(always)]
    pub fn get_input(&self) -> Option<Input> {
        match self.clone() {
            Form::Input(input) => Some(input.clone()),
            _ => None
        }
    }

    #[inline(always)]
    pub fn get_output(&self) -> Option<Output> {
        match self.clone() {
            Form::Output(output) => Some(output.clone()),
            _ => None
        }
    }

    #[inline(always)]
    pub fn get_failure(&self) -> Option<&Failure> {
        match self {
            Form::Failure(failure) => Some(failure),
            _ => None
        }
    }

    #[inline]
    pub fn is_blank(&self) -> bool {
        matches!(self, Form::Blank)
    }

    #[inline]
    pub fn is_input(&self) -> bool {
        matches!(self, Form::Input(_))
    }

    #[inline]
    pub fn is_output(&self) -> bool {
        matches!(self, Form::Output(_))
    }

    #[inline]
    pub fn is_failure(&self) -> bool {
        matches!(self, Form::Failure(_))
    }

    #[inline]
    pub fn is_multiple(&self) -> bool {
        matches!(self, Form::Multiple(_))
    }

    #[inline(always)]
    pub fn as_forms(&self) -> &[Form<Input, Output, Failure>] {
        match self {
            Form::Multiple(forms) => forms.as_slice(),
            _ => slice::from_ref(self),
        }
    }

    #[track_caller]
    pub fn unwrap_input(&self) -> &Input {
        match self {
            Form::Input(input) => input,
            _ => panic!("called `Form::unwrap_input()` on a non-Input value")
        }
    }

    #[track_caller]
    pub fn unwrap_output(&self) -> &Output {
        match self {
            Form::Output(output) => output,
            _ => panic!("called `Form::unwrap_output()` on a non-Output value")
        }
    }

    #[track_caller]
    pub fn unwrap_failure(&self) -> &Failure {
        match self {
            Form::Failure(failure) => failure,
            _ => panic!("called `Form::unwrap_failure()` on a non-Failure value")
        }
    }

    pub fn flatten(&self) -> Vec<Form<Input, Output, Failure>> {
        let mut result = Vec::new();
        self.flatten_into(&mut result);
        result
    }

    fn flatten_into(&self, result: &mut Vec<Form<Input, Output, Failure>>) {
        match self {
            Form::Multiple(forms) => {
                for form in forms {
                    form.flatten_into(result);
                }
            }
            form => result.push(form.clone()),
        }
    }

    pub fn collect_inputs(&self) -> Vec<Input> {
        let mut inputs = Vec::new();
        self.collect_inputs_into(&mut inputs);
        inputs
    }

    fn collect_inputs_into(&self, inputs: &mut Vec<Input>) {
        match self {
            Form::Input(input) => inputs.push(input.clone()),
            Form::Multiple(forms) => {
                for form in forms {
                    form.collect_inputs_into(inputs);
                }
            }
            _ => {}
        }
    }

    pub fn collect_outputs(&self) -> Vec<Output> {
        let mut outputs = Vec::new();
        self.collect_outputs_into(&mut outputs);
        outputs
    }

    fn collect_outputs_into(&self, outputs: &mut Vec<Output>) {
        match self {
            Form::Output(output) => outputs.push(output.clone()),
            Form::Multiple(forms) => {
                for form in forms {
                    form.collect_outputs_into(outputs);
                }
            }
            _ => {}
        }
    }

    pub fn collect_failures(&self) -> Vec<Failure> {
        let mut failures = Vec::new();
        self.collect_failures_into(&mut failures);
        failures
    }

    fn collect_failures_into(&self, failures: &mut Vec<Failure>) {
        match self {
            Form::Failure(failure) => failures.push(failure.clone()),
            Form::Multiple(forms) => {
                for form in forms {
                    form.collect_failures_into(failures);
                }
            }
            _ => {}
        }
    }

    #[inline(always)]
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