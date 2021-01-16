use colored::Colorize;

use std::fmt;
use std::path::PathBuf;

use rpfm_error;

#[derive(Debug)]
pub enum Wh2LuaError {
    ConfigError(String),
    RpfmError(rpfm_error::Error),
    OutDirNotEmpty(PathBuf),
    IoError(std::io::Error),
}

impl From<std::io::Error> for Wh2LuaError {
    fn from(err: std::io::Error) -> Self {
        Wh2LuaError::IoError(err)
    }
}

impl From<rpfm_error::Error> for Wh2LuaError {
    fn from(err: rpfm_error::Error) -> Self {
        Wh2LuaError::RpfmError(err)
    }
}

impl fmt::Display for Wh2LuaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", "[ERROR]".red())?;
        match &self {
            &Wh2LuaError::ConfigError(message) => {
                write!(f, "{}", message)
            }
            &Wh2LuaError::RpfmError(err) => {
                write!(f, "{}", err.to_terminal())
            }
            &Wh2LuaError::OutDirNotEmpty(path) => {
                write!(f, "Output directory not empty: {}", path.display())
            }
            &Wh2LuaError::IoError(io_error) => {
                write!(f, "Unexpected IO error: {}", io_error)
            }
        }
    }
}
