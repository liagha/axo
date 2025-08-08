#[macro_export]
macro_rules! chars {
    ( $low:tt .. $high:tt ) => {
        crate::text::CharRange::open_right($low, $high)
    };
    ( $low:tt ..= $high:tt ) => {
        $crate::text::CharRange {
            low: $low,
            high: $high,
        }
    };
    ( .. ) => {
        crate::text::CharRange::all()
    };
}
