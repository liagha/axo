use {
    super::{
        order::Order,
        form::{Form},
        pattern::Classifier,
    },
    crate::{
        hash::Hash,
        vector::Show,
        format::{Debug, Display, Formatter, Result},
    },
};

impl<Input, Output, Failure> Debug for Classifier<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Todo")
    }
}


impl<Input, Output, Failure> Display for Form<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.clone() {
            Form::Blank => {
                write!(f, "Blank")
            }

            Form::Input(input) => {
                write!(f, "Input({:?})", input)
            }

            Form::Output(output) => {
                write!(f, "Output({:?})", output)
            }

            Form::Multiple(forms) => {
                write!(f, "Multiple({})", forms.format())
            }

            Form::Failure(error) => {
                write!(f, "Failure({:?})", error)
            }
        }
    }
}

impl<Input, Output, Failure> Debug for Order<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Order::Align => write!(f, "Align"),
            Order::Branch { found, missing } => write!(f, "Trigger({:?}, {:?})", found, missing),
            Order::Fail(_) => write!(f, "Failure"),
            Order::Ignore => write!(f, "Ignore"),
            Order::Inspect(_) => write!(f, "Inspector"),
            Order::Multiple(actions) => write!(f, "Multiple({:?})", actions),
            Order::Panic(_) => write!(f, "Panic"),
            Order::Pardon => write!(f, "Pardon"),
            Order::Perform(_) => write!(f, "Execute"),
            Order::Skip => write!(f, "Skip"),
            Order::Transform(_) => write!(f, "Map"),
        }
    }
}