use std::str::FromStr;
use crate::Color;

#[allow(dead_code)]
pub trait ColorConversion {
    fn to_rgba(&self) -> Option<(f32, f32, f32, f32)>;
    fn to_rgba_u8(&self) -> Option<(u8,u8,u8,u8)>;
    fn to_rgba_array(&self) -> Option<[f32;4]>;
    fn to_hex(&self) -> Option<String>;
    fn to_grayscale(&self) -> Option<u8>;
    fn to_css(&self) -> Option<String>;
    fn to_ansi_code(&self) -> String;
    fn to_background_ansi_code(&self) -> String;
    fn indexed_to_rgba(index: u8) -> (f32, f32, f32, f32);
    fn from_hex(hex: &str) -> Result<Color, String>;
}

impl ColorConversion for Color {
    fn to_rgba(&self) -> Option<(f32, f32, f32, f32)> {
        match *self {
            Color::Rgb(r, g, b) => Some((
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                1.0,
            )),
            Color::Rgba(r, g, b, a) => Some((
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
            )),
            Color::Hex(ref hex) => {
                if hex.len() == 7 || hex.len() == 9 {
                    let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
                    let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
                    let b = u8::from_str_radix(&hex[5..7], 16).ok()?;
                    let a = if hex.len() == 9 {
                        u8::from_str_radix(&hex[7..9], 16).ok()?
                    } else {
                        255
                    };
                    Some((
                        r as f32 / 255.0,
                        g as f32 / 255.0,
                        b as f32 / 255.0,
                        a as f32 / 255.0,
                    ))
                } else {
                    None
                }
            }
            Color::Gray(g) => Some((
                g as f32 / 255.0,
                g as f32 / 255.0,
                g as f32 / 255.0,
                1.0,
            )),
            Color::Indexed(n) => {
                match n {
                    0..=7 => match n {
                        0 => Some((0.0, 0.0, 0.0, 1.0)),
                        1 => Some((1.0, 0.0, 0.0, 1.0)),
                        2 => Some((0.0, 1.0, 0.0, 1.0)),
                        3 => Some((1.0, 1.0, 0.0, 1.0)),
                        4 => Some((0.0, 0.0, 1.0, 1.0)),
                        5 => Some((1.0, 0.0, 1.0, 1.0)),
                        6 => Some((0.0, 1.0, 1.0, 1.0)),
                        7 => Some((1.0, 1.0, 1.0, 1.0)),
                        _ => None
                    },
                    232..=255 => {
                        let intensity = (n - 232) as f32 / 23.0;
                        Some((intensity, intensity, intensity, 1.0))
                    },
                    _ => {
                        Some((0.5, 0.5, 0.5, 1.0))
                    }
                }
            },
            Color::Transparent => Some((0.0, 0.0, 0.0, 0.0)),
            Color::Red => Some((1.0, 0.0, 0.0, 1.0)),
            Color::Green => Some((0.0, 1.0, 0.0, 1.0)),
            Color::Blue => Some((0.0, 0.0, 1.0, 1.0)),
            Color::Yellow => Some((1.0, 1.0, 0.0, 1.0)),
            Color::Magenta => Some((1.0, 0.0, 1.0, 1.0)),
            Color::Cyan => Some((0.0, 1.0, 1.0, 1.0)),
            Color::White => Some((1.0, 1.0, 1.0, 1.0)),
            Color::Black => Some((0.0, 0.0, 0.0, 1.0)),
            Color::Orange => Some((1.0, 0.647, 0.0, 1.0)),
            Color::Pink => Some((1.0, 0.75, 0.8, 1.0)),
            Color::Lime => Some((0.0, 1.0, 0.0, 1.0)),
            Color::Indigo => Some((0.294, 0.0, 0.51, 1.0)),
            Color::Violet => Some((0.933, 0.51, 0.933, 1.0)),
            Color::Turquoise => Some((0.25, 0.88, 0.82, 1.0)),
            Color::Teal => Some((0.0, 0.5, 0.5, 1.0)),
            Color::Mint => Some((0.68, 1.0, 0.65, 1.0)),
            Color::Coral => Some((1.0, 0.5, 0.31, 1.0)),
            Color::Charcoal => Some((0.2, 0.2, 0.2, 1.0)),
            Color::BrightRed => Some((1.0, 0.4, 0.4, 1.0)),
            Color::BrightGreen => Some((0.4, 1.0, 0.4, 1.0)),
            Color::BrightYellow => Some((1.0, 1.0, 0.4, 1.0)),
            Color::BrightBlue => Some((0.4, 0.4, 1.0, 1.0)),
            Color::BrightMagenta => Some((1.0, 0.4, 1.0, 1.0)),
            Color::BrightCyan => Some((0.4, 1.0, 1.0, 1.0)),
            Color::BrightWhite => Some((1.0, 1.0, 1.0, 1.0)),
            Color::BrightBlack => Some((0.2, 0.2, 0.2, 1.0)),
            Color::Crimson => Some((0.86, 0.08, 0.24, 1.0)),
            Color::Gold => Some((1.0, 0.84, 0.0, 1.0)),
            Color::Silver => Some((0.75, 0.75, 0.75, 1.0)),
            Color::Bronze => Some((0.8, 0.5, 0.2, 1.0)),
            Color::LightGray => Some((0.8, 0.8, 0.8, 1.0)),
            Color::DarkGray => Some((0.25, 0.25, 0.25, 1.0)),
            Color::SlateGray => Some((0.44, 0.5, 0.56, 1.0)),
        }
    }

