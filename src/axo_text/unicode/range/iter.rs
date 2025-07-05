use {
    crate::{
        character,
        operations,
        axo_text::unicode::CharRange,
    }
};

const SURROGATE_RANGE: operations::Range<u32> = 0xD800..0xE000;

#[derive(Clone, Debug)]
pub struct CharIter {
    low: char,

    high: char,
}

impl From<CharRange> for CharIter {
    fn from(range: CharRange) -> CharIter {
        CharIter {
            low: range.low,
            high: range.high,
        }
    }
}

impl From<CharIter> for CharRange {
    fn from(iter: CharIter) -> CharRange {
        CharRange {
            low: iter.low,
            high: iter.high,
        }
    }
}

impl CharIter {
    #[inline]
    #[allow(unsafe_code)]
    fn step_forward(&mut self) {
        if self.low == character::MAX {
            self.high = '\0'
        } else {
            self.low = unsafe { forward(self.low) }
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn step_backward(&mut self) {
        if self.high == '\0' {
            self.low = character::MAX;
        } else {
            self.high = unsafe { backward(self.high) }
        }
    }

    #[inline]
    fn is_finished(&self) -> bool {
        self.low > self.high
    }
}

impl Iterator for CharIter {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<char> {
        if self.is_finished() {
            return None;
        }

        let ch = self.low;
        self.step_forward();
        Some(ch)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn last(self) -> Option<char> {
        if self.is_finished() {
            None
        } else {
            Some(self.high)
        }
    }

    fn max(self) -> Option<char> {
        self.last()
    }

    fn min(mut self) -> Option<char> {
        self.next()
    }
}

impl DoubleEndedIterator for CharIter {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.is_finished() {
            None
        } else {
            let ch = self.high;
            self.step_backward();
            Some(ch)
        }
    }
}

impl ExactSizeIterator for CharIter {
    fn len(&self) -> usize {
        if self.is_finished() {
            return 0;
        }
        let naive_range = (self.low as u32)..(self.high as u32 + 1);
        if naive_range.start <= SURROGATE_RANGE.start && SURROGATE_RANGE.end <= naive_range.end {
            naive_range.len() - SURROGATE_RANGE.len()
        } else {
            naive_range.len()
        }
    }
}

pub const BEFORE_SURROGATE: char = '\u{D7FF}';
pub const AFTER_SURROGATE: char = '\u{E000}';

#[inline]
#[allow(unsafe_code)]
pub unsafe fn forward(ch: char) -> char {
    if ch == BEFORE_SURROGATE {
        AFTER_SURROGATE
    } else {
        character::from_u32_unchecked(ch as u32 + 1)
    }
}

#[inline]
#[allow(unsafe_code)]
pub unsafe fn backward(ch: char) -> char {
    if ch == AFTER_SURROGATE {
        BEFORE_SURROGATE
    } else {
        character::from_u32_unchecked(ch as u32 - 1)
    }
}
