use {
    crate::{
        format::{Debug, Display, Formatter, Result, Show, Stencil},
        reporter::Error,
    },
    broccli::{Color, TextStyle},
};

impl<'error, K, H> Show<'error> for Error<'error, K, H>
where
    K: Clone + Display,
    H: Clone + Display,
{
    fn format(&self, config: Stencil) -> Stencil {
        let (message, details) = self.handle();

        config
            .clone()
            .new("Error")
            .field("message", message.to_string())
            .field("details", details.to_string())
    }
}

impl<'error, K, H> Display for Error<'error, K, H>
where
    K: Clone + Display,
    H: Clone + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let (message, details) = self.handle();

        write!(
            f,
            "{}{}\n{}",
            "error:".colorize(Color::Crimson).bold(),
            message,
            details
        )
    }
}

impl<'error, K, H> Debug for Error<'error, K, H>
where
    K: Clone + Display,
    H: Clone + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let (message, details) = self.handle();

        write!(
            f,
            "{}{}\n{}",
            "error:".colorize(Color::Crimson).bold(),
            message,
            details
        )
    }
}
