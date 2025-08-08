use {
    crate::{
        data,
        internal::Ordering,
        text::unicode::CharIter,
    }
};

#[derive(Clone, Copy, Debug, Eq)]
pub struct CharRange {
    pub low: char,

    pub high: char,
}

impl CharRange {
    pub fn closed(start: char, stop: char) -> CharRange {
        CharRange {
            low: start,
            high: stop,
        }
    }

    pub fn open_right(start: char, stop: char) -> CharRange {
        let mut iter = CharRange::closed(start, stop).iter();
        let _ = iter.next_back();
        iter.into()
    }

    pub fn open_left(start: char, stop: char) -> CharRange {
        let mut iter = CharRange::closed(start, stop).iter();
        let _ = iter.next();
        iter.into()
    }

    pub fn open(start: char, stop: char) -> CharRange {
        let mut iter = CharRange::closed(start, stop).iter();
        let _ = iter.next();
        let _ = iter.next_back();
        iter.into()
    }

    pub fn all() -> CharRange {
        CharRange::closed('\u{0}', data::MAX)
    }

    pub fn assigned_normal_planes() -> CharRange {
        CharRange::closed('\u{0}', '\u{2_FFFF}')
    }
}

impl CharRange {
    pub fn contains(&self, ch: char) -> bool {
        self.low <= ch && ch <= self.high
    }

    pub fn cmp_char(&self, ch: char) -> Ordering {
        assert!(!self.is_empty(), "Cannot compare empty range's ordering");
        if self.high < ch {
            Ordering::Less
        } else if self.low > ch {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }

    pub fn len(&self) -> usize {
        self.iter().len()
    }

    pub fn is_empty(&self) -> bool {
        self.low > self.high
    }

    pub fn iter(&self) -> CharIter {
        (*self).into()
    }
}

impl IntoIterator for CharRange {
    type Item = char;
    type IntoIter = CharIter;

    fn into_iter(self) -> CharIter {
        self.iter()
    }
}

impl PartialEq<CharRange> for CharRange {
    fn eq(&self, other: &CharRange) -> bool {
        (self.is_empty() && other.is_empty()) || (self.low == other.low && self.high == other.high)
    }
}
