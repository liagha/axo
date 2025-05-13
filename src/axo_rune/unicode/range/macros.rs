#[macro_export]
macro_rules! chars {
    ( $low:tt .. $high:tt ) => {
        crate::axo_rune::CharRange::open_right($low, $high)
    };
    ( $low:tt ..= $high:tt ) => {
        $crate::axo_rune::CharRange {
            low: $low,
            high: $high,
        }
    };
    ( .. ) => {
        crate::axo_rune::CharRange::all()
    };
}
