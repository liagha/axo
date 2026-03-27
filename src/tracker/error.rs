use crate::tracker::Location;
use crate::{
    format::{Display, Formatter, Result},
    internal::platform::{Error as IOError, ErrorKind as IOKind},
};

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
    ArgumentListTooLong(Location<'error>),
    Interrupted(Location<'error>),
    Unsupported(Location<'error>),
    UnExpectedEOF(Location<'error>),
    OutOfMemory(Location<'error>),
    UnSupportedInput(Location<'error>),
}

impl<'a> Display for ErrorKind<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::NotAnEntry(location) => write!(f, "`{}`, not an entry.", location),
            ErrorKind::EmptyVoid(location) => write!(f, "`{}`, can't get the value of a void.", location),
            ErrorKind::NotFound(location) => write!(f, "`{}`, entity was not found.", location),
            ErrorKind::PermissionDenied(location) => write!(f, "`{}`, the operation lacked the necessary permissions to complete.", location),
            ErrorKind::AlreadyExists(location) => write!(f, "`{}`, the entity already exists.", location),
            ErrorKind::WouldBlock(location) => write!(f, "`{}`, the operation needs to block to complete, but the blocking operation was requested to not occur.", location),
            ErrorKind::NotADirectory(location) => write!(f, "`{}`, the object is, unexpectedly, not a directory.", location),
            ErrorKind::IsADirectory(location) => write!(f, "`{}`, the object is, unexpectedly, a directory.", location),
            ErrorKind::DirectoryNotEmpty(location) => write!(f, "`{}`, a non-empty directory was specified where an empty directory was expected.", location),
            ErrorKind::ReadOnly(location) => write!(f, "`{}`, the storage is read-only, but a write operation was attempted.", location),
            ErrorKind::InvalidInput(location) => write!(f, "`{}`, a parameter was not valid.", location),
            ErrorKind::InvalidData(location) => write!(f, "`{}`, data not valid for the operation were encountered.", location),
            ErrorKind::StorageFull(location) => write!(f, "`{}`, the storage is full.", location),
            ErrorKind::NotSeekable(location) => write!(f, "`{}`, the underlying storage is full.", location),
            ErrorKind::QuotaExceeded(location) => write!(f, "`{}`, filesystem quota or some other kind of quota was exceeded.", location),
            ErrorKind::FileTooLarge(location) => write!(f, "`{}`, the file is larger than the maximum allowed size.", location),
            ErrorKind::ResourceBusy(location) => write!(f, "`{}`, the resource is busy.", location),
            ErrorKind::ExecutableFileBusy(location) => write!(f, "`{}`, executable file is busy.", location),
            ErrorKind::Deadlock(location) => write!(f, "`{}`, the operation is deadlock.", location),
            ErrorKind::CrossesDevices(location) => write!(f, "`{}`, cross-device or cross-filesystem (hard) link or rename.", location),
            ErrorKind::TooManyLinks(location) => write!(f, "`{}`, too many (hard) links to the same object.", location),
            ErrorKind::ArgumentListTooLong(location) => write!(f, "`{}`, the program argument list was too long.", location),
            ErrorKind::Interrupted(location) => write!(f, "`{}`, the operation was interrupted.", location),
            ErrorKind::Unsupported(location) => write!(f, "`{}`, the operation was unsupported.", location),
            ErrorKind::UnExpectedEOF(location) => write!(f, "`{}`, the operation could not be completed because an `end of file` was reached prematurely.", location),
            ErrorKind::OutOfMemory(location) => write!(f, "`{}`, the operation could not be completed, because it failed to allocate enough memory.", location),
            ErrorKind::UnSupportedInput(location) => write!(f, "the given input `{}` isn't supported.", location),
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
            IOKind::ArgumentListTooLong => Self::ArgumentListTooLong(target),
            IOKind::Interrupted => Self::Interrupted(target),
            IOKind::Unsupported => Self::Unsupported(target),
            IOKind::UnexpectedEof => Self::UnExpectedEOF(target),
            IOKind::OutOfMemory => Self::OutOfMemory(target),
            _ => panic!("unexpected IO error: {}", value.kind()),
        }
    }
}
