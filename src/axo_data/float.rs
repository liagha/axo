use std::hash::{Hash, Hasher};
use std::cmp::Ordering;

/// A wrapper around `f64` that provides implementations for `Eq` and `Hash`.
#[derive(Debug, Copy, Clone)]
pub struct FloatLiteral(pub f64);

impl PartialEq for FloatLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 || (self.0.is_nan() && other.0.is_nan())
    }
}

impl Eq for FloatLiteral {}

impl Hash for FloatLiteral {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.0.is_nan() {
            // Hash all NaN values to the same hash
            state.write_u64(0x7ff8000000000000);
        } else {
            // Hash the raw bits of the f64 value
            state.write_u64(self.0.to_bits());
        }
    }
}

impl PartialOrd for FloatLiteral {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for FloatLiteral {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl From<f64> for FloatLiteral {
    fn from(f: f64) -> FloatLiteral {
        FloatLiteral(f)
    }
}

impl core::fmt::Display for FloatLiteral {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}