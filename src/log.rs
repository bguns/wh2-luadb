use crate::Wh2LuaError;
use colored::Colorize;

pub struct Log {}

impl Log {
    pub fn info(info_text: &str) {
        eprintln!("{} {}", "[INFO]".blue(), info_text);
    }

    pub fn rpfm(message: &str) {
        eprintln!("{} {}", "  [RPFM]".yellow(), message);
    }

    pub fn _error(error_text: &str) {
        eprintln!("{} {}", "[ERROR]".red(), error_text);
    }

    pub fn error(error: &Wh2LuaError) {
        eprintln!("{}", error);
    }

    pub fn _warning(warning_text: &str) {
        eprintln!("{} {}", "[WARNING]".yellow(), warning_text);
    }
}
