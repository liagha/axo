use crate::{
    data::Str,
    format::{Display, Formatter, Result, Show, Stencil},
    parser::Element,
    resolver::Type,
    scanner::Token,
};

#[derive(Clone)]
pub enum ErrorKind<'error> {
    Mismatch(Type<'error>, Type<'error>),
    EmptyIndex,
    IndexOutOfBounds(usize, usize),
    UnIndexable,
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

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::Mismatch(left, right) => write!(
                f,
                "expected `{}` but got `{}`.",
                left.format(Stencil::default()),
                right.format(Stencil::default())
            )
            .into(),
            ErrorKind::EmptyIndex => {
                write!(f, "the index was empty.")
            }
            ErrorKind::IndexOutOfBounds(index, len) => {
                write!(f, "index `{}` out of bounds of `{}`.", index, len).into()
            }
            ErrorKind::UnIndexable => {
                write!(f, "unindexable indexing target.")
            }
            ErrorKind::InvalidOperation(token) => write!(
                f,
                "invalid operation for operand types: `{}`.",
                token.format(Stencil::default())
            )
            .into(),
            ErrorKind::InvalidAnnotation(element) => write!(
                f,
                "invalid type annotation: `{}`.",
                element.format(Stencil::default())
            )
            .into(),
            ErrorKind::UndefinedSymbol { query } => {
                write!(
                    f,
                    "undefined symbol: `{}`.",
                    query.format(Stencil::default())
                )
            }

            ErrorKind::MissingMember { target, member } => {
                write!(
                    f,
                    "the member `{}` is missing from `{}`.",
                    member.format(Stencil::default()),
                    target.format(Stencil::default())
                )
            }

            ErrorKind::UndefinedMember { target, member } => {
                write!(
                    f,
                    "the member `{}` doesn't exist in `{}`.",
                    member.format(Stencil::default()),
                    target.format(Stencil::default())
                )
            }

            ErrorKind::DefinedMember { target, member } => {
                write!(
                    f,
                    "the member `{}` is already defined in `{}`.",
                    member.format(Stencil::default()),
                    target.format(Stencil::default())
                )
            }
            ErrorKind::ExcessiveUnionMembers { target, members } => write!(
                f,
                "union `{}` can only have one member initialized, but {} were provided.",
                target.format(Stencil::default()),
                members.len()
            )
            .into(),
        }
    }
}
