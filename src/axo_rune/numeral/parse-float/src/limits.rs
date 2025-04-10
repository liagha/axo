//! Determine the limits of exact exponent and mantissas for floats.

#![doc(hidden)]

use lexical_util::assert::debug_assert_radix;
#[cfg(feature = "f16")]
use lexical_util::bf16::bf16;
#[cfg(feature = "f16")]
use lexical_util::f16::f16;

// EXACT EXPONENT
// --------------

// Calculating the exponent limit requires determining the largest exponent
// we can calculate for a radix that can be **exactly** store in the
// float type. If the value is a power-of-two, then we simply
// need to scale the minimum, denormal exp and maximum exp to the type
// size. Otherwise, we need to calculate the number of digits
// that can fit into the type's precision, after removing a power-of-two
// (since these values can be represented exactly).
//
// The mantissa limit is the number of digits we can remove from
// the exponent into the mantissa, and is therefore is the
// `⌊ precision / log2(radix) ⌋`, where precision does not include
// the hidden bit.
//
// The algorithm for calculating both `exponent_limit` and `mantissa_limit`,
// in Python, can be done as follows:
//
// DO NOT MODIFY: Generated by `src/etc/limits.py`

// EXACT FLOAT
// -----------

/// Get exact exponent limit for radix.
#[doc(hidden)]
pub trait ExactFloat {
    /// Get min and max exponent limits (exact) from radix.
    fn exponent_limit(radix: u32) -> (i64, i64);

    /// Get the number of digits that can be shifted from exponent to mantissa.
    fn mantissa_limit(radix: u32) -> i64;
}

impl ExactFloat for f32 {
    #[inline(always)]
    fn exponent_limit(radix: u32) -> (i64, i64) {
        debug_assert_radix(radix);
        f32_exponent_limit(radix)
    }

    #[inline(always)]
    fn mantissa_limit(radix: u32) -> i64 {
        debug_assert_radix(radix);
        f32_mantissa_limit(radix)
    }
}

impl ExactFloat for f64 {
    #[inline(always)]
    fn exponent_limit(radix: u32) -> (i64, i64) {
        debug_assert_radix(radix);
        f64_exponent_limit(radix)
    }

    #[inline(always)]
    fn mantissa_limit(radix: u32) -> i64 {
        debug_assert_radix(radix);
        f64_mantissa_limit(radix)
    }
}

#[cfg(feature = "f16")]
impl ExactFloat for f16 {
    #[inline(always)]
    fn exponent_limit(_: u32) -> (i64, i64) {
        unimplemented!()
    }

    #[inline(always)]
    fn mantissa_limit(_: u32) -> i64 {
        unimplemented!()
    }
}

#[cfg(feature = "f16")]
impl ExactFloat for bf16 {
    #[inline(always)]
    fn exponent_limit(_: u32) -> (i64, i64) {
        unimplemented!()
    }

    #[inline(always)]
    fn mantissa_limit(_: u32) -> i64 {
        unimplemented!()
    }
}

//#[cfg(feature = "f128")]
//impl ExactFloat for f128 {
//    #[inline(always)]
//    fn exponent_limit(radix: u32) -> (i64, i64) {
//        debug_assert_radix(radix);
//        f128_exponent_limit(radix)
//        }
//    }
//
//    #[inline(always)]
//    fn mantissa_limit(radix: u32) -> i64 {
//        debug_assert_radix(radix);
//        f128_mantissa_limit(radix)
//    }
//}

// CONST FN
// --------

/// Get the exponent limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(feature = "radix")]
pub const fn f32_exponent_limit(radix: u32) -> (i64, i64) {
    match radix {
        2 => (-127, 127),
        3 => (-15, 15),
        4 => (-63, 63),
        5 => (-10, 10),
        6 => (-15, 15),
        7 => (-8, 8),
        8 => (-42, 42),
        9 => (-7, 7),
        10 => (-10, 10),
        11 => (-6, 6),
        12 => (-15, 15),
        13 => (-6, 6),
        14 => (-8, 8),
        15 => (-6, 6),
        16 => (-31, 31),
        17 => (-5, 5),
        18 => (-7, 7),
        19 => (-5, 5),
        20 => (-10, 10),
        21 => (-5, 5),
        22 => (-6, 6),
        23 => (-5, 5),
        24 => (-15, 15),
        25 => (-5, 5),
        26 => (-6, 6),
        27 => (-5, 5),
        28 => (-8, 8),
        29 => (-4, 4),
        30 => (-6, 6),
        31 => (-4, 4),
        32 => (-25, 25),
        33 => (-4, 4),
        34 => (-5, 5),
        35 => (-4, 4),
        36 => (-7, 7),
        _ => (0, 0),
    }
}

