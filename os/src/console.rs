use core::fmt::{self, Write};
use crate::sbi::console_putchar;
use alloc::sync::Arc;
use lazy_static::*;
use spin::Mutex;

// struct Stdout;

// lazy_static! {
//     static ref STDOUT: Mutex<Stdout> = Mutex::new(Stdout);
// }

// impl Write for Stdout {
//     fn write_str(&mut self, s: &str) -> fmt::Result {
//         for c in s.chars() {
//             console_putchar(c as usize);
//         }
//         Ok(())
//     }
// }



// pub fn print(args: fmt::Arguments) {
//     Stdout.write_fmt(args).unwrap();
// }

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

#[macro_export]
macro_rules! info {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("\x1b[33m{}\x1b[0m\n", format_args!($($arg)*)));
}


#[macro_export]
macro_rules! debug {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("\x1b[34m{}\x1b[0m\n", format_args!($($arg)*)));
}


#[macro_export]
macro_rules! warn {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("\x1b[31m{}\x1b[0m\n", format_args!($($arg)*)));
}




struct Stderr;

impl Write for Stderr {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

lazy_static! {
    static ref STDERR: Mutex<Stderr> = Mutex::new(Stderr {});
}

/// Use ANSICON to format colorized string
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
pub fn print(args: fmt::Arguments) {
    STDERR.lock().write_fmt(args).unwrap();
}


/// Use colorize! to print with color
pub fn print_colorized(args: fmt::Arguments, foreground_color: u8, background_color: u8) {
    STDERR.lock().write_fmt(colorize!(args, foreground_color, background_color)).unwrap();
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



#[allow(dead_code)]
pub enum ANSICON {
    Reset = 0,
    Bold = 1,
    Underline = 4,
    Blink = 5,
    Reverse = 7,
    FgBlack = 30,
    FgRed = 31,
    FgGreen = 32,
    FgYellow = 33,
    FgBlue = 34,
    FgMagenta = 35,
    FgCyan = 36,
    FgWhite = 37,
    FgDefault = 39,
    FgLightGray = 90,
    FgLightRed = 91,
    FgLightGreen = 92,
    FgLightYellow = 93,
    FgLightBlue = 94,
    FgLightMagenta = 95,
    FgLightCyan = 96,
    FgLightWhite = 97,
    BgBlack = 40,
    BgRed = 41,
    BgGreen = 42,
    BgYellow = 43,
    BgBlue = 44,
    BgMagenta = 45,
    BgCyan = 46,
    BgWhite = 47,
    BgDefault = 49,
    BgLightGray = 100,
    BgLightRed = 101,
    BgLightGreen = 102,
    BgLightYellow = 103,
    BgLightBlue = 104,
    BgLightMagenta = 105,
    BgLightCyan = 106,
    BgLightWhite = 107,
}
