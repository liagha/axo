
use {
    crate::{
        chars,
        char_property,
    }
};

char_property! {
    pub struct WhiteSpace(bool) {
        abbr => "WSpace";
        long => "White_Space";
        human => "White Space";

        data_table_path => "tables/whitespace.rsv";
    }

    pub fn is_whitespace(char) -> bool;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_values() {
        use super::is_whitespace;

        assert_eq!(is_whitespace('\u{0020}'), true);
        assert_eq!(is_whitespace('\u{0021}'), false);
        assert_eq!(is_whitespace('\u{0022}'), false);

        assert_eq!(is_whitespace('\u{0030}'), false);
        assert_eq!(is_whitespace('\u{0031}'), false);
        assert_eq!(is_whitespace('\u{0032}'), false);

        assert_eq!(is_whitespace('\u{0040}'), false);
        assert_eq!(is_whitespace('\u{0041}'), false);
        assert_eq!(is_whitespace('\u{0042}'), false);

        assert_eq!(is_whitespace('\u{0060}'), false);
        assert_eq!(is_whitespace('\u{0061}'), false);
        assert_eq!(is_whitespace('\u{0062}'), false);

        assert_eq!(is_whitespace('\u{007e}'), false);
        assert_eq!(is_whitespace('\u{007f}'), false);

        assert_eq!(is_whitespace('\u{061b}'), false);
        assert_eq!(is_whitespace('\u{061c}'), false);
        assert_eq!(is_whitespace('\u{061d}'), false);

        assert_eq!(is_whitespace('\u{200d}'), false);
        assert_eq!(is_whitespace('\u{200e}'), false);
        assert_eq!(is_whitespace('\u{200f}'), false);
        assert_eq!(is_whitespace('\u{2010}'), false);

        assert_eq!(is_whitespace('\u{2029}'), true);
        assert_eq!(is_whitespace('\u{202a}'), false);
        assert_eq!(is_whitespace('\u{202e}'), false);
        assert_eq!(is_whitespace('\u{202f}'), true);

        assert_eq!(is_whitespace('\u{10000}'), false);
        assert_eq!(is_whitespace('\u{10001}'), false);

        assert_eq!(is_whitespace('\u{20000}'), false);
        assert_eq!(is_whitespace('\u{30000}'), false);
        assert_eq!(is_whitespace('\u{40000}'), false);
        assert_eq!(is_whitespace('\u{50000}'), false);
        assert_eq!(is_whitespace('\u{60000}'), false);
        assert_eq!(is_whitespace('\u{70000}'), false);
        assert_eq!(is_whitespace('\u{80000}'), false);
        assert_eq!(is_whitespace('\u{90000}'), false);
        assert_eq!(is_whitespace('\u{a0000}'), false);
        assert_eq!(is_whitespace('\u{b0000}'), false);
        assert_eq!(is_whitespace('\u{c0000}'), false);
        assert_eq!(is_whitespace('\u{d0000}'), false);
        assert_eq!(is_whitespace('\u{e0000}'), false);

        assert_eq!(is_whitespace('\u{efffe}'), false);
        assert_eq!(is_whitespace('\u{effff}'), false);

        assert_eq!(is_whitespace('\u{f0000}'), false);
        assert_eq!(is_whitespace('\u{f0001}'), false);
        assert_eq!(is_whitespace('\u{ffffe}'), false);
        assert_eq!(is_whitespace('\u{fffff}'), false);
        assert_eq!(is_whitespace('\u{100000}'), false);
        assert_eq!(is_whitespace('\u{100001}'), false);
        assert_eq!(is_whitespace('\u{10fffe}'), false);
        assert_eq!(is_whitespace('\u{10ffff}'), false);
    }
}
