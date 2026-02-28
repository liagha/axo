use {
    crate::data::{from_utf8, Utf8Error},
    crate::format::{self as fmt, Debug, Display},
    crate::internal::{operation::Deref, platform::null},
    crate::runtime::memory::{next_capacity, AllocationError, Result as AllocationResult},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StringAbi {
    pub ptr: *const u8,
    pub len: usize,
}

impl StringAbi {
    #[inline]
    pub fn empty() -> Self {
        Self {
            ptr: null(),
            len: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utf8Str<'a>(&'a str);

impl<'a> Utf8Str<'a> {
    #[inline]
    pub fn new(inner: &'a str) -> Self {
        Self(inner)
    }

    #[inline]
    pub fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, Utf8Error> {
        Ok(Self(from_utf8(bytes)?))
    }

    #[inline]
    pub fn as_str(self) -> &'a str {
        self.0
    }

    #[inline]
    pub fn as_bytes(self) -> &'a [u8] {
        self.0.as_bytes()
    }

    #[inline]
    pub fn len_bytes(self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn abi(self) -> StringAbi {
        StringAbi {
            ptr: self.0.as_ptr(),
            len: self.0.len(),
        }
    }
}

impl<'a> Default for Utf8Str<'a> {
    fn default() -> Self {
        Self("")
    }
}

impl<'a> From<&'a str> for Utf8Str<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

impl<'a> Deref for Utf8Str<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> AsRef<str> for Utf8Str<'a> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'a> AsRef<[u8]> for Utf8Str<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl<'a> Display for Utf8Str<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl<'a> Debug for Utf8Str<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Utf8String {
    bytes: Vec<u8>,
}

impl Utf8String {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn from_str(value: &str) -> Self {
        Self {
            bytes: value.as_bytes().to_vec(),
        }
    }

    #[inline]
    pub fn from_utf8(bytes: Vec<u8>) -> Result<Self, Utf8Error> {
        from_utf8(&bytes)?;
        Ok(Self { bytes })
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.bytes) }
    }

    #[inline]
    pub fn as_utf8_str(&self) -> Utf8Str<'_> {
        Utf8Str::new(self.as_str())
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    pub fn len_bytes(&self) -> usize {
        self.bytes.len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.bytes.capacity()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.bytes.clear();
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) -> AllocationResult<()> {
        let current = self.bytes.capacity();
        let required = next_capacity(current, additional)?;

        if required > current {
            self.bytes
                .try_reserve_exact(required - current)
                .map_err(|_| AllocationError)?;
        }

        Ok(())
    }

    #[inline]
    pub fn push_str(&mut self, value: &str) -> AllocationResult<()> {
        self.reserve(value.len())?;
        self.bytes.extend_from_slice(value.as_bytes());
        Ok(())
    }

    #[inline]
    pub fn push_char(&mut self, value: char) -> AllocationResult<()> {
        let mut buf = [0u8; 4];
        let encoded = value.encode_utf8(&mut buf);
        self.push_str(encoded)
    }

    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    #[inline]
    pub fn into_std_string(self) -> String {
        unsafe { String::from_utf8_unchecked(self.bytes) }
    }

    #[inline]
    pub fn abi(&self) -> StringAbi {
        StringAbi {
            ptr: self.bytes.as_ptr(),
            len: self.bytes.len(),
        }
    }
}

impl From<&str> for Utf8String {
    fn from(value: &str) -> Self {
        Self::from_str(value)
    }
}

impl From<String> for Utf8String {
    fn from(value: String) -> Self {
        Self {
            bytes: value.into_bytes(),
        }
    }
}

impl<'a> From<Utf8Str<'a>> for StringAbi {
    fn from(value: Utf8Str<'a>) -> Self {
        value.abi()
    }
}

impl From<&str> for StringAbi {
    fn from(value: &str) -> Self {
        StringAbi {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }
}

impl From<&Utf8String> for StringAbi {
    fn from(value: &Utf8String) -> Self {
        value.abi()
    }
}

impl TryFrom<Vec<u8>> for Utf8String {
    type Error = Utf8Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_utf8(value)
    }
}

impl Deref for Utf8String {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Utf8String {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for Utf8String {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Display for Utf8String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Debug for Utf8String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.as_str(), f)
    }
}
