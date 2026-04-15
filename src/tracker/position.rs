use crate::{
    data::{Identity, Offset, Str},
    internal::platform::{read_to_string, Path, PathBuf},
    tracker::{ErrorKind, Span, TrackError},
};

pub type Location<'a> = Str<'a>;

impl<'a> Location<'a> {
    pub fn as_path(&self) -> Result<PathBuf, TrackError<'a>> {
        Ok(PathBuf::from(self).clone())
    }

    pub fn to_path(&self) -> Result<PathBuf, TrackError<'a>> {
        Ok(PathBuf::from(self).clone())
    }

    pub fn get_value(&self) -> Result<Str<'a>, TrackError<'a>> {
        let path = self.to_path()?;

        match read_to_string(&path) {
            Ok(content) => Ok(content.into()),
            Err(error) => Err(TrackError::new(ErrorKind::from_io(error, self.clone()), Span::void())),
        }
    }

    pub fn stem(&self) -> Option<&str> {
        Path::new(self).file_stem()?.to_str()
    }

    pub fn extension(&self) -> Option<&str> {
        Path::new(self).extension()?.to_str()
    }

    pub fn entry(string: Str<'a>) -> Location<'a> {
        string
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
        Self {
            offset: self.offset + amount,
            ..*self
        }
    }

    #[inline]
    pub fn add(&mut self, amount: Offset) {
        self.offset += amount;
    }
}
