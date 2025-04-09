//! # UNIC — Unicode Character Tools — Character Property
//!
//! A component of [`unic`: Unicode and Internationalization Crates for Rust](/unic/).
//!
//! Character Property taxonomy, contracts and build macros.
//!
//! ## References
//!
//! * [Unicode UTR #23: The Unicode Character Property Model](http://unicode.org/reports/tr23/).
//!
//! * [Unicode UAX #44: Unicode Character Database](http://unicode.org/reports/tr44/).
//!
//! * [PropertyAliases.txt](https://www.unicode.org/Public/UCD/latest/ucd/PropertyAliases.txt).

mod property;
pub use self::property::{CharProperty, PartialCharProperty, TotalCharProperty};

mod range_types;
pub use range_types::{
    BinaryCharProperty,
    CustomCharProperty,
    EnumeratedCharProperty,
    NumericCharProperty,
    NumericCharPropertyValue,
};

mod macros;

// pub because is used in macros, called from macro call-site.
pub mod tables;

// Used in macros
#[doc(hidden)]
pub use core::{fmt as __fmt, str as __str};
