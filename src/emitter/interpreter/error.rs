use crate::emitter::ErrorKind;

pub type InterpretError<'a> = crate::reporter::Error<'a, ErrorKind<'a>>;