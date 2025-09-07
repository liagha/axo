mod float;
mod string;

pub use {
    float::Float,
    string::{
        Str, Utf8Error, FromStr, from_utf8,
    },
};

pub type Char = char;
pub type Boolean = bool;
pub type Pointer = *const u8;
pub type Offset = usize;
pub type Scale = usize;
pub type Integer = i128;

pub mod any {
    pub use {
        core::{
            any::{Any, TypeId},
        }
    };
}

pub mod character {
    pub use {
        core::{
            char::{from_u32, from_u32_unchecked, MAX},
        },
    };

    use super::{Number, Str};

    pub fn parse_radix<T: Number>(input: Str, radix: T) -> Option<T> {
        if input.is_empty() {
            return None;
        }

        let radix_u8 = radix.to_u8().unwrap();

        if radix_u8 < 2 || radix_u8 > 36 {
            return None;
        }

        let mut accumulator = T::default();

        for &byte in input.as_bytes() {
            let value = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'z' => byte - b'a' + 10,
                b'A'..=b'Z' => byte - b'A' + 10,
                _ => return None,
            };

            if value >= radix_u8 {
                return None;
            }

            let digit = T::from_u8(value).unwrap();

            accumulator = accumulator.mul(radix)
                .add(digit);
        }

        Some(accumulator)
    }
}

pub mod memory {
    pub use {
        core::{
            borrow::Borrow,
            iter::Copied,
            marker::PhantomData,
            mem::{
                discriminant, replace,
                transmute,
            },
        },
    };
}

pub mod slice {
    pub use {
        core::{
            slice::{from_ref, from_raw_parts, Iter, SliceIndex}
        }
    };
}

pub mod thread {
    pub use {
        std::{
            sync::{
                Arc, Mutex
            },
        },
    };
}

use crate::internal::operation::{Add, Sub, Mul, Div, Rem, Neg, BitAnd, BitOr, BitXor, Shl, Shr};

pub trait Number:
Copy
+ Default
+ PartialEq
+ PartialOrd
+ Add<Output = Self>
+ Sub<Output = Self>
+ Mul<Output = Self>
+ Div<Output = Self>
+ Rem<Output = Self>
+ BitAnd<Output = Self>
+ BitOr<Output = Self>
+ BitXor<Output = Self>
+ Shl<u32, Output = Self>
+ Shr<u32, Output = Self>
{
    fn zero() -> Self;
    fn one() -> Self;
    fn is_zero(&self) -> bool;
    fn abs(&self) -> Self;
    fn digit_count(&self) -> u32;
    fn to_u8(&self) -> Option<u8>;
    fn from_u8(value: u8) -> Option<Self>;
}

impl Number for u8 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn is_zero(&self) -> bool { *self == 0 }
    fn abs(&self) -> Self { *self }
    fn digit_count(&self) -> u32 {
        if *self == 0 { return 1; }
        let mut count = 0;
        let mut value = *self;
        while value != 0 {
            value /= 10;
            count += 1;
        }
        count
    }
    fn to_u8(&self) -> Option<u8> { Some(*self) }
    fn from_u8(value: u8) -> Option<Self> { Some(value) }
}

impl Number for u16 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn is_zero(&self) -> bool { *self == 0 }
    fn abs(&self) -> Self { *self }
    fn digit_count(&self) -> u32 {
        if *self == 0 { return 1; }
        let mut count = 0;
        let mut value = *self;
        while value != 0 {
            value /= 10;
            count += 1;
        }
        count
    }
    fn to_u8(&self) -> Option<u8> { (*self).try_into().ok() }
    fn from_u8(value: u8) -> Option<Self> { Some(value as u16) }
}

impl Number for u32 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn is_zero(&self) -> bool { *self == 0 }
    fn abs(&self) -> Self { *self }
    fn digit_count(&self) -> u32 {
        if *self == 0 { return 1; }
        let mut count = 0;
        let mut value = *self;
        while value != 0 {
            value /= 10;
            count += 1;
        }
        count
    }
    fn to_u8(&self) -> Option<u8> { (*self).try_into().ok() }
    fn from_u8(value: u8) -> Option<Self> { Some(value as u32) }
}

