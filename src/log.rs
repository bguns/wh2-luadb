use crate::Wh2LuaError;
use colored::Colorize;

pub struct Log {}

impl Log {
    pub fn info(info_text: &str) {
        println!("{} {}", "[INFO]".blue(), info_text);
    }

    pub fn _error(error_text: &str) {
        println!("{} {}", "[ERROR]".red(), error_text);
    }

    pub fn error(error: &Wh2LuaError) {
        println!("{}", error);
    }

    pub fn _warning(warning_text: &str) {
        println!("{} {}", "[WARNING]".yellow(), warning_text);
    }
}
