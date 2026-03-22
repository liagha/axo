use {
    crate::{
        data::{Offset, Str},
        internal::{
            operation::Ordering,
            platform::{
                args,
                read_to_string,
                Path,
                PathBuf
            },
        },
        tracker::{ErrorKind, Span, TrackError}
    }
};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Location<'location> {
    Entry(Str<'location>),
    Void,
    Flag,
}

impl<'location> Location<'location> {
    pub fn as_path(&self) -> Result<PathBuf, TrackError<'location>> {
        match self {
            Location::Entry(path) => {
                let path = PathBuf::from(path);

                Ok(path.clone())
            }
            _ => {
                let kind = ErrorKind::NotAnEntry(*self);

                Err(
                    TrackError::new(
                        kind,
                        Span::void(),
                    )
                )
            }
        }
    }

    pub fn to_path(&self) -> Result<PathBuf, TrackError<'location>> {
        match self {
            Location::Entry(path) => {
                let path = PathBuf::from(path);

                Ok(path.clone())
            }
            _ => {
                let kind = ErrorKind::NotAnEntry(*self);

                Err(
                    TrackError::new(
                        kind,
                        Span::void(),
                    )
                )
            }
        }
    }

    pub fn get_value(&self) -> Result<Str<'location>, TrackError<'location>> {
        match self {
            Location::Entry(path) => {
                let location = Location::Entry(path.clone());
                let path = location.to_path()?;

                match read_to_string(&path) {
                    Ok(content) => Ok(content.into()),
                    Err(error) => {
                        let kind = ErrorKind::from_io(error, *self);

                        Err(TrackError::new(kind, Span::void()))
                    }
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
            Location::Entry(path) => {
                let path = Path::new(path);
                path.file_stem()?.to_str()
            }
            _ => None,
        }
    }

    pub fn extension(&self) -> Option<&str> {
        match self {
            Location::Entry(path) => {
                let path = Path::new(path);
                path.extension()?.to_str()
            }
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Position<'position> {
    pub line: Offset,
    pub column: Offset,
    pub location: Location<'position>,
}

impl<'a> Position<'a> {
    #[inline]
    pub fn new(location: Location<'a>) -> Self {
        Self {
            line: 1,
            column: 1,
            location,
        }
    }

    #[inline]
    pub fn default(location: Location<'a>) -> Self {
        Self {
            line: 1,
            column: 1,
            location,
        }
    }

    #[inline]
    pub fn path(line: Offset, column: Offset, path: Str<'a>) -> Self {
        Self {
            line,
            column,
            location: Location::Entry(path),
        }
    }

    #[inline]
    pub fn set_line(&mut self, line: Offset) {
        self.line = line;
    }

    #[inline]
    pub fn set_column(&mut self, column: Offset) {
        self.column = column;
    }

    #[inline]
    pub fn set_path(&mut self, path: Str<'a>) {
        self.location = Location::Entry(path);
    }

    #[inline]
    pub fn set_location(&mut self, location: Location<'a>) {
        self.location = location;
    }

    #[inline]
    pub fn swap_line(&self, line: Offset) -> Self {
        Self { line, ..*self }
    }

    #[inline]
    pub fn swap_column(&self, column: Offset) -> Self {
        Self { column, ..*self }
    }

    #[inline]
    pub fn swap_path(&self, path: Str<'a>) -> Self {
        Self {
            location: Location::Entry(path),
            ..*self
        }
    }

    #[inline]
    pub fn swap_location(&self, location: Location<'a>) -> Self {
        Self { location, ..*self }
    }

    #[inline]
    pub fn advance_line(&self, amount: Offset) -> Self {
        Self {
            line: self.line + amount,
            ..*self
        }
    }

    #[inline]
    pub fn join_column(&self, amount: Offset) -> Self {
        Self {
            column: self.column + amount,
            ..*self
        }
    }

    #[inline]
    pub fn add_line(&mut self, amount: Offset) {
        self.line += amount;
    }

    #[inline]
    pub fn add_column(&mut self, amount: Offset) {
        self.column += amount;
    }

    pub fn cmp(&self, other: &Self) -> Ordering {
        if self.location != other.location {
            return Ordering::Less;
        }

        match self.line.cmp(&other.line) {
            Ordering::Equal => self.column.cmp(&other.column),
            other => other,
        }
    }
}

impl<'a> PartialOrd for Position<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Position<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

use crate::internal::cache::{Encode, Decode};

impl<'location> Encode for Location<'location> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            Location::Entry(path) => {
                buffer.push(0);
                path.encode(buffer);
            }
            Location::Void => buffer.push(1),
            Location::Flag => buffer.push(2),
        }
    }
}

impl<'location> Decode<'location> for Location<'location> {
    fn decode(buffer: &'location [u8], cursor: &mut usize) -> Self {
        let tag = buffer[*cursor];
        *cursor += 1;
        match tag {
            0 => Location::Entry(Str::decode(buffer, cursor)),
            1 => Location::Void,
            2 => Location::Flag,
            _ => panic!(),
        }
    }
}

impl<'position> Encode for Position<'position> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.line.encode(buffer);
        self.column.encode(buffer);
        self.location.encode(buffer);
    }
}

impl<'position> Decode<'position> for Position<'position> {
    fn decode(buffer: &'position [u8], cursor: &mut usize) -> Self {
        Position {
            line: Offset::decode(buffer, cursor),
            column: Offset::decode(buffer, cursor),
            location: Location::decode(buffer, cursor),
        }
    }
}