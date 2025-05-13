use {
    crate::{
        chars,
        char_property,
    }
};

char_property! {
    pub struct Control(bool) {
        abbr => "Control";
        long => "Control";
        human => "Control";

        data_table_path => "tables/control.rsv";
    }

    pub fn is_control(char) -> bool;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_values() {
        use super::is_control;

        assert_eq!(is_control('\u{0000}'), true);
        assert_eq!(is_control('\u{0001}'), true);
        assert_eq!(is_control('\u{0002}'), true);

        assert_eq!(is_control('\u{0010}'), true);
        assert_eq!(is_control('\u{0011}'), true);
        assert_eq!(is_control('\u{0012}'), true);

        assert_eq!(is_control('\u{0020}'), false);
        assert_eq!(is_control('\u{0021}'), false);
        assert_eq!(is_control('\u{0022}'), false);

        assert_eq!(is_control('\u{0030}'), false);
        assert_eq!(is_control('\u{0031}'), false);
        assert_eq!(is_control('\u{0032}'), false);

        assert_eq!(is_control('\u{0040}'), false);
        assert_eq!(is_control('\u{0041}'), false);
        assert_eq!(is_control('\u{0042}'), false);

        assert_eq!(is_control('\u{0060}'), false);
        assert_eq!(is_control('\u{0061}'), false);
        assert_eq!(is_control('\u{0062}'), false);

        assert_eq!(is_control('\u{007e}'), false);
        assert_eq!(is_control('\u{007f}'), true);

        assert_eq!(is_control('\u{061b}'), false);
        assert_eq!(is_control('\u{061c}'), false);
        assert_eq!(is_control('\u{061d}'), false);

        assert_eq!(is_control('\u{200d}'), false);
        assert_eq!(is_control('\u{200e}'), false);
        assert_eq!(is_control('\u{200f}'), false);
        assert_eq!(is_control('\u{2010}'), false);

        assert_eq!(is_control('\u{2029}'), false);
        assert_eq!(is_control('\u{202a}'), false);
        assert_eq!(is_control('\u{202e}'), false);
        assert_eq!(is_control('\u{202f}'), false);

        assert_eq!(is_control('\u{10000}'), false);
        assert_eq!(is_control('\u{10001}'), false);

        assert_eq!(is_control('\u{20000}'), false);
        assert_eq!(is_control('\u{30000}'), false);
        assert_eq!(is_control('\u{40000}'), false);
        assert_eq!(is_control('\u{50000}'), false);
        assert_eq!(is_control('\u{60000}'), false);
        assert_eq!(is_control('\u{70000}'), false);
        assert_eq!(is_control('\u{80000}'), false);
        assert_eq!(is_control('\u{90000}'), false);
        assert_eq!(is_control('\u{a0000}'), false);
        assert_eq!(is_control('\u{b0000}'), false);
        assert_eq!(is_control('\u{c0000}'), false);
        assert_eq!(is_control('\u{d0000}'), false);
        assert_eq!(is_control('\u{e0000}'), false);

        assert_eq!(is_control('\u{efffe}'), false);
        assert_eq!(is_control('\u{effff}'), false);

        assert_eq!(is_control('\u{f0000}'), false);
        assert_eq!(is_control('\u{f0001}'), false);
        assert_eq!(is_control('\u{ffffe}'), false);
        assert_eq!(is_control('\u{fffff}'), false);
        assert_eq!(is_control('\u{100000}'), false);
        assert_eq!(is_control('\u{100001}'), false);
        assert_eq!(is_control('\u{10fffe}'), false);
        assert_eq!(is_control('\u{10ffff}'), false);
    }
}
