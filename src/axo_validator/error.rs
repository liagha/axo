pub enum ErrorKind {

}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            _ => write!(f, "{}", self),
        }
    }
}