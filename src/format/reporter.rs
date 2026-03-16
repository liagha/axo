use {
    crate::{
        data::Str,
        reporter::Error,
        format::{Show, Debug, Display, Formatter, Result}
    }
};

impl<'error, K, H> Show<'error> for Error<'error, K, H>
where
    K: Clone + Display,
    H: Clone + Display,
{
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'error> {
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
        write!(f, "{}", self.format(0))
    }
}

impl<'error, K, H> Debug for Error<'error, K, H>
where
    K: Clone + Display,
    H: Clone + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.format(1))
    }
}
