extern crate alloc;

use {
    crate::{
        data::{
            memory::{Borrow, Copied},
            slice::{Iter, SliceIndex},
        },
        format::{
            self,
            Display, Formatter,
        },
        internal::{
            operation::{Deref, Index},
            platform::{OsStr, OsString, Path, PathBuf},
        },
    },
};

pub use {
    core::{
        str::{from_utf8, FromStr, Utf8Error},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Str<'a>(pub &'a [u8]);

impl<'a> Str<'a> {
    #[inline]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.0
    }

    #[inline]
    pub fn as_str(&self) -> Option<&'a str> {
        from_utf8(self.0).ok()
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_str(self) -> &'a str {
        self.as_str().unwrap()
    }

    #[inline]
    pub fn is_ascii(&self) -> bool {
        self.0.is_ascii()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn bytes(&self) -> Iter<'a, u8> {
        self.0.iter()
    }
}

impl<'a> Deref for Str<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str().expect("Str contains invalid UTF-8")
    }
}

impl<'a, I> Index<I> for Str<'a>
where
    I: SliceIndex<str, Output = str>,
{
    type Output = str;

    fn index(&self, index: I) -> &Self::Output {
        &self.as_str().expect("Str contains invalid UTF-8")[index]
    }
}

impl<'a> AsRef<[u8]> for Str<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl<'a> AsRef<str> for Str<'a> {
    fn as_ref(&self) -> &str {
        self.as_str().expect("Str contains invalid UTF-8")
    }
}

impl<'a> AsRef<Path> for Str<'a> {
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str().expect("Str contains invalid UTF-8"))
    }
}

impl<'a> AsRef<OsStr> for Str<'a> {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self.as_str().expect("Str contains invalid UTF-8"))
    }
}

impl<'a> Borrow<[u8]> for Str<'a> {
    fn borrow(&self) -> &[u8] {
        self.0
    }
}

impl<'a> Borrow<str> for Str<'a> {
    fn borrow(&self) -> &str {
        self.as_str().expect("Str contains invalid UTF-8")
    }
}

impl<'a> PartialEq<str> for Str<'a> {
    fn eq(&self, other: &str) -> bool {
        self.as_str().map_or(false, |s| s == other)
    }
}

impl<'a> PartialEq<&str> for Str<'a> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str().map_or(false, |s| s == *other)
    }
}

impl<'a> PartialEq<String> for Str<'a> {
    fn eq(&self, other: &String) -> bool {
        self.as_str().map_or(false, |s| s == other)
    }
}

impl<'a> PartialEq<&String> for Str<'a> {
    fn eq(&self, other: &&String) -> bool {
        self.as_str().map_or(false, |s| s == *other)
    }
}

impl<'a> PartialEq<&[u8]> for Str<'a> {
    fn eq(&self, other: &&[u8]) -> bool {
        self.0 == *other
    }
}

impl<'a> PartialEq<[u8]> for Str<'a> {
    fn eq(&self, other: &[u8]) -> bool {
        self.0 == other
    }
}

impl<'a> Default for Str<'a> {
    fn default() -> Self {
        Str(b"")
    }
}

impl<'a> Display for Str<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        match self.as_str() {
            Some(s) => write!(f, "{}", s),
            None => write!(f, "{:?}", self.0),
        }
    }
}

impl<'a> From<&'a str> for Str<'a> {
    fn from(s: &'a str) -> Self {
        Str(s.as_bytes())
    }
}

