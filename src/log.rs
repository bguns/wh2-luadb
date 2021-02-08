use crate::Wh2LuaError;
use crossterm::cursor::MoveToColumn;
use crossterm::execute;
use crossterm::style::Colorize;
use crossterm::terminal::{Clear, ClearType, DisableLineWrap, EnableLineWrap};

use std::io::stderr;

static mut SINGLE_LINE: bool = false;
static mut FILES_OVERWRITTEN: Vec<String> = Vec::new();

/// Provides static functions to log things to console (through stderr)
pub struct Log {}

impl Log {
    pub fn info(info_text: &str) {
        Self::print_log(&format!("{} {}", "[INFO]".blue(), info_text));
    }

    /// Logs "\[DEBUG\] text", but only on --debug builds
    #[allow(unused_variables)]
    pub fn debug(text: &str) {
        #[cfg(debug_assertions)]
        Self::print_log(&format!("{} {}", "[DEBUG]".yellow(), text));
    }

    pub fn rpfm(message: &str) {
        Self::print_log(&format!("{} {}", "[RPFM]".magenta(), message));
    }

    pub fn error(error: &Wh2LuaError) {
        Self::print_log(&format!("{}", error));
    }

    pub fn warning(warning_text: &str) {
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

    /// Sets the flag that controls if log messages should be print on the same line (true) (clearing the previous message), or if each message should appear on a newline (false).
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

    /// Add a String representation of a file path to the (static) list of overwritten files
    pub fn add_overwritten_file(file_path_str: String) {
        unsafe {
            FILES_OVERWRITTEN.push(file_path_str);
        }
    }

    /// Logs all files in the static list of overwritten files
    pub fn print_overwritten_files() {
        unsafe {
            if !&FILES_OVERWRITTEN.is_empty() {
                Self::warning("files overwritten: ");
                for file_path in &FILES_OVERWRITTEN {
                    eprintln!("{}", file_path);
                }
            }
        }
    }
}