/// Get the exponent limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(all(feature = "power-of-two", not(feature = "radix")))]
pub const fn f32_exponent_limit(radix: u32) -> (i64, i64) {
    match radix {
        2 => (-127, 127),
        4 => (-63, 63),
        8 => (-42, 42),
        10 => (-10, 10),
        16 => (-31, 31),
        32 => (-25, 25),
        _ => (0, 0),
    }
}

/// Get the exponent limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(not(feature = "power-of-two"))]
pub const fn f32_exponent_limit(radix: u32) -> (i64, i64) {
    match radix {
        10 => (-10, 10),
        _ => (0, 0),
    }
}

/// Get the mantissa limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(feature = "radix")]
pub const fn f32_mantissa_limit(radix: u32) -> i64 {
    match radix {
        2 => 24,
        3 => 15,
        4 => 12,
        5 => 10,
        6 => 9,
        7 => 8,
        8 => 8,
        9 => 7,
        10 => 7,
        11 => 6,
        12 => 6,
        13 => 6,
        14 => 6,
        15 => 6,
        16 => 6,
        17 => 5,
        18 => 5,
        19 => 5,
        20 => 5,
        21 => 5,
        22 => 5,
        23 => 5,
        24 => 5,
        25 => 5,
        26 => 5,
        27 => 5,
        28 => 4,
        29 => 4,
        30 => 4,
        31 => 4,
        32 => 4,
        33 => 4,
        34 => 4,
        35 => 4,
        36 => 4,
        _ => 0,
    }
}

/// Get the mantissa limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(all(feature = "power-of-two", not(feature = "radix")))]
pub const fn f32_mantissa_limit(radix: u32) -> i64 {
    match radix {
        2 => 24,
        4 => 12,
        8 => 8,
        10 => 7,
        16 => 6,
        32 => 4,
        _ => 0,
    }
}

/// Get the mantissa limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(not(feature = "power-of-two"))]
pub const fn f32_mantissa_limit(radix: u32) -> i64 {
    match radix {
        10 => 7,
        _ => 0,
    }
}

/// Get the exponent limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(feature = "radix")]
pub const fn f64_exponent_limit(radix: u32) -> (i64, i64) {
    match radix {
        2 => (-1023, 1023),
        3 => (-33, 33),
        4 => (-511, 511),
        5 => (-22, 22),
        6 => (-33, 33),
        7 => (-18, 18),
        8 => (-341, 341),
        9 => (-16, 16),
        10 => (-22, 22),
        11 => (-15, 15),
        12 => (-33, 33),
        13 => (-14, 14),
        14 => (-18, 18),
        15 => (-13, 13),
        16 => (-255, 255),
        17 => (-12, 12),
        18 => (-16, 16),
        19 => (-12, 12),
        20 => (-22, 22),
        21 => (-12, 12),
        22 => (-15, 15),
        23 => (-11, 11),
        24 => (-33, 33),
        25 => (-11, 11),
        26 => (-14, 14),
        27 => (-11, 11),
        28 => (-18, 18),
        29 => (-10, 10),
        30 => (-13, 13),
        31 => (-10, 10),
        32 => (-204, 204),
        33 => (-10, 10),
        34 => (-12, 12),
        35 => (-10, 10),
        36 => (-16, 16),
        _ => (0, 0),
    }
}

// Get the exponent limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(all(feature = "power-of-two", not(feature = "radix")))]
pub const fn f64_exponent_limit(radix: u32) -> (i64, i64) {
    match radix {
        2 => (-1023, 1023),
        4 => (-511, 511),
        8 => (-341, 341),
        10 => (-22, 22),
        16 => (-255, 255),
        32 => (-204, 204),
        _ => (0, 0),
    }
}

