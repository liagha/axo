use {
    core::fmt::{
        Debug, Formatter
    },
    crate::{
        string::*,
    },
};

impl Debug for Jaro {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "are a smart match")
    }
}

impl Debug for Cosine {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "are a smart match")
    }
}

impl Debug for Exact {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "are an exact match")
    }
}

impl Debug for Relaxed {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "are case-insensitive match")
    }
}

impl Debug for Prefix {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "share the same prefix")
    }
}

impl Debug for Suffix {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "share the same suffix")
    }
}

impl Debug for Contains {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "are a part of each other")
    }
}

impl Debug for Keyboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "are typed close on the keyboard")
    }
}

impl Debug for Phonetic {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "sound similar")
    }
}

impl Debug for Sequential {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "have letters in the same order")
    }
}

impl Debug for Words {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "share common words")
    }
}
