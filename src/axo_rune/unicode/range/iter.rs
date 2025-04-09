use core::{char, ops};
use crate::axo_rune::unicode::CharRange;

const SURROGATE_RANGE: ops::Range<u32> = 0xD800..0xE000;

/// An iterator over a range of unicode code points.
///
/// Constructed via `CharRange::iter`. See `CharRange` for more information.
#[derive(Clone, Debug)]
pub struct CharIter {
    /// The lowest uniterated character (inclusive).
    ///
    /// Iteration is finished if this is higher than `high`.
    low: char,

    /// The highest uniterated character (inclusive).
    ///
    /// Iteration is finished if this is lower than `low`.
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
    // When stepping `self.low` forward would go over `char::MAX`,
    // Set `self.high` to `'\0'` instead. It will have the same effect --
    // consuming the last element from the iterator and ending iteration.
    fn step_forward(&mut self) {
        if self.low == char::MAX {
            self.high = '\0'
        } else {
            self.low = unsafe { forward(self.low) }
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    // When stepping `self.high` backward would cause underflow,
    // set `self.low` to `char::MAX` instead. It will have the same effect --
    // consuming the last element from the iterator and ending iteration.
    fn step_backward(&mut self) {
        if self.high == '\0' {
            self.low = char::MAX;
        } else {
            self.high = unsafe { backward(self.high) }
        }
    }

    #[inline]
    /// ExactSizeIterator::is_empty() for stable
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
/// Step a character one step towards `char::MAX`.
///
/// # Safety
///
/// If the given character is `char::MAX`, the return value is not a valid character.
pub unsafe fn forward(ch: char) -> char {
    if ch == BEFORE_SURROGATE {
        AFTER_SURROGATE
    } else {
        char::from_u32_unchecked(ch as u32 + 1)
    }
}

#[inline]
#[allow(unsafe_code)]
/// Step a character one step towards `'\0'`.
///
/// # Safety
///
/// If the given character is `'\0'`, this will cause an underflow.
/// (Thus, it will panic in debug mode, undefined behavior in release mode.)
pub unsafe fn backward(ch: char) -> char {
    if ch == AFTER_SURROGATE {
        BEFORE_SURROGATE
    } else {
        char::from_u32_unchecked(ch as u32 - 1)
    }
}
