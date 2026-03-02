use {
    crate::runtime::text::Utf8String,
    crate::internal::platform::{self as io, BufRead, OnceLock, Write},
};

#[derive(Debug)]
pub struct IOError {
    kind: io::ErrorKind,
    code: Option<i32>,
}

impl IOError {
    #[inline]
    pub fn kind(&self) -> io::ErrorKind {
        self.kind
    }

    #[inline]
    pub fn raw_os_error(&self) -> Option<i32> {
        self.code
    }
}

impl From<io::Error> for IOError {
    fn from(value: io::Error) -> Self {
        Self {
            kind: value.kind(),
            code: value.raw_os_error(),
        }
    }
}

pub struct Stdout {
    inner: io::StdoutLock<'static>,
}

impl Stdout {
    #[inline]
    pub fn write_all(&mut self, bytes: &[u8]) -> Result<(), IOError> {
        write_all(&mut self.inner, bytes)
    }
}

pub struct Stdin {
    inner: io::StdinLock<'static>,
}

impl Stdin {
    #[inline]
    pub fn read_line(&mut self) -> Result<Utf8String, IOError> {
        let mut line = String::new();
        self.inner.read_line(&mut line).map_err(IOError::from)?;
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        Ok(Utf8String::from(line))
    }
}

#[inline]
pub fn stdout() -> Stdout {
    static HANDLE: OnceLock<io::Stdout> = OnceLock::new();
    let handle = HANDLE.get_or_init(io::stdout);
    Stdout {
        inner: handle.lock(),
    }
}

#[inline]
pub fn stdin() -> Stdin {
    static HANDLE: OnceLock<io::Stdin> = OnceLock::new();
    let handle = HANDLE.get_or_init(io::stdin);
    Stdin {
        inner: handle.lock(),
    }
}

#[inline]
pub fn write_stdout(bytes: &[u8]) -> Result<(), IOError> {
    let mut stream = stdout();
    stream.write_all(bytes)
}

#[inline]
pub fn write_stderr(bytes: &[u8]) -> Result<(), IOError> {
    let mut stderr = io::stderr().lock();
    write_all(&mut stderr, bytes)
}

#[inline]
pub fn print(text: &str) -> Result<(), IOError> {
    println(text)
}

#[inline]
pub fn println(text: &str) -> Result<(), IOError> {
    print_raw(text)?;
    write_stdout(b"\n")
}

#[inline]
pub fn print_raw(text: &str) -> Result<(), IOError> {
    write_stdout(text.as_bytes())
}

#[inline]
pub fn eprint(text: &str) -> Result<(), IOError> {
    eprintln(text)
}

#[inline]
pub fn eprintln(text: &str) -> Result<(), IOError> {
    eprint_raw(text)?;
    write_stderr(b"\n")
}

#[inline]
pub fn eprint_raw(text: &str) -> Result<(), IOError> {
    write_stderr(text.as_bytes())
}

#[inline]
pub fn read_line() -> Result<Utf8String, IOError> {
    let mut stream = stdin();
    stream.read_line()
}

fn write_all<W: Write>(writer: &mut W, bytes: &[u8]) -> Result<(), IOError> {
    let mut written = 0usize;

    while written < bytes.len() {
        match writer.write(&bytes[written..]) {
            Ok(0) => {
                return Err(io::Error::new(io::ErrorKind::WriteZero, "write returned zero").into());
            }
            Ok(count) => {
                written = written.checked_add(count).ok_or_else(|| {
                    IOError::from(io::Error::new(
                        io::ErrorKind::Other,
                        "write length overflow",
                    ))
                })?;
            }
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => return Err(err.into()),
        }
    }

    writer.flush().map_err(Into::into)
}
