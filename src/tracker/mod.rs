pub mod error;
mod format;
mod position;
mod span;

use crate::{
    data::{Scale, Str},
    format::Display,
    internal::platform::{read_to_string, Path, PathBuf},
    reporter::Error,
};
pub use {error::*, position::*, span::*};

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
            Err(error) => Err(TrackError::new(
                ErrorKind::from_io(error, self.clone()),
                Span::void(),
            )),
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

pub trait Spanned<'spanned> {
    #[track_caller]
    fn span(&self) -> Span;
}

impl<'error, E> Spanned<'error> for Error<'error, E>
where
    E: Clone + Display,
{
    #[track_caller]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'item, T: Spanned<'item>> Spanned<'item> for Vec<T> {
    #[track_caller]
    fn span(&self) -> Span {
        Span::from_slice(self.as_slice())
    }
}

impl<'item, T: Spanned<'item>> Spanned<'item> for &[T] {
    #[track_caller]
    fn span(&self) -> Span {
        Span::from_slice(self)
    }
}

impl<'item, T: Spanned<'item>> Spanned<'item> for Box<[T]> {
    #[track_caller]
    fn span(&self) -> Span {
        self.as_ref().span()
    }
}

impl<'item, T: Spanned<'item>, const N: Scale> Spanned<'item> for [T; N] {
    #[track_caller]
    fn span(&self) -> Span {
        Span::from_slice(self.as_slice())
    }
}

pub type TrackError<'error> = Error<'error, ErrorKind<'error>>;