impl<'a> From<String> for Str<'a> {
    fn from(s: String) -> Self {
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<&'a String> for Str<'a> {
    fn from(s: &'a String) -> Self {
        Str(s.as_bytes())
    }
}

impl<'a> From<&'a [u8]> for Str<'a> {
    fn from(b: &'a [u8]) -> Self {
        Str(b)
    }
}

impl<'a> From<f64> for Str<'a> {
    fn from(f: f64) -> Self {
        let s = alloc::format!("{}", f);
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<f32> for Str<'a> {
    fn from(f: f32) -> Self {
        let s = alloc::format!("{}", f);
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<i8> for Str<'a> {
    fn from(i: i8) -> Self {
        let s = alloc::format!("{}", i);
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<i16> for Str<'a> {
    fn from(i: i16) -> Self {
        let s = alloc::format!("{}", i);
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<i32> for Str<'a> {
    fn from(i: i32) -> Self {
        let s = alloc::format!("{}", i);
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<i64> for Str<'a> {
    fn from(i: i64) -> Self {
        let s = alloc::format!("{}", i);
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<i128> for Str<'a> {
    fn from(i: i128) -> Self {
        let s = alloc::format!("{}", i);
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<isize> for Str<'a> {
    fn from(i: isize) -> Self {
        let s = alloc::format!("{}", i);
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<u8> for Str<'a> {
    fn from(u: u8) -> Self {
        let s = alloc::format!("{}", u);
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<u16> for Str<'a> {
    fn from(u: u16) -> Self {
        let s = alloc::format!("{}", u);
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<u32> for Str<'a> {
    fn from(u: u32) -> Self {
        let s = alloc::format!("{}", u);
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<u64> for Str<'a> {
    fn from(u: u64) -> Self {
        let s = alloc::format!("{}", u);
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<u128> for Str<'a> {
    fn from(u: u128) -> Self {
        let s = alloc::format!("{}", u);
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<usize> for Str<'a> {
    fn from(u: usize) -> Self {
        let s = alloc::format!("{}", u);
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<bool> for Str<'a> {
    fn from(b: bool) -> Self {
        let s = if b { "true" } else { "false" };
        Str(s.as_bytes())
    }
}

impl<'a> From<char> for Str<'a> {
    fn from(c: char) -> Self {
        let s = alloc::format!("{}", c);
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<&'a Path> for Str<'a> {
    fn from(p: &'a Path) -> Self {
        Str(p.as_os_str().as_encoded_bytes())
    }
}

impl<'a> From<PathBuf> for Str<'a> {
    fn from(p: PathBuf) -> Self {
        let s = p.into_os_string().into_string().expect("PathBuf contains invalid UTF-8");
        Str(s.leak().as_bytes())
    }
}

impl<'a> From<&'a OsStr> for Str<'a> {
    fn from(os: &'a OsStr) -> Self {
        Str(os.as_encoded_bytes())
    }
}

impl<'a> From<OsString> for Str<'a> {
    fn from(os: OsString) -> Self {
        let s = os.into_string().expect("OsString contains invalid UTF-8");
        Str(s.leak().as_bytes())
    }
}

impl<'a> IntoIterator for Str<'a> {
    type Item = u8;
    type IntoIter = Copied<Iter<'a, u8>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

impl<'a> IntoIterator for &Str<'a> {
    type Item = u8;
    type IntoIter = Copied<Iter<'a, u8>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

impl<'a> FromIterator<char> for Str<'a> {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let s: String = iter.into_iter().collect();
        Str(s.leak().as_bytes())
    }
}

impl<'a> FromIterator<u8> for Str<'a> {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let bytes: Vec<u8> = iter.into_iter().collect();
        Str(bytes.leak())
    }
}

impl<'a> FromIterator<&'a u8> for Str<'a> {
    fn from_iter<T: IntoIterator<Item = &'a u8>>(iter: T) -> Self {
        let bytes: Vec<u8> = iter.into_iter().copied().collect();
        Str(bytes.leak())
    }
}

impl<'a> FromIterator<String> for Str<'a> {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        let s: String = iter.into_iter().collect();
        Str(s.leak().as_bytes())
    }
}

impl<'a> FromIterator<&'a str> for Str<'a> {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let s: String = iter.into_iter().collect();
        Str(s.leak().as_bytes())
    }
}

impl<'a> TryFrom<Vec<u8>> for Str<'a> {
    type Error = Utf8Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        from_utf8(&bytes)?;
        Ok(Str(bytes.leak()))
    }
}