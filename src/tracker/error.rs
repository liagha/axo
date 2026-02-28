use std::fmt::{write, Display};
use std::io::{Error as IOError, ErrorKind as IOKind};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    EmptyVoid,
    NotFound,
    PermissionDenied,
    AlreadyExists,
    WouldBlock,
    NotADirectory,
    IsADirectory,
    DirectoryNotEmpty,
    ReadOnly,
    InvalidInput,
    InvalidData,
    StorageFull,
    NotSeekable,
    QuotaExceeded,
    FileTooLarge,
    ResourceBusy,
    ExecutableFileBusy,
    Deadlock,
    CrossesDevices,
    TooManyLinks,
    InvalidFilename,
    ArgumentListTooLong,
    Interrupted,
    Unsupported,
    UnExpectedEOF,
    OutOfMemory,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::EmptyVoid => write!(f, "can't get the value of a void."),
            ErrorKind::NotFound => write!(f, "entity was not found."),
            ErrorKind::PermissionDenied => write!(f, "the operation lacked the necessary permissions to complete."),
            ErrorKind::AlreadyExists => write!(f, "the entity already exists."),
            ErrorKind::WouldBlock => write!(f, "the operation needs to block to complete, but the blocking operation was requested to not occur."),
            ErrorKind::NotADirectory => write!(f, "the object is, unexpectedly, not a directory."),
            ErrorKind::IsADirectory => write!(f, "the object is, unexpectedly, a directory."),
            ErrorKind::DirectoryNotEmpty => write!(f, "a non-empty directory was specified where an empty directory was expected."),
            ErrorKind::ReadOnly => write!(f, "the storage is read-only, but a write operation was attempted."),
            ErrorKind::InvalidInput => write!(f, "a parameter was not valid."),
            ErrorKind::InvalidData => write!(f, "data not valid for the operation were encountered."),
            ErrorKind::StorageFull => write!(f, "the storage is full."),
            ErrorKind::NotSeekable => write!(f, "the underlying storage is full."),
            ErrorKind::QuotaExceeded => write!(f, "filesystem quota or some other kind of quota was exceeded."),
            ErrorKind::FileTooLarge => write!(f, "the file is larger than the maximum allowed size."),
            ErrorKind::ResourceBusy => write!(f, "the resource is busy."),
            ErrorKind::ExecutableFileBusy => write!(f, "executable file is busy."),
            ErrorKind::Deadlock => write!(f, "the operation is deadlock."),
            ErrorKind::CrossesDevices => write!(f, "cross-device or cross-filesystem (hard) link or rename."),
            ErrorKind::TooManyLinks => write!(f, "too many (hard) links to the same object."),
            ErrorKind::InvalidFilename => write!(f, "a filename is invalid."),
            ErrorKind::ArgumentListTooLong => write!(f, "the program argument list was too long."),
            ErrorKind::Interrupted => write!(f, "the operation was interrupted."),
            ErrorKind::Unsupported => write!(f, "the operation was unsupported."),
            ErrorKind::UnExpectedEOF => write!(f, "the operation could not be completed because an `end of file` was reached prematurely."),
            ErrorKind::OutOfMemory => write!(f, "the operation could not be completed, because it failed to allocate enough memory."),
        }
    }
}

impl From<IOError> for ErrorKind {
    fn from(value: IOError) -> Self {
        match value.kind() {
            IOKind::NotFound => Self::NotFound,
            IOKind::PermissionDenied => Self::PermissionDenied,
            IOKind::AlreadyExists => Self::AlreadyExists,
            IOKind::WouldBlock => Self::WouldBlock,
            IOKind::NotADirectory => Self::NotADirectory,
            IOKind::IsADirectory => Self::IsADirectory,
            IOKind::DirectoryNotEmpty => Self::DirectoryNotEmpty,
            IOKind::ReadOnlyFilesystem => Self::ReadOnly,
            IOKind::InvalidInput => Self::InvalidInput,
            IOKind::InvalidData => Self::InvalidData,
            IOKind::StorageFull => Self::StorageFull,
            IOKind::NotSeekable => Self::NotSeekable,
            IOKind::QuotaExceeded => Self::QuotaExceeded,
            IOKind::FileTooLarge => Self::FileTooLarge,
            IOKind::ResourceBusy => Self::ResourceBusy,
            IOKind::ExecutableFileBusy => Self::ExecutableFileBusy,
            IOKind::Deadlock => Self::Deadlock,
            IOKind::CrossesDevices => Self::CrossesDevices,
            IOKind::TooManyLinks => Self::TooManyLinks,
            IOKind::InvalidFilename => Self::InvalidFilename,
            IOKind::ArgumentListTooLong => Self::ArgumentListTooLong,
            IOKind::Interrupted => Self::Interrupted,
            IOKind::Unsupported => Self::Unsupported,
            IOKind::UnexpectedEof => Self::UnExpectedEOF,
            IOKind::OutOfMemory => Self::OutOfMemory,
            _ => panic!("unexpected IO error: {}", value.kind()),
        }
    }
}