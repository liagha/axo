use crate::Color;
use crate::conversion::ColorConversion;

use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub struct ColoredText<T> {
    pub content: T,
    pub color: Color,
}

impl<T: Display> Display for ColoredText<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.content.to_string().term_colorize(self.color))
    }
}

impl<T: Debug> Debug for ColoredText<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", format!("{:?}", self.content).term_colorize(self.color))
    }
}

impl<T: Display + Debug> TextStyle for T {
    fn colorize(&self, color: Color) -> String {
        format!("{}{}{}", color.to_ansi_code(), self, Color::reset())
    }

    fn term_colorize(&self, color: Color) -> String {
        if !cfg!(target_arch = "wasm32") {
            if color_support() {
                return format!("{}{}{}", color.to_ansi_code(), self, Color::reset())
            }
        }

        self.to_string()
    }

    fn background(&self, color: Color) -> String {
        format!("{}{}{}", color.to_background_ansi_code(), self, Color::reset())
    }

    fn bold(&self) -> String {
        format!("{}{}{}", Color::bold(), self, Color::reset())
    }

    fn italic(&self) -> String {
        format!("{}{}{}", Color::italic(), self, Color::reset())
    }

    fn underline(&self) -> String {
        format!("{}{}{}", Color::underline(), self, Color::reset())
    }

    fn strikethrough(&self) -> String {
        format!("{}{}{}", Color::strikethrough(), self, Color::reset())
    }
}

#[allow(dead_code)]
pub trait TextStyle {
    fn colorize(&self, color: Color) -> String;
    fn term_colorize(&self, color: Color) -> String;
    fn background(&self, color: Color) -> String;
    fn bold(&self) -> String;
    fn italic(&self) -> String;
    fn underline(&self) -> String;
    fn strikethrough(&self) -> String;
}

fn color_support() -> bool {
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("xterm") || term.contains("screen") || term.contains("vt100") {
            true
        } else {
            false
        }
    } else {
        if let Ok(_) = std::env::var("COLORTERM") {
            true
        } else {
            false
        }
    }
}
