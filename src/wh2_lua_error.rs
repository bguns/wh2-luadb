use colored::Colorize;

use std::fmt;
use std::path::PathBuf;
use std::process::ExitStatus;

#[derive(Debug)]
pub enum Wh2LuaError {
    OutDirNotEmpty(PathBuf),
    IoError(std::io::Error),
    UnexpectedExitStatus(ExitStatus),
}

impl From<std::io::Error> for Wh2LuaError {
    fn from(err: std::io::Error) -> Self {
        Wh2LuaError::IoError(err)
    }
}

impl fmt::Display for Wh2LuaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", "[ERROR]".red())?;
        match &self {
            &Wh2LuaError::OutDirNotEmpty(path) => {
                write!(f, "Output directory not empty: {}", path.to_str().unwrap())
            }
            &Wh2LuaError::IoError(io_error) => {
                write!(f, "Unexpected IO error: {}", io_error)
            }
            Wh2LuaError::UnexpectedExitStatus(exit_status) => match exit_status.code() {
                Some(code) => write!(f, "RPFM exited with an unexptected exit status ({}).", code),
                None => write!(f, "RPFM process terminated by signal"),
            },
        }
    }
}