    fn to_rgba_u8(&self) -> Option<(u8,u8,u8,u8)> {
        let (r,g,b,a) = self.to_rgba()?;

        Some(((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, (a * 255.0) as u8))
    }

    fn to_rgba_array(&self) -> Option<[f32;4]> {
        let (r,g,b,a) = self.to_rgba()?;

        Some([r, g, b, a])
    }
    fn to_hex(&self) -> Option<String> {
        let (r, g, b ,a) = self.to_rgba_u8()?;

        Some(format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a))
    }

    fn to_grayscale(&self) -> Option<u8> {
        let (r, g, b, _) = self.to_rgba_u8()?;

        Some(((r as u16 + g as u16 + b as u16) / 3) as u8)
    }

    fn to_css(&self) -> Option<String> {
        match *self {
            Color::Rgb(r, g, b) => Some(format!("rgb({}, {}, {})", r, g, b)),
            _ => {
                let (r, g, b, a) = self.to_rgba_u8()?;

                Some(format!("rgba({}, {}, {}, {})", r, g, b, a as f32 / 255.0))
            },
        }
    }

    fn to_ansi_code(&self) -> String {
        match *self {
            Color::Red => "\x1b[31m".to_string(),
            Color::Green => "\x1b[32m".to_string(),
            Color::Yellow => "\x1b[33m".to_string(),
            Color::Blue => "\x1b[34m".to_string(),
            Color::Magenta => "\x1b[35m".to_string(),
            Color::Cyan => "\x1b[36m".to_string(),
            Color::White => "\x1b[37m".to_string(),
            Color::Black => "\x1b[30m".to_string(),

            Color::BrightRed => "\x1b[91m".to_string(),
            Color::BrightGreen => "\x1b[92m".to_string(),
            Color::BrightYellow => "\x1b[93m".to_string(),
            Color::BrightBlue => "\x1b[94m".to_string(),
            Color::BrightMagenta => "\x1b[95m".to_string(),
            Color::BrightCyan => "\x1b[96m".to_string(),
            Color::BrightWhite => "\x1b[97m".to_string(),
            Color::BrightBlack => "\x1b[90m".to_string(),

            Color::Orange => "\x1b[38;5;208m".to_string(),
            Color::Pink => "\x1b[38;5;213m".to_string(),
            Color::Teal => "\x1b[38;5;37m".to_string(),
            Color::Violet => "\x1b[38;5;177m".to_string(),
            Color::Indigo => "\x1b[38;5;54m".to_string(),
            Color::Lime => "\x1b[38;5;154m".to_string(),
            Color::Turquoise => "\x1b[38;5;80m".to_string(),
            Color::Coral => "\x1b[38;5;203m".to_string(),
            Color::Crimson => "\x1b[38;5;161m".to_string(),
            Color::Mint => "\x1b[38;5;121m".to_string(),
            Color::Gold => "\x1b[38;5;220m".to_string(),
            Color::Silver => "\x1b[38;5;250m".to_string(),
            Color::Bronze => "\x1b[38;5;136m".to_string(),

            Color::LightGray => "\x1b[38;5;250m".to_string(),
            Color::DarkGray => "\x1b[38;5;238m".to_string(),
            Color::SlateGray => "\x1b[38;5;241m".to_string(),
            Color::Charcoal => "\x1b[38;5;232m".to_string(),

            Color::Transparent => "\x1b[39m".to_string(),

            Color::Rgb(r, g, b) => format!("\x1b[38;2;{};{};{}m", r, g, b),
            Color::Indexed(n) => format!("\x1b[38;5;{}m", n),
            Color::Gray(g) => {
                let gray_index = 232 + (g.min(23));
                format!("\x1b[38;5;{}m", gray_index)
            }

            Color::Rgba(r, g, b, _) => format!("\x1b[38;2;{};{};{}m", r, g, b),

            Color::Hex(_) => {
                if let Some((r, g, b, _)) = self.to_rgba() {
                    format!("\x1b[38;2;{};{};{}m", r, g, b)
                } else {
                    "\x1b[39m".to_string()
                }
            }
        }
    }

