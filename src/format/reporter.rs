use {
    crate::{
        data::Str,
        reporter::Error,
        format::{Show, Verbosity, Debug, Display, Formatter, Result}
    }
};

impl<'error, K, H> Show<'error> for Error<'error, K, H>
where
    K: Clone + Display,
    H: Clone + Display,
{
    fn format(&self, verbosity: Verbosity) -> Str<'error> {
        match verbosity {
            _ => {
                let (msg, details) = self.handle();

                format!("{} \n {}", msg, details).into()
            }
        }
    }
}

impl<'error, K, H> Display for Error<'error, K, H>
where
    K: Clone + Display,
    H: Clone + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.format(Verbosity::Minimal))
    }
}

impl<'error, K, H> Debug for Error<'error, K, H>
where
    K: Clone + Display,
    H: Clone + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.format(Verbosity::Detailed))
    }
}
