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

pub mod tables;

#[doc(hidden)]
pub use crate::{format as __fmt, string as __str};