    fn to_background_ansi_code(&self) -> String {
        match *self {
            Color::Red => "\x1b[41m".to_string(),
            Color::Green => "\x1b[42m".to_string(),
            Color::Yellow => "\x1b[43m".to_string(),
            Color::Blue => "\x1b[44m".to_string(),
            Color::Magenta => "\x1b[45m".to_string(),
            Color::Cyan => "\x1b[46m".to_string(),
            Color::White => "\x1b[47m".to_string(),
            Color::Black => "\x1b[40m".to_string(),

            Color::BrightRed => "\x1b[101m".to_string(),
            Color::BrightGreen => "\x1b[102m".to_string(),
            Color::BrightYellow => "\x1b[103m".to_string(),
            Color::BrightBlue => "\x1b[104m".to_string(),
            Color::BrightMagenta => "\x1b[105m".to_string(),
            Color::BrightCyan => "\x1b[106m".to_string(),
            Color::BrightWhite => "\x1b[107m".to_string(),
            Color::BrightBlack => "\x1b[100m".to_string(),

            Color::Orange => "\x1b[48;5;208m".to_string(),
            Color::Pink => "\x1b[48;5;213m".to_string(),
            Color::Teal => "\x1b[48;5;37m".to_string(),
            Color::Violet => "\x1b[48;5;177m".to_string(),
            Color::Indigo => "\x1b[48;5;54m".to_string(),
            Color::Lime => "\x1b[48;5;154m".to_string(),
            Color::Turquoise => "\x1b[48;5;80m".to_string(),
            Color::Coral => "\x1b[48;5;203m".to_string(),
            Color::Crimson => "\x1b[48;5;161m".to_string(),
            Color::Mint => "\x1b[48;5;121m".to_string(),
            Color::Gold => "\x1b[48;5;220m".to_string(),
            Color::Silver => "\x1b[48;5;250m".to_string(),
            Color::Bronze => "\x1b[48;5;136m".to_string(),

            Color::LightGray => "\x1b[48;5;250m".to_string(),
            Color::DarkGray => "\x1b[48;5;238m".to_string(),
            Color::SlateGray => "\x1b[48;5;241m".to_string(),
            Color::Charcoal => "\x1b[48;5;232m".to_string(),

            Color::Transparent => "\x1b[49m".to_string(),

            Color::Rgb(r, g, b) => format!("\x1b[48;2;{};{};{}m", r, g, b),
            Color::Indexed(n) => format!("\x1b[48;5;{}m", n),
            Color::Gray(g) => {
                let gray_index = 232 + (g.min(23));
                format!("\x1b[48;5;{}m", gray_index)
            }

            Color::Rgba(r, g, b, _) => format!("\x1b[48;2;{};{};{}m", r, g, b),

            Color::Hex(ref hex) => {
                if hex.len() == 6 {
                    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                    format!("\x1b[48;2;{};{};{}m", r, g, b)
                } else {
                    "\x1b[49m".to_string()
                }
            }
        }
    }

