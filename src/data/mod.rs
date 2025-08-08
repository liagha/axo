#![allow(unused_imports)]
pub mod float;
mod format;
mod string;

pub use {
    string::Str,
    core::{
        slice::from_ref,
        mem::{
            discriminant, replace
        },
    },
    std::{
        sync::{
            Arc, Mutex
        },
    },

    core::{
        char::{
            from_u32, from_u32_unchecked, MAX,
        },
        any::{
            Any, TypeId,
        },
        str::FromStr,
        marker::PhantomData,
    },
};

pub fn parse_radix<T: Number>(input: Str, radix: T) -> Option<T> {
    if input.is_empty() {
        return None;
    }

    let radix_u8 = radix.into();

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

        let digit = T::from(value);

        accumulator = accumulator.mul(radix)
            .add(digit);
    }

    Some(accumulator)
}

pub type Offset = usize;
pub type Scale = usize;

use crate::internal::{Add, Sub, Mul, Div};

pub trait Number: Copy + Default + Into<u8> + From<u8> + Add<Output=Self> + Sub<Output=Self> + Mul<Output=Self> + Div<Output=Self> {}

impl<T> Number for T where T: Copy + Default + Into<u8> + From<u8> + Add<Output=Self> + Sub<Output=Self> + Mul<Output=Self> + Div<Output=Self> {}