use std::io::{stdout, Stdout, Write};
use broccolor::TextStyle;
use crate::errors::Error;
use crate::widgets::{BoxConfig, LineStyle};

pub struct Interface {
    cursor_pos: [i32; 2],
    stdout: Stdout,
}

#[allow(dead_code)]
impl Interface {
    pub fn new() -> Self {
        Self {
            cursor_pos: [0, 0],
            stdout: stdout(),
        }
    }

    pub fn write_escape(&mut self, escape_sequence: &str) -> Result<(), Error> {
        let mut handle = self.stdout.lock();
        write!(handle, "{}", escape_sequence).map_err(|_| Error::CursorMove)?;
        handle.flush().map_err(|e| Error::Flush(e))?;
        Ok(())
    }

    pub fn move_cursor(&mut self, x: i32, y: i32) -> Result<(), Error> {
        let escape = format!("\x1B[{};{}H", y + 1, x + 1);
        self.write_escape(&escape)?;
        self.cursor_pos = [x, y];
        Ok(())
    }

    pub fn move_cursor_up(&mut self, n: i32) -> Result<(), Error> {
        let escape = format!("\x1B[{}A", n);
        self.write_escape(&escape)?;
        self.cursor_pos[1] += n;
        Ok(())
    }

    pub fn move_cursor_down(&mut self, n: i32) -> Result<(), Error> {
        let escape = format!("\x1B[{}B", n);
        self.write_escape(&escape)?;
        self.cursor_pos[1] -= n;
        Ok(())
    }

    pub fn move_cursor_forward(&mut self, n: i32) -> Result<(), Error> {
        let escape = format!("\x1B[{}C", n);
        self.write_escape(&escape)?;
        self.cursor_pos[0] += n;
        Ok(())
    }

    pub fn move_cursor_backward(&mut self, n: i32) -> Result<(), Error> {
        let escape = format!("\x1B[{}D", n);
        self.write_escape(&escape)?;
        self.cursor_pos[0] -= n;
        Ok(())
    }

    pub fn save_cursor(&mut self) -> Result<(), Error> {
        self.write_escape("\x1B[s")
    }

    pub fn restore_cursor(&mut self) -> Result<(), Error> {
        self.write_escape("\x1B[u")
    }

    pub fn scroll_up(&mut self, n: i32) -> Result<(), Error> {
        let escape = format!("\x1B[{}S", n);
        self.write_escape(&escape)
    }

    pub fn scroll_down(&mut self, n: i32) -> Result<(), Error> {
        let escape = format!("\x1B[{}T", n);
        self.write_escape(&escape)
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        self.write_escape("\x1B[2J\x1B[H")?;
        self.cursor_pos = [0, 0];
        Ok(())
    }

    pub fn clear_line(&mut self) -> Result<(), Error> {
        self.write_escape("\x1B[2K")
    }

    pub fn clear_from_cursor_to_end(&mut self) -> Result<(), Error> {
        self.write_escape("\x1B[0J")
    }

    pub fn clear_from_cursor_to_begin(&mut self) -> Result<(), Error> {
        self.write_escape("\x1B[1J")
    }
    pub fn draw_box(&mut self, config: BoxConfig) -> Result<(), Error> {
        let (x, y, width, height) = (config.x as i32, config.y as i32, config.width as i32, config.height as i32);
        let border = match config.border_style {
            LineStyle::Single => ('┌', '─', '┐', '│', '└', '┘'),
            LineStyle::Double => ('╔', '═', '╗', '║', '╚', '╝'),
            LineStyle::Rounded => ('╭', '─', '╮', '│', '╰', '╯'),
            LineStyle::Dashed => ('┌', '╌', '┐', '╎', '└', '┘'),
            LineStyle::Dotted => ('┌', '┄', '┐', '┆', '└', '┘'),
            LineStyle::Thick => ('▛', '▀', '▜', '▐', '▙', '▟'),
            LineStyle::Custom { top_left: tl, horizontal: h, top_right: tr, vertical: v, bottom_right: br, bottom_left: bl }
            => (tl, h, tr, v, br, bl),
        };

        // Apply color and background to the border characters
        let border_top_left = border.0.to_string().colorize(config.color).background(config.background);
        let border_top = border.1.to_string().repeat((width - 2) as usize).colorize(config.color).background(config.background);
        let border_top_right = border.2.to_string().colorize(config.color).background(config.background);
        let border_vertical = border.3.to_string().colorize(config.color).background(config.background);
        let border_bottom_left = border.4.to_string().colorize(config.color).background(config.background);
        let border_bottom = border.1.to_string().repeat((width - 2) as usize).colorize(config.color).background(config.background);
        let border_bottom_right = border.5.to_string().colorize(config.color).background(config.background);

        // Draw the top border
        self.move_cursor(x, y)?;
        write!(self.stdout, "{}{}{}", border_top_left, border_top, border_top_right).map_err(|_| Error::WriteError)?;

        // Draw the sides and fill the interior with the background color
        for i in 1..height - 1 {
            self.move_cursor(x, y + i)?;
            write!(self.stdout, "{}", border_vertical).map_err(|_| Error::WriteError)?;

            // Fill the interior with the background color
            let interior = " ".repeat((width - 2) as usize).background(config.background);
            write!(self.stdout, "{}", interior).map_err(|_| Error::WriteError)?;

            self.move_cursor(x + width - 1, y + i)?;
            write!(self.stdout, "{}", border_vertical).map_err(|_| Error::WriteError)?;
        }

        // Draw the bottom border
        self.move_cursor(x, y + height - 1)?;
        write!(self.stdout, "{}{}{}", border_bottom_left, border_bottom, border_bottom_right).map_err(|_| Error::WriteError)?;

        self.stdout.flush().map_err(|e| Error::Flush(e))
    }
}
