pub enum Error {
    CursorMove,
    Flush(std::io::Error),
    WriteError,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::CursorMove => write!(f, "Cursor move failed"),
            Error::Flush(e) => write!(f, "Flush failed: {}", e),
            Error::WriteError => write!(f, "WRite failed!")
        }
    }
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::CursorMove => write!(f, "Cursor move failed"),
            Error::Flush(e) => write!(f, "Flush failed: {}", e),
            Error::WriteError => write!(f, "WRite failed!")
        }
    }
}