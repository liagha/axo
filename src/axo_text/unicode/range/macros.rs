#[macro_export]
macro_rules! chars {
    ( $low:tt .. $high:tt ) => {
        crate::axo_text::CharRange::open_right($low, $high)
    };
    ( $low:tt ..= $high:tt ) => {
        $crate::axo_text::CharRange {
            low: $low,
            high: $high,
        }
    };
    ( .. ) => {
        crate::axo_text::CharRange::all()
    };
}
