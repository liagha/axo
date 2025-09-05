extern crate alloc;

use {
    crate::{
        data::{
            memory::{Borrow, Copied},
            slice::{Iter, SliceIndex},
        },
        format::{
            self,
            Display, Debug, Formatter,
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Str<'string>(pub &'string [u8]);

impl<'string> Str<'string> {
    #[inline]
    pub fn as_bytes(&self) -> &'string [u8] {
        self.0
    }

    #[inline]
    pub fn as_str(&self) -> Option<&'string str> {
        from_utf8(self.0).ok()
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_str(self) -> &'string str {
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
    pub fn bytes(&self) -> Iter<'string, u8> {
        self.0.iter()
    }

    pub fn lines(self) -> Vec<Str<'string>> {
        let bytes = self.0;
        let mut lines = Vec::new();
        let mut start = 0;

        for (i, &byte) in bytes.iter().enumerate() {
            if byte == b'\n' {
                lines.push(Str(&bytes[start..i]));
                start = i + 1;
            } else if byte == b'\r' {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    lines.push(Str(&bytes[start..i]));
                    start = i + 2;
                } else {
                    lines.push(Str(&bytes[start..i]));
                    start = i + 1;
                }
            }
        }

        if start < bytes.len() {
            lines.push(Str(&bytes[start..]));
        }

        lines
    }

    #[inline]
    pub fn map<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&str) -> R,
    {
        f(self.as_str().expect("Str contains invalid UTF-8."))
    }

    #[inline]
    pub fn join<S: AsRef<str>>(self, sep: &str, iter: impl IntoIterator<Item = S>) -> Str<'string> {
        let s = iter.into_iter()
            .map(|s| s.as_ref().to_string())
            .collect::<Vec<String>>()
            .join(sep);
        Str(s.leak().as_bytes())
    }

    #[inline]
    pub fn split(&self, pat: &str) -> Vec<Str<'string>> {
        self.as_str()
            .expect("Str contains invalid UTF-8.")
            .split(pat)
            .map(|s| Str(s.as_bytes()))
            .collect()
    }

    #[inline]
    pub fn trim(&self) -> Str<'string> {
        Str(self.as_str()
            .expect("Str contains invalid UTF-8.")
            .trim()
            .as_bytes())
    }

    #[inline]
    pub fn to_lowercase(&self) -> String {
        self.as_str()
            .expect("Str contains invalid UTF-8.")
            .to_lowercase()
    }

    #[inline]
    pub fn to_uppercase(&self) -> String {
        self.as_str()
            .expect("Str contains invalid UTF-8.")
            .to_uppercase()
    }

    #[inline]
    pub fn contains(&self, pat: &str) -> bool {
        self.as_str()
            .expect("Str contains invalid UTF-8.")
            .contains(pat)
    }
}

impl<'string> Deref for Str<'string> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str().expect("Str contains invalid UTF-8.")
    }
}

impl<'string, I> Index<I> for Str<'string>
where
    I: SliceIndex<str, Output = str>,
{
    type Output = str;

    fn index(&self, index: I) -> &Self::Output {
        &self.as_str().expect("Str contains invalid UTF-8.")[index]
    }
}

impl<'string> AsRef<[u8]> for Str<'string> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl<'string> AsRef<str> for Str<'string> {
    fn as_ref(&self) -> &str {
        self.as_str().expect("Str contains invalid UTF-8.")
    }
}

impl<'string> AsRef<Path> for Str<'string> {
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str().expect("Str contains invalid UTF-8."))
    }
}

impl<'string> AsRef<OsStr> for Str<'string> {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self.as_str().expect("Str contains invalid UTF-8."))
    }
}

impl<'string> Borrow<[u8]> for Str<'string> {
    fn borrow(&self) -> &[u8] {
        self.0
    }
}

impl<'string> Borrow<str> for Str<'string> {
    fn borrow(&self) -> &str {
        self.as_str().expect("Str contains invalid UTF-8")
    }
}

impl<'string> PartialEq<str> for Str<'string> {
    fn eq(&self, other: &str) -> bool {
        self.as_str().map_or(false, |s| s == other)
    }
}

impl<'string> PartialEq<&str> for Str<'string> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str().map_or(false, |s| s == *other)
    }
}

impl<'string> PartialEq<String> for Str<'string> {
    fn eq(&self, other: &String) -> bool {
        self.as_str().map_or(false, |s| s == other)
    }
}

impl<'string> PartialEq<&String> for Str<'string> {
    fn eq(&self, other: &&String) -> bool {
        self.as_str().map_or(false, |s| s == *other)
    }
}

impl<'string> PartialEq<&[u8]> for Str<'string> {
    fn eq(&self, other: &&[u8]) -> bool {
        self.0 == *other
    }
}

impl<'string> PartialEq<[u8]> for Str<'string> {
    fn eq(&self, other: &[u8]) -> bool {
        self.0 == other
    }
}

impl<'string> Default for Str<'string> {
    fn default() -> Self {
        Str(b"")
    }
}

