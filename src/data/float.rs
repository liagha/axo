use {
    crate::{
        format::{Display, Debug, Formatter, Result},
        internal::{
            hash::{Hash, Hasher},
            operation::{Add, Sub, Mul, Div, Neg, Rem, Ordering}
        },
    }
};

#[derive(Clone, Copy, Debug)]
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
            state.write_u64(0x7ff8000000000000);
        } else {
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
        self.partial_cmp(other).unwrap_or_else(|| {
            if self.0.is_nan() && !other.0.is_nan() {
                Ordering::Greater
            } else if !self.0.is_nan() && other.0.is_nan() {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
    }
}

impl Add for FloatLiteral {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        FloatLiteral(self.0 + other.0)
    }
}

impl Sub for FloatLiteral {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        FloatLiteral(self.0 - other.0)
    }
}

impl Mul for FloatLiteral {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        FloatLiteral(self.0 * other.0)
    }
}

impl Div for FloatLiteral {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        FloatLiteral(self.0 / other.0)
    }
}

impl Rem for FloatLiteral {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        FloatLiteral(self.0 % other.0)
    }
}

impl Neg for FloatLiteral {
    type Output = Self;

    fn neg(self) -> Self {
        FloatLiteral(-self.0)
    }
}

impl From<f64> for FloatLiteral {
    fn from(f: f64) -> FloatLiteral {
        FloatLiteral(f)
    }
}

impl From<FloatLiteral> for f64 {
    fn from(val: FloatLiteral) -> f64 {
        val.0
    }
}

impl From<i32> for FloatLiteral {
    fn from(i: i32) -> FloatLiteral {
        FloatLiteral(i as f64)
    }
}

impl From<f32> for FloatLiteral {
    fn from(f: f32) -> FloatLiteral {
        FloatLiteral(f as f64)
    }
}

impl Display for FloatLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl FloatLiteral {
    pub fn zero(self) -> Self {
        FloatLiteral(0.0)    
    }
    
    pub fn abs(self) -> Self {
        FloatLiteral(self.0.abs())
    }

    pub fn sqrt(self) -> Self {
        FloatLiteral(self.0.sqrt())
    }

    pub fn powi(self, n: i32) -> Self {
        FloatLiteral(self.0.powi(n))
    }

    pub fn powf(self, n: f64) -> Self {
        FloatLiteral(self.0.powf(n))
    }

    pub fn exp(self) -> Self {
        FloatLiteral(self.0.exp())
    }

    pub fn ln(self) -> Self {
        FloatLiteral(self.0.ln())
    }

    pub fn log10(self) -> Self {
        FloatLiteral(self.0.log10())
    }

    pub fn log2(self) -> Self {
        FloatLiteral(self.0.log2())
    }

    pub fn sin(self) -> Self {
        FloatLiteral(self.0.sin())
    }

    pub fn cos(self) -> Self {
        FloatLiteral(self.0.cos())
    }

    pub fn tan(self) -> Self {
        FloatLiteral(self.0.tan())
    }

    pub fn asin(self) -> Self {
        FloatLiteral(self.0.asin())
    }

    pub fn acos(self) -> Self {
        FloatLiteral(self.0.acos())
    }

    pub fn atan(self) -> Self {
        FloatLiteral(self.0.atan())
    }

    pub fn floor(self) -> Self {
        FloatLiteral(self.0.floor())
    }

    pub fn ceil(self) -> Self {
        FloatLiteral(self.0.ceil())
    }

    pub fn round(self) -> Self {
        FloatLiteral(self.0.round())
    }

    pub fn trunc(self) -> Self {
        FloatLiteral(self.0.trunc())
    }

    pub fn fract(self) -> Self {
        FloatLiteral(self.0.fract())
    }

    pub fn is_nan(self) -> bool {
        self.0.is_nan()
    }

    pub fn is_infinite(self) -> bool {
        self.0.is_infinite()
    }

    pub fn is_finite(self) -> bool {
        self.0.is_finite()
    }

    pub fn is_normal(self) -> bool {
        self.0.is_normal()
    }

    pub fn min(self, other: Self) -> Self {
        FloatLiteral(self.0.min(other.0))
    }

    pub fn max(self, other: Self) -> Self {
        FloatLiteral(self.0.max(other.0))
    }

    pub fn clamp(self, min: Self, max: Self) -> Self {
        FloatLiteral(self.0.clamp(min.0, max.0))
    }

    pub fn to_degrees(self) -> Self {
        FloatLiteral(self.0.to_degrees())
    }

    pub fn to_radians(self) -> Self {
        FloatLiteral(self.0.to_radians())
    }

    pub fn recip(self) -> Self {
        FloatLiteral(self.0.recip())
    }

    pub fn to_bits(self) -> u64 {
        self.0.to_bits()
    }

    pub fn from_bits(bits: u64) -> Self {
        FloatLiteral(f64::from_bits(bits))
    }
}