/// Get the exponent limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(not(feature = "power-of-two"))]
pub const fn f64_exponent_limit(radix: u32) -> (i64, i64) {
    match radix {
        10 => (-22, 22),
        _ => (0, 0),
    }
}

/// Get the mantissa limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(feature = "radix")]
pub const fn f64_mantissa_limit(radix: u32) -> i64 {
    match radix {
        2 => 53,
        3 => 33,
        4 => 26,
        5 => 22,
        6 => 20,
        7 => 18,
        8 => 17,
        9 => 16,
        10 => 15,
        11 => 15,
        12 => 14,
        13 => 14,
        14 => 13,
        15 => 13,
        16 => 13,
        17 => 12,
        18 => 12,
        19 => 12,
        20 => 12,
        21 => 12,
        22 => 11,
        23 => 11,
        24 => 11,
        25 => 11,
        26 => 11,
        27 => 11,
        28 => 11,
        29 => 10,
        30 => 10,
        31 => 10,
        32 => 10,
        33 => 10,
        34 => 10,
        35 => 10,
        36 => 10,
        _ => 0,
    }
}

/// Get the mantissa limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(all(feature = "power-of-two", not(feature = "radix")))]
pub const fn f64_mantissa_limit(radix: u32) -> i64 {
    match radix {
        2 => 53,
        4 => 26,
        8 => 17,
        10 => 15,
        16 => 13,
        32 => 10,
        _ => 0,
    }
}

/// Get the mantissa limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(not(feature = "power-of-two"))]
pub const fn f64_mantissa_limit(radix: u32) -> i64 {
    match radix {
        10 => 15,
        _ => 0,
    }
}

/// Get the exponent limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(feature = "f128")]
#[cfg(feature = "radix")]
pub const fn f128_exponent_limit(radix: u32) -> (i64, i64) {
    match radix {
        2 => (-16494, 16383),
        3 => (-71, 71),
        4 => (-8247, 8191),
        5 => (-48, 48),
        6 => (-71, 71),
        7 => (-40, 40),
        8 => (-5498, 5461),
        9 => (-35, 35),
        10 => (-48, 48),
        11 => (-32, 32),
        12 => (-71, 71),
        13 => (-30, 30),
        14 => (-40, 40),
        15 => (-28, 28),
        16 => (-4123, 4095),
        17 => (-27, 27),
        18 => (-35, 35),
        19 => (-26, 26),
        20 => (-48, 48),
        21 => (-25, 25),
        22 => (-32, 32),
        23 => (-24, 24),
        24 => (-71, 71),
        25 => (-24, 24),
        26 => (-30, 30),
        27 => (-23, 23),
        28 => (-40, 40),
        29 => (-23, 23),
        30 => (-28, 28),
        31 => (-22, 22),
        32 => (-3298, 3276),
        33 => (-22, 22),
        34 => (-27, 27),
        35 => (-22, 22),
        36 => (-35, 35),
        // Invalid radix
        _ => (0, 0),
    }
}

/// Get the exponent limit as a const fn.
#[inline(always)]
#[cfg(feature = "f128")]
#[cfg(all(feature = "power-of-two", not(feature = "radix")))]
pub const fn f128_exponent_limit(radix: u32) -> (i64, i64) {
    match radix {
        2 => (-16494, 16383),
        4 => (-8247, 8191),
        8 => (-5498, 5461),
        10 => (-48, 48),
        16 => (-4123, 4095),
        32 => (-3298, 3276),
        // Invalid radix
        _ => (0, 0),
    }
}

/// Get the exponent limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(feature = "f128")]
#[cfg(not(feature = "power-of-two"))]
pub const fn f128_exponent_limit(radix: u32) -> (i64, i64) {
    match radix {
        10 => (-48, 48),
        // Invalid radix
        _ => (0, 0),
    }
}

