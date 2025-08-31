mod property;
pub use self::property::{CharProperty, PartialCharProperty, TotalCharProperty};

mod range;
pub use range::{
    BinaryCharProperty,
    CustomCharProperty,
    EnumeratedCharProperty,
    NumericCharProperty,
    NumericCharPropertyValue,
};

mod macros;

pub mod tables;

#[doc(hidden)]
pub(super) use {
    crate::{format as __fmt, data as __str},
};
