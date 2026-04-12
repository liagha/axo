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
    IndexBounds(usize, usize),
    Unindexable,
    InvalidUnary(Token<'error>, Type<'error>),
    InvalidBinary(Token<'error>, Type<'error>, Type<'error>),
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
    ExcessiveMembers {
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
            ErrorKind::EmptyIndex => write!(f, "empty index.").into(),
            ErrorKind::IndexBounds(index, len) => {
                write!(f, "index `{}` out of bounds for length `{}`.", index, len).into()
            }
            ErrorKind::Unindexable => write!(f, "target is not indexable.").into(),
            ErrorKind::InvalidUnary(operator, operand) => write!(
                f,
                "cannot apply `{}` to `{}`.",
                operator.format(Stencil::default()),
                operand.format(Stencil::default())
            )
                .into(),
            ErrorKind::InvalidBinary(operator, left, right) => write!(
                f,
                "cannot apply `{}` to `{}` and `{}`.",
                operator.format(Stencil::default()),
                left.format(Stencil::default()),
                right.format(Stencil::default())
            )
                .into(),
            ErrorKind::InvalidAnnotation(element) => write!(
                f,
                "invalid type annotation `{}`.",
                element.format(Stencil::default())
            )
                .into(),
            ErrorKind::UndefinedSymbol { query } => {
                write!(f, "undefined symbol `{}`.", query.format(Stencil::default())).into()
            }
            ErrorKind::MissingMember { target, member } => write!(
                f,
                "member `{}` missing from `{}`.",
                member.format(Stencil::default()),
                target.format(Stencil::default())
            )
                .into(),
            ErrorKind::UndefinedMember { target, member } => write!(
                f,
                "member `{}` undefined in `{}`.",
                member.format(Stencil::default()),
                target.format(Stencil::default())
            )
                .into(),
            ErrorKind::DefinedMember { target, member } => write!(
                f,
                "member `{}` already defined in `{}`.",
                member.format(Stencil::default()),
                target.format(Stencil::default())
            )
                .into(),
            ErrorKind::ExcessiveMembers { target, members } => write!(
                f,
                "union `{}` requires 1 initialized member but {} were provided.",
                target.format(Stencil::default()),
                members.len()
            )
                .into(),
        }
    }
}