/// Get the mantissa limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(feature = "f128")]
#[cfg(feature = "radix")]
pub const fn f128_mantissa_limit(radix: u32) -> i64 {
    match radix {
        2 => 113,
        3 => 71,
        4 => 56,
        5 => 48,
        6 => 43,
        7 => 40,
        8 => 37,
        9 => 35,
        10 => 34,
        11 => 32,
        12 => 31,
        13 => 30,
        14 => 29,
        15 => 28,
        16 => 28,
        17 => 27,
        18 => 27,
        19 => 26,
        20 => 26,
        21 => 25,
        22 => 25,
        23 => 24,
        24 => 24,
        25 => 24,
        26 => 24,
        27 => 23,
        28 => 23,
        29 => 23,
        30 => 23,
        31 => 22,
        32 => 22,
        33 => 22,
        34 => 22,
        35 => 22,
        36 => 21,
        // Invalid radix
        _ => 0,
    }
}

/// Get the mantissa limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(feature = "f128")]
#[cfg(all(feature = "power-of-two", not(feature = "radix")))]
pub const fn f128_mantissa_limit(radix: u32) -> i64 {
    match radix {
        2 => 113,
        4 => 56,
        8 => 37,
        10 => 34,
        16 => 28,
        32 => 22,
        // Invalid radix
        _ => 0,
    }
}

/// Get the mantissa limit as a const fn.
#[must_use]
#[inline(always)]
#[cfg(feature = "f128")]
#[cfg(not(feature = "power-of-two"))]
pub const fn f128_mantissa_limit(radix: u32) -> i64 {
    match radix {
        10 => 34,
        // Invalid radix
        _ => 0,
    }
}

// POWER LIMITS
// ------------

//  The code used to generate these limits is as follows:
//
//  ```text
//  import math
//
//  def find_power(base, max_value):
//      '''Using log is unreliable, since it uses float math.'''
//
//      power = 0
//      while base**power < max_value:
//          power += 1
//      return power - 1
//
//  def print_function(bits):
//      print('#[inline(always)]')
//      print(f'pub const fn u{bits}_power_limit(radix: u32) -> u32 {{')
//      print('    match radix {')
//      max_value = 2**bits - 1
//      for radix in range(2, 37):
//          power = find_power(radix, max_value)
//          print(f'        {radix} => {power},')
//      print('        // Any other radix should be unreachable.')
//      print('        _ => 1,')
//      print('    }')
//      print('}')
//      print('')
//
//  print_function(32)
//  print_function(64)
//  ```

/// Get the maximum value for `radix^N` that can be represented in a u32.
/// This is calculated as `⌊log(2^32 - 1, b)⌋`.
#[must_use]
#[inline(always)]
#[cfg(feature = "radix")]
pub const fn u32_power_limit(radix: u32) -> u32 {
    match radix {
        2 => 31,
        3 => 20,
        4 => 15,
        5 => 13,
        6 => 12,
        7 => 11,
        8 => 10,
        9 => 10,
        10 => 9,
        11 => 9,
        12 => 8,
        13 => 8,
        14 => 8,
        15 => 8,
        16 => 7,
        17 => 7,
        18 => 7,
        19 => 7,
        20 => 7,
        21 => 7,
        22 => 7,
        23 => 7,
        24 => 6,
        25 => 6,
        26 => 6,
        27 => 6,
        28 => 6,
        29 => 6,
        30 => 6,
        31 => 6,
        32 => 6,
        33 => 6,
        34 => 6,
        35 => 6,
        36 => 6,
        // Any other radix should be unreachable.
        _ => 1,
    }
}

/// This is calculated as `⌊log(2^32 - 1, b)⌋`.
#[must_use]
#[inline(always)]
#[cfg(all(feature = "power-of-two", not(feature = "radix")))]
pub const fn u32_power_limit(radix: u32) -> u32 {
    match radix {
        2 => 31,
        4 => 15,
        5 => 13,
        8 => 10,
        10 => 9,
        16 => 7,
        32 => 6,
        // Any other radix should be unreachable.
        _ => 1,
    }
}

/// This is calculated as `⌊log(2^32 - 1, b)⌋`.
#[must_use]
#[inline(always)]
#[cfg(not(feature = "power-of-two"))]
pub const fn u32_power_limit(radix: u32) -> u32 {
    match radix {
        5 => 13,
        10 => 9,
        // Any other radix should be unreachable.
        _ => 1,
    }
}

