pub mod alphabetic;
pub mod whitespace;
pub mod alphanumeric;
pub mod control;
pub mod numeric;

pub use {
    alphabetic::{is_alphabetic, Alphabetic},
    alphanumeric::is_alphanumeric,
    control::is_control,
    numeric::is_numeric,
    whitespace::{is_whitespace, WhiteSpace},
};
