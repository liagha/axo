use {
    crate::{
        format::{Show, Display, Formatter, Result, Verbosity},
        resolver::{Type},
        parser::Element,
        scanner::Token,
        data::Str,
    }
};

#[derive(Clone)]
pub enum ErrorKind<'error> {
    Mismatch(Type<'error>, Type<'error>),
    InvalidOperation(Token<'error>),
    InvalidAnnotation(Element<'error>),
    UndefinedSymbol {
        query: Str<'error>,
    },
    MissingMember {
        target: Str<'error>,
        member: Str<'error>,
    },
    UndefinedMember {
        target: Str<'error>,
        member: Str<'error>,
    },
    DefinedMember {
        target: Str<'error>,
        member: Str<'error>,
    },
    ExcessiveUnionMembers {
        target: Str<'error>,
        members: Vec<Str<'error>>,
    },
}

impl<'error> Show<'error> for ErrorKind<'error> {
    fn format(&self, verbosity: Verbosity) -> Str<'error> {
        match self {
            ErrorKind::Mismatch(left, right) => {
                format!("expected `{}` but got `{}`.", left.format(verbosity), right.format(verbosity)).into()
            }
            ErrorKind::InvalidOperation(token) => {
                format!("invalid operation for operand types: `{}`.", token.format(verbosity)).into()
            }
            ErrorKind::InvalidAnnotation(element) => {
                format!("invalid type annotation: `{}`.", element.format(verbosity)).into()
            }
            ErrorKind::UndefinedSymbol { query } => {
                format!("undefined symbol: `{}`.", query.format(verbosity))
            }

            ErrorKind::MissingMember { target, member } => {
                format!("the member `{}` is missing from `{}`.", member.format(verbosity), target.format(verbosity))
            }

            ErrorKind::UndefinedMember { target, member } => {
                format!("the member `{}` doesn't exist in `{}`.", member.format(verbosity), target.format(verbosity))
            }

            ErrorKind::DefinedMember { target, member } => {
                format!("the member `{}` is already defined in `{}`.", member.format(verbosity), target.format(verbosity))
            }
            ErrorKind::ExcessiveUnionMembers { target, members } => {
                format!(
                    "union `{}` can only have one member initialized, but {} were provided.",
                    target.format(verbosity),
                    members.len()
                ).into()
            }
        }.into()
    }
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.format(Verbosity::Minimal))
    }
}