/// Get the maximum value for `radix^N` that can be represented in a u64.
/// This is calculated as `⌊log(2^64 - 1, b)⌋`.
#[must_use]
#[inline(always)]
#[cfg(feature = "radix")]
pub const fn u64_power_limit(radix: u32) -> u32 {
    match radix {
        2 => 63,
        3 => 40,
        4 => 31,
        5 => 27,
        6 => 24,
        7 => 22,
        8 => 21,
        9 => 20,
        10 => 19,
        11 => 18,
        12 => 17,
        13 => 17,
        14 => 16,
        15 => 16,
        16 => 15,
        17 => 15,
        18 => 15,
        19 => 15,
        20 => 14,
        21 => 14,
        22 => 14,
        23 => 14,
        24 => 13,
        25 => 13,
        26 => 13,
        27 => 13,
        28 => 13,
        29 => 13,
        30 => 13,
        31 => 12,
        32 => 12,
        33 => 12,
        34 => 12,
        35 => 12,
        36 => 12,
        // Any other radix should be unreachable.
        _ => 1,
    }
}

/// Get the maximum value for `radix^N` that can be represented in a u64.
/// This is calculated as `⌊log(2^64 - 1, b)⌋`.
#[must_use]
#[inline(always)]
#[cfg(all(feature = "power-of-two", not(feature = "radix")))]
pub const fn u64_power_limit(radix: u32) -> u32 {
    match radix {
        2 => 63,
        4 => 31,
        5 => 27,
        8 => 21,
        10 => 19,
        16 => 15,
        32 => 12,
        // Any other radix should be unreachable.
        _ => 1,
    }
}

#[must_use]
#[inline(always)]
#[cfg(not(feature = "power-of-two"))]
pub const fn u64_power_limit(radix: u32) -> u32 {
    match radix {
        5 => 27,
        10 => 19,
        // Any other radix should be unreachable.
        _ => 1,
    }
}

// MAX DIGITS
// ----------

/// Calculate the maximum number of digits possible in the mantissa.
///
/// Returns the maximum number of digits plus one.
///
/// We can exactly represent a float in radix `b` from radix 2 if
/// `b` is divisible by 2. This function calculates the exact number of
/// digits required to exactly represent that float. This makes sense,
/// and the exact reference and I quote is:
///
///  > A necessary and sufficient condition for all numbers representable in
///  > radix β
///  > with a finite number of digits to be representable in radix γ with a
///  > finite number of digits is that β should divide an integer power of γ.
///
/// According to the "Handbook of Floating Point Arithmetic",
/// for IEEE754, with `emin` being the min exponent, `p2` being the
/// precision, and `b` being the radix, the number of digits follows as:
///
/// `−emin + p2 + ⌊(emin + 1) log(2, b) − log(1 − 2^(−p2), b)⌋`
///
/// For f16, this follows as:
///     emin = -14
///     p2 = 11
///
/// For bfloat16 , this follows as:
///     emin = -126
///     p2 = 8
///
/// For f32, this follows as:
///     emin = -126
///     p2 = 24
///
/// For f64, this follows as:
///     emin = -1022
///     p2 = 53
///
/// For f128, this follows as:
///     emin = -16382
///     p2 = 113
///
/// In Python:
///     `-emin + p2 + math.floor((emin+ 1)*math.log(2, b)-math.log(1-2**(-p2),
/// b))`
///
/// This was used to calculate the maximum number of digits for [2, 36].
///
/// The minimum, denormal exponent can be calculated as follows: given
/// the number of exponent bits `exp_bits`, and the number of bits
/// in the mantissa `mantissa_bits`, we have an exponent bias
/// `exp_bias` equal to `2^(exp_bits-1) - 1 + mantissa_bits`. We
/// therefore have a denormal exponent `denormal_exp` equal to
/// `1 - exp_bias` and the minimum, denormal float `min_float` is
/// therefore `2^denormal_exp`.
///
/// For f16, this follows as:
///     exp_bits = 5
///     mantissa_bits = 10
///     exp_bias = 25
///     denormal_exp = -24
///     min_float = 5.96 * 10^−8
///
/// For bfloat16, this follows as:
///     exp_bits = 8
///     mantissa_bits = 7
///     exp_bias = 134
///     denormal_exp = -133
///     min_float = 9.18 * 10^−41
///
/// For f32, this follows as:
///     exp_bits = 8
///     mantissa_bits = 23
///     exp_bias = 150
///     denormal_exp = -149
///     min_float = 1.40 * 10^−45
///
/// For f64, this follows as:
///     exp_bits = 11
///     mantissa_bits = 52
///     exp_bias = 1075
///     denormal_exp = -1074
///     min_float = 5.00 * 10^−324
///
/// For f128, this follows as:
///     exp_bits = 15
///     mantissa_bits = 112
///     exp_bias = 16495
///     denormal_exp = -16494
///     min_float = 6.48 * 10^−4966
///
/// These match statements can be generated with the following Python
/// code:
/// ```python
/// import math
///
/// def digits(emin, p2, b):
///     return -emin + p2 + math.floor((emin+ 1)*math.log(2, b)-math.log(1-2**(-p2), b))
///
/// def max_digits(emin, p2):
///     radices = [6, 10, 12, 14, 18, 20, 22, 24 26 28, 30, 34, 36]
///     print('match radix {')
///     for radix in radices:
///         value = digits(emin, p2, radix)
///         print(f'    {radix} => Some({value + 2}),')
///     print('    // Powers of two should be unreachable.')
///     print('    // Odd numbers will have infinite digits.')
///     print('    _ => None,')
///     print('}')
/// ```
#[allow(clippy::doc_markdown)] // reason="not meant to be function parameters"
pub trait MaxDigits {
    fn max_digits(radix: u32) -> Option<usize>;
}

