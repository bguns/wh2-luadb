use crate::Wh2LuaError;
use colored::Colorize;

static mut NEWLINE: bool = true;

pub struct Log {}

impl Log {
    pub fn info(info_text: &str) {
        Self::print_log(&format!("{} {}", "[INFO]".blue(), info_text));
    }

    #[allow(unused_variables)]
    pub fn debug(text: &str) {
        #[cfg(debug_assertions)]
        Self::print_log(&format!("{} {}", "[DEBUG]".yellow(), text));
    }

    pub fn rpfm(message: &str) {
        Self::print_log(&format!("{} {}", "  [RPFM]".magenta(), message));
    }

    pub fn _error(error_text: &str) {
        Self::print_log(&format!("{} {}", "[ERROR]".red(), error_text));
    }

    pub fn error(error: &Wh2LuaError) {
        Self::print_log(&format!("{}", error));
    }

    pub fn _warning(warning_text: &str) {
        Self::print_log(&format!("{} {}", "[WARNING]".yellow(), warning_text));
    }

    fn print_log(message: &str) {
        unsafe {
            if NEWLINE {
                eprintln!("{}", message);
            } else {
                eprint!("\r{}\r{}", " ".repeat(70), message);
            }
        }
    }

    pub fn set_single_line_log(single_line: bool) {
        unsafe {
            if !NEWLINE && !single_line {
                eprintln!();
            }
            NEWLINE = !single_line
        }
    }
}
