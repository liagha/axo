use broccolor::Color;

#[derive(Debug, Clone, Copy)]
pub struct BoxConfig {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub border_style: LineStyle,
    pub color: Color,
    pub background: Color,
}

#[derive(Debug, Clone, Copy)]
pub enum LineStyle {
    Single,
    Double,
    Rounded,
    Dashed,
    Dotted,
    Thick,
    Custom {
        top_left: char,
        horizontal: char,
        top_right: char,
        vertical: char,
        bottom_right: char,
        bottom_left: char,
    },
}