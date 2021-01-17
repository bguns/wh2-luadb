use crate::Wh2LuaError;
use colored::Colorize;
use crossterm::cursor::MoveToColumn;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType, DisableLineWrap, EnableLineWrap};

use std::io::stderr;

static mut SINGLE_LINE: bool = false;

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
            if SINGLE_LINE {
                let mut stderr = stderr();
                execute!(stderr, MoveToColumn(1), Clear(ClearType::CurrentLine)).unwrap();
                eprint!("{}", message);
            } else {
                eprintln!("{}", message);
            }
        }
    }

    pub fn set_single_line_log(single_line: bool) {
        unsafe {
            let mut stderr = stderr();
            if SINGLE_LINE && !single_line {
                eprintln!();
            }
            SINGLE_LINE = single_line;
            if SINGLE_LINE {
                execute!(stderr, DisableLineWrap).unwrap();
            } else {
                execute!(stderr, EnableLineWrap).unwrap();
            }
        }
    }
}
