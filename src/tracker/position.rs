use {
    crate::{
        data::{from_utf8, slice::from_raw_parts, Offset, Pointer, Scale, Str},
        internal::{
            operation::Ordering,
            platform::{
                args,
                read_to_string,
                PathBuf
            },
        },
        tracker::{ErrorKind, Span, TrackError}
    }
};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Location<'location> {
    Entry(Str<'location>),
    Raw { ptr: Pointer, len: Scale },
    Void,
    Flag,
}

impl<'location> Location<'location> {
    pub fn to_path(&self) -> Result<PathBuf, TrackError<'location>> {
        match self {
            Location::Entry(path) => {
                let path = PathBuf::from(path.as_str().unwrap());

                if path.exists() {
                    Ok(path)
                } else {
                    let kind = ErrorKind::NotFound;

                    Err(
                        TrackError::new(
                            kind,
                            Span::void(),
                        )
                    )
                }
            }
            _ => {
                let kind = ErrorKind::NotAnEntry;

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
                        let kind: ErrorKind = error.into();

                        Err(TrackError::new(kind, Span::file(Str::from(path))))
                    }
                }
            }
            Location::Raw { ptr, len, .. } => {
                let slice = unsafe { from_raw_parts(*ptr, *len) };
                match from_utf8(slice) {
                    Ok(s) => Ok(s.into()),
                    Err(e) => Ok(String::from_utf8_lossy(slice).to_string().into()),
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
            Location::Void => Err(TrackError::new(ErrorKind::EmptyVoid, Span::void())),
        }
    }

    pub fn file(string: Str<'location>) -> Location<'location> {
        Location::Entry(string)
    }

    pub fn raw<T>(value: &'location T) -> Location<'location>
    where
        T: AsRef<[u8]> + ?Sized,
    {
        let bytes = value.as_ref();
        Location::Raw {
            ptr: bytes.as_ptr(),
            len: bytes.len(),
        }
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