/// emin = -126
/// p2 = 24
impl MaxDigits for f32 {
    #[inline(always)]
    fn max_digits(radix: u32) -> Option<usize> {
        debug_assert_radix(radix);
        f32_max_digits(radix)
    }
}

/// emin = -1022
/// p2 = 53
impl MaxDigits for f64 {
    #[inline(always)]
    fn max_digits(radix: u32) -> Option<usize> {
        debug_assert_radix(radix);
        f64_max_digits(radix)
    }
}

#[cfg(feature = "f16")]
impl MaxDigits for f16 {
    #[inline(always)]
    fn max_digits(_: u32) -> Option<usize> {
        unimplemented!()
    }
}

#[cfg(feature = "f16")]
impl MaxDigits for bf16 {
    #[inline(always)]
    fn max_digits(_: u32) -> Option<usize> {
        unimplemented!()
    }
}

///// `emin = -16382`
///// `p2 = 113`
//#[cfg(feature = "f128")]
//impl MaxDigits for f128 {
//    #[inline(always)]
//    fn max_digits(radix: u32) -> Option<usize> {
//        match radix {
//            6 => Some(10159),
//            10 => Some(11565),
//            12 => Some(11927),
//            14 => Some(12194),
//            18 => Some(12568),
//            20 => Some(12706),
//            22 => Some(12823),
//            24 => Some(12924),
//            26 => Some(13012),
//            28 => Some(13089),
//            30 => Some(13158),
//            34 => Some(13277),
//            36 => Some(13328),
//            // Powers of two should be unreachable.
//            // Odd numbers will have infinite digits.
//            _ => None,
//        }
//    }
//}

// CONST FN
// --------

/// Get the maximum number of significant digits as a const fn.
#[must_use]
#[inline(always)]
pub const fn f32_max_digits(radix: u32) -> Option<usize> {
    match radix {
        6 => Some(103),
        10 => Some(114),
        12 => Some(117),
        14 => Some(119),
        18 => Some(122),
        20 => Some(123),
        22 => Some(123),
        24 => Some(124),
        26 => Some(125),
        28 => Some(125),
        30 => Some(126),
        34 => Some(127),
        36 => Some(127),
        // Powers of two should be unreachable.
        // Odd numbers will have infinite digits.
        _ => None,
    }
}

/// Get the maximum number of significant digits as a const fn.
#[must_use]
#[inline(always)]
pub const fn f64_max_digits(radix: u32) -> Option<usize> {
    match radix {
        6 => Some(682),
        10 => Some(769),
        12 => Some(792),
        14 => Some(808),
        18 => Some(832),
        20 => Some(840),
        22 => Some(848),
        24 => Some(854),
        26 => Some(859),
        28 => Some(864),
        30 => Some(868),
        34 => Some(876),
        36 => Some(879),
        // Powers of two should be unreachable.
        // Odd numbers will have infinite digits.
        _ => None,
    }
}
