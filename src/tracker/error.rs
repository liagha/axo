use {
    crate::{
        format::{Result, Display, Formatter},
        internal::platform::{Error as IOError, ErrorKind as IOKind},
    }
};
use crate::tracker::Location;

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind<'error> {
    NotAnEntry(Location<'error>),
    EmptyVoid(Location<'error>),
    NotFound(Location<'error>),
    PermissionDenied(Location<'error>),
    AlreadyExists(Location<'error>),
    WouldBlock(Location<'error>),
    NotADirectory(Location<'error>),
    IsADirectory(Location<'error>),
    DirectoryNotEmpty(Location<'error>),
    ReadOnly(Location<'error>),
    InvalidInput(Location<'error>),
    InvalidData(Location<'error>),
    StorageFull(Location<'error>),
    NotSeekable(Location<'error>),
    QuotaExceeded(Location<'error>),
    FileTooLarge(Location<'error>),
    ResourceBusy(Location<'error>),
    ExecutableFileBusy(Location<'error>),
    Deadlock(Location<'error>),
    CrossesDevices(Location<'error>),
    TooManyLinks(Location<'error>),
    InvalidFilename(Location<'error>),
    ArgumentListTooLong(Location<'error>),
    Interrupted(Location<'error>),
    Unsupported(Location<'error>),
    UnExpectedEOF(Location<'error>),
    OutOfMemory(Location<'error>),
}

impl<'a> Display for ErrorKind<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::NotAnEntry(loc) => write!(f, "`{}`, not an entry.", loc),
            ErrorKind::EmptyVoid(loc) => write!(f, "`{}`, can't get the value of a void.", loc),
            ErrorKind::NotFound(loc) => write!(f, "`{}`, entity was not found.", loc),
            ErrorKind::PermissionDenied(loc) => write!(f, "`{}`, the operation lacked the necessary permissions to complete.", loc),
            ErrorKind::AlreadyExists(loc) => write!(f, "`{}`, the entity already exists.", loc),
            ErrorKind::WouldBlock(loc) => write!(f, "`{}`, the operation needs to block to complete, but the blocking operation was requested to not occur.", loc),
            ErrorKind::NotADirectory(loc) => write!(f, "`{}`, the object is, unexpectedly, not a directory.", loc),
            ErrorKind::IsADirectory(loc) => write!(f, "`{}`, the object is, unexpectedly, a directory.", loc),
            ErrorKind::DirectoryNotEmpty(loc) => write!(f, "`{}`, a non-empty directory was specified where an empty directory was expected.", loc),
            ErrorKind::ReadOnly(loc) => write!(f, "`{}`, the storage is read-only, but a write operation was attempted.", loc),
            ErrorKind::InvalidInput(loc) => write!(f, "`{}`, a parameter was not valid.", loc),
            ErrorKind::InvalidData(loc) => write!(f, "`{}`, data not valid for the operation were encountered.", loc),
            ErrorKind::StorageFull(loc) => write!(f, "`{}`, the storage is full.", loc),
            ErrorKind::NotSeekable(loc) => write!(f, "`{}`, the underlying storage is full.", loc),
            ErrorKind::QuotaExceeded(loc) => write!(f, "`{}`, filesystem quota or some other kind of quota was exceeded.", loc),
            ErrorKind::FileTooLarge(loc) => write!(f, "`{}`, the file is larger than the maximum allowed size.", loc),
            ErrorKind::ResourceBusy(loc) => write!(f, "`{}`, the resource is busy.", loc),
            ErrorKind::ExecutableFileBusy(loc) => write!(f, "`{}`, executable file is busy.", loc),
            ErrorKind::Deadlock(loc) => write!(f, "`{}`, the operation is deadlock.", loc),
            ErrorKind::CrossesDevices(loc) => write!(f, "`{}`, cross-device or cross-filesystem (hard) link or rename.", loc),
            ErrorKind::TooManyLinks(loc) => write!(f, "`{}`, too many (hard) links to the same object.", loc),
            ErrorKind::InvalidFilename(loc) => write!(f, "`{}`, a filename is invalid.", loc),
            ErrorKind::ArgumentListTooLong(loc) => write!(f, "`{}`, the program argument list was too long.", loc),
            ErrorKind::Interrupted(loc) => write!(f, "`{}`, the operation was interrupted.", loc),
            ErrorKind::Unsupported(loc) => write!(f, "`{}`, the operation was unsupported.", loc),
            ErrorKind::UnExpectedEOF(loc) => write!(f, "`{}`, the operation could not be completed because an `end of file` was reached prematurely.", loc),
            ErrorKind::OutOfMemory(loc) => write!(f, "`{}`, the operation could not be completed, because it failed to allocate enough memory.", loc),
        }
    }
}

impl<'a> ErrorKind<'a> {
    pub fn from_io(value: IOError, target: Location<'a>) -> ErrorKind<'a> {
        match value.kind() {
            IOKind::NotFound => Self::NotFound(target),
            IOKind::PermissionDenied => Self::PermissionDenied(target),
            IOKind::AlreadyExists => Self::AlreadyExists(target),
            IOKind::WouldBlock => Self::WouldBlock(target),
            IOKind::NotADirectory => Self::NotADirectory(target),
            IOKind::IsADirectory => Self::IsADirectory(target),
            IOKind::DirectoryNotEmpty => Self::DirectoryNotEmpty(target),
            IOKind::ReadOnlyFilesystem => Self::ReadOnly(target),
            IOKind::InvalidInput => Self::InvalidInput(target),
            IOKind::InvalidData => Self::InvalidData(target),
            IOKind::StorageFull => Self::StorageFull(target),
            IOKind::NotSeekable => Self::NotSeekable(target),
            IOKind::QuotaExceeded => Self::QuotaExceeded(target),
            IOKind::FileTooLarge => Self::FileTooLarge(target),
            IOKind::ResourceBusy => Self::ResourceBusy(target),
            IOKind::ExecutableFileBusy => Self::ExecutableFileBusy(target),
            IOKind::Deadlock => Self::Deadlock(target),
            IOKind::CrossesDevices => Self::CrossesDevices(target),
            IOKind::TooManyLinks => Self::TooManyLinks(target),
            IOKind::InvalidFilename => Self::InvalidFilename(target),
            IOKind::ArgumentListTooLong => Self::ArgumentListTooLong(target),
            IOKind::Interrupted => Self::Interrupted(target),
            IOKind::Unsupported => Self::Unsupported(target),
            IOKind::UnexpectedEof => Self::UnExpectedEOF(target),
            IOKind::OutOfMemory => Self::OutOfMemory(target),
            _ => panic!("unexpected IO error: {}", value.kind()),
        }
    }
}
