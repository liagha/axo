use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};

pub fn get_terminal_size() -> Result<(u16, u16), std::io::Error> {
    let mut size: winsize = unsafe { std::mem::zeroed() };

    if unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size) } == -1 {
        return Err(std::io::Error::last_os_error());
    }

    Ok((size.ws_col, size.ws_row))
}
