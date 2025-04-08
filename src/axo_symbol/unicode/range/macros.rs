#[macro_export]
/// Convenience macro for the initialization of `CharRange`s.
///
/// # Syntax
///
/// ```
/// # #[macro_use] extern crate unic_char_range;
/// # fn main() {
/// chars!('a'..'z'); // The half open range including 'a' and excluding 'z'
/// chars!('a'..='z'); // The closed range including 'a' and including 'z'
/// chars!(..); // All characters
/// # }
/// ```
///
/// `chars!('a'..='z')` and `chars!(..)` are constant-time expressions, and can be used
/// where such are required, such as in the initialization of constant data structures.
///
/// NOTE: Because an `expr` capture cannot be followed by a `..`/`..=`, this macro captures token
/// trees. This means that if you want to pass more than one token, you must parenthesize it (e.g.
/// `chars!('\0' ..= (char::MAX))`).
macro_rules! chars {
    ( $low:tt .. $high:tt ) => {
        crate::axo_symbol::CharRange::open_right($low, $high)
    };
    ( $low:tt ..= $high:tt ) => {
        $crate::axo_symbol::CharRange {
            low: $low,
            high: $high,
        }
    };
    ( .. ) => {
        crate::axo_symbol::CharRange::all()
    };
}
