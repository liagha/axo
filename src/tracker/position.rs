use orbyte::Orbyte;
use crate::{
    data::{Identity, Str, Offset},
    internal::platform::{args, read_to_string, Path, PathBuf},
    tracker::{ErrorKind, Span, TrackError},
};

#[derive(Clone, Copy, Eq, Hash, Orbyte, PartialEq)]
pub enum Location<'location> {
    Entry(Str<'location>),
    Void,
    Flag,
}

impl<'location> Location<'location> {
    pub fn as_path(&self) -> Result<PathBuf, TrackError<'location>> {
        match self {
            Location::Entry(path) => Ok(PathBuf::from(path).clone()),
            _ => Err(TrackError::new(ErrorKind::NotAnEntry(*self), Span::void())),
        }
    }

    pub fn to_path(&self) -> Result<PathBuf, TrackError<'location>> {
        match self {
            Location::Entry(path) => Ok(PathBuf::from(path).clone()),
            _ => Err(TrackError::new(ErrorKind::NotAnEntry(*self), Span::void())),
        }
    }

    pub fn get_value(&self) -> Result<Str<'location>, TrackError<'location>> {
        match self {
            Location::Entry(path) => {
                let location = Location::Entry(path.clone());
                let path = location.to_path()?;

                match read_to_string(&path) {
                    Ok(content) => Ok(content.into()),
                    Err(error) => Err(TrackError::new(ErrorKind::from_io(error, *self), Span::void())),
                }
            }
            Location::Flag => Ok(args()
                .skip(1)
                .map(|arg| {
                    if arg.contains(' ') || arg.contains('\t') {
                        format!("\"{}\"", arg.replace('\\', "\\\\").replace('"', "\\\""))
                    } else {
                        arg
                    }
                })
                .collect::<Vec<String>>()
                .join(" ")
                .into()),
            Location::Void => Err(TrackError::new(ErrorKind::EmptyVoid(*self), Span::void())),
        }
    }

    pub fn stem(&self) -> Option<&str> {
        match self {
            Location::Entry(path) => Path::new(path).file_stem()?.to_str(),
            _ => None,
        }
    }

    pub fn extension(&self) -> Option<&str> {
        match self {
            Location::Entry(path) => Path::new(path).extension()?.to_str(),
            _ => None,
        }
    }

    pub fn entry(string: Str<'location>) -> Location<'location> {
        Location::Entry(string)
    }

    pub fn flag() -> Location<'location> {
        Location::Flag
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Position {
    pub identity: Identity,
    pub offset: Offset,
}

impl Position {
    #[inline]
    pub fn new(identity: Identity) -> Self {
        Self { identity, offset: 0 }
    }

    #[inline]
    pub fn default(identity: Identity) -> Self {
        Self { identity, offset: 0 }
    }

    #[inline]
    pub fn set_identity(&mut self, identity: Identity) {
        self.identity = identity;
    }

    #[inline]
    pub fn set_offset(&mut self, offset: Offset) {
        self.offset = offset;
    }

    #[inline]
    pub fn swap_identity(&self, identity: Identity) -> Self {
        Self { identity, ..*self }
    }

    #[inline]
    pub fn swap_offset(&self, offset: Offset) -> Self {
        Self { offset, ..*self }
    }

    #[inline]
    pub fn advance(&self, amount: Offset) -> Self {
        Self { offset: self.offset + amount, ..*self }
    }

    #[inline]
    pub fn add(&mut self, amount: Offset) {
        self.offset += amount;
    }
}
