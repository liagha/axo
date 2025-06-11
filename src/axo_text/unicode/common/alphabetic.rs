
use {
    crate::{
        chars,
        char_property,
    }
};

char_property! {
    pub struct Alphabetic(bool) {
        abbr => "Alpha";
        long => "Alphabetic";
        human => "Alphabetic";

        data_table_path => "tables/alphabetic.rsv";
    }

    pub fn is_alphabetic(char) -> bool;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_values() {
        use super::is_alphabetic;

        assert_eq!(is_alphabetic('\u{0020}'), false);
        assert_eq!(is_alphabetic('\u{0021}'), false);
        assert_eq!(is_alphabetic('\u{0022}'), false);

        assert_eq!(is_alphabetic('\u{0030}'), false);
        assert_eq!(is_alphabetic('\u{0031}'), false);
        assert_eq!(is_alphabetic('\u{0032}'), false);

        assert_eq!(is_alphabetic('\u{0040}'), false);
        assert_eq!(is_alphabetic('\u{0041}'), true);
        assert_eq!(is_alphabetic('\u{0042}'), true);

        assert_eq!(is_alphabetic('\u{0060}'), false);
        assert_eq!(is_alphabetic('\u{0061}'), true);
        assert_eq!(is_alphabetic('\u{0062}'), true);

        assert_eq!(is_alphabetic('\u{007e}'), false);
        assert_eq!(is_alphabetic('\u{007f}'), false);

        assert_eq!(is_alphabetic('\u{061b}'), false);
        assert_eq!(is_alphabetic('\u{061c}'), false);
        assert_eq!(is_alphabetic('\u{061d}'), false);

        assert_eq!(is_alphabetic('\u{200d}'), false);
        assert_eq!(is_alphabetic('\u{200e}'), false);
        assert_eq!(is_alphabetic('\u{200f}'), false);
        assert_eq!(is_alphabetic('\u{2010}'), false);

        assert_eq!(is_alphabetic('\u{2029}'), false);
        assert_eq!(is_alphabetic('\u{202a}'), false);
        assert_eq!(is_alphabetic('\u{202e}'), false);
        assert_eq!(is_alphabetic('\u{202f}'), false);

        assert_eq!(is_alphabetic('\u{10000}'), true);
        assert_eq!(is_alphabetic('\u{10001}'), true);

        assert_eq!(is_alphabetic('\u{20000}'), true);
        assert_eq!(is_alphabetic('\u{30000}'), false);
        assert_eq!(is_alphabetic('\u{40000}'), false);
        assert_eq!(is_alphabetic('\u{50000}'), false);
        assert_eq!(is_alphabetic('\u{60000}'), false);
        assert_eq!(is_alphabetic('\u{70000}'), false);
        assert_eq!(is_alphabetic('\u{80000}'), false);
        assert_eq!(is_alphabetic('\u{90000}'), false);
        assert_eq!(is_alphabetic('\u{a0000}'), false);
        assert_eq!(is_alphabetic('\u{b0000}'), false);
        assert_eq!(is_alphabetic('\u{c0000}'), false);
        assert_eq!(is_alphabetic('\u{d0000}'), false);
        assert_eq!(is_alphabetic('\u{e0000}'), false);

        assert_eq!(is_alphabetic('\u{efffe}'), false);
        assert_eq!(is_alphabetic('\u{effff}'), false);

        assert_eq!(is_alphabetic('\u{f0000}'), false);
        assert_eq!(is_alphabetic('\u{f0001}'), false);
        assert_eq!(is_alphabetic('\u{ffffe}'), false);
        assert_eq!(is_alphabetic('\u{fffff}'), false);
        assert_eq!(is_alphabetic('\u{100000}'), false);
        assert_eq!(is_alphabetic('\u{100001}'), false);
        assert_eq!(is_alphabetic('\u{10fffe}'), false);
        assert_eq!(is_alphabetic('\u{10ffff}'), false);
    }
}