impl<'string> Display for Str<'string> {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        match self.as_str() {
            Some(s) => write!(f, "{}", s),
            None => write!(f, "{:?}", self.0),
        }
    }
}

impl<'string> Debug for Str<'string> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self)
    }
}

impl<'string> From<&'string str> for Str<'string> {
    fn from(s: &'string str) -> Self {
        Str(s.as_bytes())
    }
}

impl<'string> From<String> for Str<'string> {
    fn from(s: String) -> Self {
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<&'string String> for Str<'string> {
    fn from(s: &'string String) -> Self {
        Str(s.as_bytes())
    }
}

impl<'string> From<&'string [u8]> for Str<'string> {
    fn from(b: &'string [u8]) -> Self {
        Str(b)
    }
}

impl<'string> From<f64> for Str<'string> {
    fn from(f: f64) -> Self {
        let s = alloc::format!("{}", f);
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<f32> for Str<'string> {
    fn from(f: f32) -> Self {
        let s = alloc::format!("{}", f);
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<i8> for Str<'string> {
    fn from(i: i8) -> Self {
        let s = alloc::format!("{}", i);
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<i16> for Str<'string> {
    fn from(i: i16) -> Self {
        let s = alloc::format!("{}", i);
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<i32> for Str<'string> {
    fn from(i: i32) -> Self {
        let s = alloc::format!("{}", i);
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<i64> for Str<'string> {
    fn from(i: i64) -> Self {
        let s = alloc::format!("{}", i);
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<i128> for Str<'string> {
    fn from(i: i128) -> Self {
        let s = alloc::format!("{}", i);
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<isize> for Str<'string> {
    fn from(i: isize) -> Self {
        let s = alloc::format!("{}", i);
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<u8> for Str<'string> {
    fn from(u: u8) -> Self {
        let s = alloc::format!("{}", u);
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<u16> for Str<'string> {
    fn from(u: u16) -> Self {
        let s = alloc::format!("{}", u);
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<u32> for Str<'string> {
    fn from(u: u32) -> Self {
        let s = alloc::format!("{}", u);
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<u64> for Str<'string> {
    fn from(u: u64) -> Self {
        let s = alloc::format!("{}", u);
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<u128> for Str<'string> {
    fn from(u: u128) -> Self {
        let s = alloc::format!("{}", u);
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<usize> for Str<'string> {
    fn from(u: usize) -> Self {
        let s = alloc::format!("{}", u);
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<bool> for Str<'string> {
    fn from(b: bool) -> Self {
        let s = if b { "true" } else { "false" };
        Str(s.as_bytes())
    }
}

impl<'string> From<char> for Str<'string> {
    fn from(c: char) -> Self {
        let s = alloc::format!("{}", c);
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<&'string Path> for Str<'string> {
    fn from(p: &'string Path) -> Self {
        Str(p.as_os_str().as_encoded_bytes())
    }
}

impl<'string> From<PathBuf> for Str<'string> {
    fn from(p: PathBuf) -> Self {
        let s = p.into_os_string().into_string().expect("PathBuf contains invalid UTF-8.");
        Str(s.leak().as_bytes())
    }
}

impl<'string> From<&'string OsStr> for Str<'string> {
    fn from(os: &'string OsStr) -> Self {
        Str(os.as_encoded_bytes())
    }
}

impl<'string> From<OsString> for Str<'string> {
    fn from(os: OsString) -> Self {
        let s = os.into_string().expect("OSString contains invalid UTF-8.");
        Str(s.leak().as_bytes())
    }
}

impl<'string> IntoIterator for Str<'string> {
    type Item = u8;
    type IntoIter = Copied<Iter<'string, u8>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

impl<'string> IntoIterator for &Str<'string> {
    type Item = u8;
    type IntoIter = Copied<Iter<'string, u8>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

impl<'string> FromIterator<char> for Str<'string> {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let s: String = iter.into_iter().collect();
        Str(s.leak().as_bytes())
    }
}

impl<'string> FromIterator<u8> for Str<'string> {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let bytes: Vec<u8> = iter.into_iter().collect();
        Str(bytes.leak())
    }
}

impl<'string> FromIterator<&'string u8> for Str<'string> {
    fn from_iter<T: IntoIterator<Item = &'string u8>>(iter: T) -> Self {
        let bytes: Vec<u8> = iter.into_iter().copied().collect();
        Str(bytes.leak())
    }
}

impl<'string> FromIterator<String> for Str<'string> {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        let s: String = iter.into_iter().collect();
        Str(s.leak().as_bytes())
    }
}

impl<'string> FromIterator<&'string str> for Str<'string> {
    fn from_iter<T: IntoIterator<Item = &'string str>>(iter: T) -> Self {
        let s: String = iter.into_iter().collect();
        Str(s.leak().as_bytes())
    }
}

impl<'string> TryFrom<Vec<u8>> for Str<'string> {
    type Error = Utf8Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        from_utf8(&bytes)?;
        Ok(Str(bytes.leak()))
    }
}