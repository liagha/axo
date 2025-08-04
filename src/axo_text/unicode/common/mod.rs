pub mod alphabetic;
pub use alphabetic::{is_alphabetic, Alphabetic};

pub mod whitespace;
pub use whitespace::{is_whitespace, WhiteSpace};

pub mod alphanumeric;
pub use alphanumeric::is_alphanumeric;

pub mod control;
pub use control::is_control;

pub mod numeric;
pub use numeric::is_numeric;
