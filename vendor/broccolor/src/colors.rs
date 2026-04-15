#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Color {
    // Primary Colors
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Black,

    // Bright Variants
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    BrightBlack,

    // Cool Specific Colors
    Orange,
    Pink,
    Teal,
    Violet,
    Indigo,
    Lime,
    Turquoise,
    Coral,
    Crimson,
    Mint,
    Gold,
    Silver,
    Bronze,

    // Named Gray Tones
    LightGray,
    DarkGray,
    SlateGray,
    Charcoal,

    // RGB, RGBA, Hex, Indexed and Transparent
    Transparent,
    Rgb(u8, u8, u8),       // Custom RGB
    Rgba(u8, u8, u8, u8),  // Custom RGBA
    Hex(&'static str),     // Hexadecimal Color Code
    Indexed(u8),           // ANSI Indexed (0–255)
    Gray(u8),              // Gray shades (0–23)
}

impl Default for Color {
    fn default() -> Self {
        Color::White
    }
}

#[allow(dead_code)]
impl Color {
    pub fn reset() -> &'static str {
        "\x1b[0m"
    }

    pub fn bold() -> &'static str {
        "\x1b[1m"
    }

    pub fn italic() -> &'static str {
        "\x1b[3m"
    }

    pub fn underline() -> &'static str {
        "\x1b[4m"
    }

    pub fn strikethrough() -> &'static str {
        "\x1b[9m"
    }

    pub fn reset_style() -> &'static str {
        "\x1b[22m"
    }
}