impl Number for u64 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn is_zero(&self) -> bool { *self == 0 }
    fn abs(&self) -> Self { *self }
    fn digit_count(&self) -> u32 {
        if *self == 0 { return 1; }
        let mut count = 0;
        let mut value = *self;
        while value != 0 {
            value /= 10;
            count += 1;
        }
        count
    }
    fn to_u8(&self) -> Option<u8> { (*self).try_into().ok() }
    fn from_u8(value: u8) -> Option<Self> { Some(value as u64) }
}

impl Number for usize {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn is_zero(&self) -> bool { *self == 0 }
    fn abs(&self) -> Self { *self }
    fn digit_count(&self) -> u32 {
        if *self == 0 { return 1; }
        let mut count = 0;
        let mut value = *self;
        while value != 0 {
            value /= 10;
            count += 1;
        }
        count
    }
    fn to_u8(&self) -> Option<u8> { (*self).try_into().ok() }
    fn from_u8(value: u8) -> Option<Self> { Some(value as usize) }
}

impl Number for i8 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn is_zero(&self) -> bool { *self == 0 }
    fn abs(&self) -> Self { (*self).abs() }
    fn digit_count(&self) -> u32 {
        if *self == 0 { return 1; }
        let mut count = 0;
        let mut value = (*self).abs();
        while value != 0 {
            value /= 10;
            count += 1;
        }
        count
    }
    fn to_u8(&self) -> Option<u8> { (*self).try_into().ok() }
    fn from_u8(value: u8) -> Option<Self> { value.try_into().ok() }
}

impl Number for i16 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn is_zero(&self) -> bool { *self == 0 }
    fn abs(&self) -> Self { (*self).abs() }
    fn digit_count(&self) -> u32 {
        if *self == 0 { return 1; }
        let mut count = 0;
        let mut value = (*self).abs();
        while value != 0 {
            value /= 10;
            count += 1;
        }
        count
    }
    fn to_u8(&self) -> Option<u8> { (*self).try_into().ok() }
    fn from_u8(value: u8) -> Option<Self> { Some(value as i16) }
}

impl Number for i32 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn is_zero(&self) -> bool { *self == 0 }
    fn abs(&self) -> Self { (*self).abs() }
    fn digit_count(&self) -> u32 {
        if *self == 0 { return 1; }
        let mut count = 0;
        let mut value = (*self).abs();
        while value != 0 {
            value /= 10;
            count += 1;
        }
        count
    }
    fn to_u8(&self) -> Option<u8> { (*self).try_into().ok() }
    fn from_u8(value: u8) -> Option<Self> { Some(value as i32) }
}

impl Number for i64 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn is_zero(&self) -> bool { *self == 0 }
    fn abs(&self) -> Self { (*self).abs() }
    fn digit_count(&self) -> u32 {
        if *self == 0 { return 1; }
        let mut count = 0;
        let mut value = (*self).abs();
        while value != 0 {
            value /= 10;
            count += 1;
        }
        count
    }
    fn to_u8(&self) -> Option<u8> { (*self).try_into().ok() }
    fn from_u8(value: u8) -> Option<Self> { Some(value as i64) }
}

impl Number for isize {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn is_zero(&self) -> bool { *self == 0 }
    fn abs(&self) -> Self { (*self).abs() }
    fn digit_count(&self) -> u32 {
        if *self == 0 { return 1; }
        let mut count = 0;
        let mut value = (*self).abs();
        while value != 0 {
            value /= 10;
            count += 1;
        }
        count
    }
    fn to_u8(&self) -> Option<u8> { (*self).try_into().ok() }
    fn from_u8(value: u8) -> Option<Self> { Some(value as isize) }
}