    fn indexed_to_rgba(index: u8) -> (f32, f32, f32, f32) {
        if index < 16 {
            // Standard ANSI colors
            const ANSI_COLORS: [(u8, u8, u8); 16] = [
                (0, 0, 0), (128, 0, 0), (0, 128, 0), (128, 128, 0),
                (0, 0, 128), (128, 0, 128), (0, 128, 128), (192, 192, 192),
                (128, 128, 128), (255, 0, 0), (0, 255, 0), (255, 255, 0),
                (0, 0, 255), (255, 0, 255), (0, 255, 255), (255, 255, 255),
            ];
            let (r, g, b) = ANSI_COLORS[index as usize];
            (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
        } else if (16..=231).contains(&index) {
            // Convert 16-231 to a 6x6x6 RGB cube
            let i = index - 16;
            let r = (i / 36) % 6;
            let g = (i / 6) % 6;
            let b = i % 6;
            let scale = [0, 95, 135, 175, 215, 255];
            return (
                scale[r as usize] as f32 / 255.0,
                scale[g as usize] as f32 / 255.0,
                scale[b as usize] as f32 / 255.0,
                1.0,
            );
        } else {
            let shade = 8 + (index - 232) * 10;
            return (shade as f32 / 255.0, shade as f32 / 255.0, shade as f32 / 255.0, 1.0);
        }
    }

    fn from_hex(hex: &str) -> Result<Color, String> {
        let hex = hex.trim_start_matches('#');

        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex")?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex")?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex")?;
                Ok(Color::Rgba(r, g, b, 255))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex")?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex")?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex")?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| "Invalid hex")?;
                Ok(Color::Rgba(r, g, b, a))
            }
            _ => Err("Invalid hex format. Use #RRGGBB or #RRGGBBAA.".to_string()),
        }
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        let colors = vec![
            ("red", Color::Red),
            ("green", Color::Green),
            ("yellow", Color::Yellow),
            ("blue", Color::Blue),
            ("magenta", Color::Magenta),
            ("cyan", Color::Cyan),
            ("white", Color::White),
            ("black", Color::Black),
            ("bright red", Color::BrightRed),
            ("bright green", Color::BrightGreen),
            ("bright yellow", Color::BrightYellow),
            ("bright blue", Color::BrightBlue),
            ("bright magenta", Color::BrightMagenta),
            ("bright cyan", Color::BrightCyan),
            ("bright white", Color::BrightWhite),
            ("bright black", Color::BrightBlack),
            ("orange", Color::Orange),
            ("pink", Color::Pink),
            ("teal", Color::Teal),
            ("violet", Color::Violet),
            ("indigo", Color::Indigo),
            ("lime", Color::Lime),
            ("turquoise", Color::Turquoise),
            ("coral", Color::Coral),
            ("crimson", Color::Crimson),
            ("mint", Color::Mint),
            ("gold", Color::Gold),
            ("silver", Color::Silver),
            ("bronze", Color::Bronze),
            ("light gray", Color::LightGray),
            ("dark gray", Color::DarkGray),
            ("slate gray", Color::SlateGray),
            ("charcoal", Color::Charcoal),
            ("transparent", Color::Transparent),
        ];

        if let Some((_, color)) = colors.iter().find(|(name, _)| *name == s) {
            return Ok(*color);
        }

        if s.starts_with('#') && (s.len() == 7 || s.len() == 9) {
            return Ok(Color::Hex(Box::leak(s.into_boxed_str())));
        }

        Err(format!("Unknown color: '{}'", s))
    }
}
