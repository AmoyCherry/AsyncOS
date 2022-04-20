use core::fmt::{self, Write};

const STDIN: usize = 0;
const STDOUT: usize = 1;

use super::{read, write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write(STDOUT, s.as_bytes());
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

pub fn getchar() -> u8 {
    let mut c = [0u8; 1];
    read(STDIN, &mut c);
    c[0]
}


#[macro_export]
macro_rules! colorize {
    ($content: ident, $foreground_color: ident) => {
        format_args!("\x1b[{}m{}\x1b[0m", $foreground_color as u8, $content)
    };
    ($content: ident, $foreground_color: ident, $background_color: ident) => {
        format_args!(
            "\x1b[{}m\x1b[{}m{}\x1b[0m",
            $foreground_color as u8, $background_color as u8, $content
        )
    };
}



/// Use colorize! to print with color
pub fn print_colorized(args: fmt::Arguments, foreground_color: u8, background_color: u8) {
    Stdout.write_fmt(colorize!(args, foreground_color, background_color)).unwrap();
}

#[macro_export]
macro_rules! print_colorized {
    ($fmt: literal, $foreground_color: expr, $background_color: expr $(, $($arg: tt)+)?) => {
        $crate::console::print_colorized(format_args!($fmt $(, $($arg)+)?), $foreground_color as u8, $background_color as u8);
    };
}

#[macro_export]
macro_rules! println_colorized {
    ($fmt: literal, $foreground_color: expr, $background_color: expr $(, $($arg: tt)+)?) => {
        $crate::console::print_colorized(format_args!(concat!($fmt, "\r\n") $(, $($arg)+)?), $foreground_color as u8, $background_color as u8);
    }
}

#[macro_export]
macro_rules! println_hart {
    ($fmt: literal, $hart_id: expr $(, $($arg: tt)+)?) => {
        $crate::console::print_colorized(format_args!(concat!("[hart {}]", $fmt, "\r\n"), $hart_id $(, $($arg)+)?), 93 + $hart_id as u8, 49u8);
    };
}