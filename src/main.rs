use crate::config::Config;
use crate::log::Log;
use crate::wh2_lua_error::Wh2LuaError;

use clap::{load_yaml, App};
use colored::Colorize;

use std::fs;
use std::process::Command;

mod config;
mod log;
mod wh2_lua_error;

fn main() {
    if let Err(error) = do_the_things() {
        Log::error(&error);
    } else {
        Log::info("all gucci!");
    }
}

fn do_the_things() -> Result<(), Wh2LuaError> {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    Log::info("Loading config...");
    let config = Config::from_matches(&matches)?;
    prepare_output_dir(&config)?;
    Log::info(&format!("Config {}", "OK".green()));

    Log::info("Processing files with RPFM...");
    run_rpfm(&config)?;

    Ok(())
}

fn prepare_output_dir(config: &Config) -> Result<(), Wh2LuaError> {
    fs::create_dir_all(&config.out_dir)?;
    // Directory is empty if its iterator has no elements
    if !&config.out_dir.read_dir()?.next().is_none() {
        return Err(Wh2LuaError::OutDirNotEmpty(config.out_dir.clone()));
    }
    Ok(())
}

fn run_rpfm(config: &Config) -> Result<(), Wh2LuaError> {
    // If a packfile is specified, we extract the packfile. The input directory for futher steps is the same as the output directory
    let rpfm_in_dir = if let Some(ref packfile) = config.packfile {
        if !packfile.exists() {
            return Err(Wh2LuaError::ConfigError(format!(
                "Packfile with specified path not found: {}",
                packfile.to_str().unwrap()
            )));
        }
        Log::rpfm(&format!(
            "Extracting db folder from packfile: {}",
            packfile.to_str().unwrap()
        ));
        // Run rpfm extract
        rpfm_packfile(&config)?;
        config.out_dir.clone()
    }
    // If an input directory is specified instead, we (obviously) use that for later steps
    else {
        if let Some(ref in_dir) = config.in_dir {
            if !in_dir.exists() {
                return Err(Wh2LuaError::ConfigError(format!(
                    "Input directory not found at specified path: {}",
                    in_dir.to_str().unwrap()
                )));
            }
            in_dir.clone()
        } else {
            return Err(Wh2LuaError::ConfigError(format!("Neither packfile nor input directory parameters found in config and/or command arguments.")));
        }
    };

    Ok(())
}

fn rpfm_packfile(config: &Config) -> Result<(), Wh2LuaError> {
    Command::new(&config.rpfm_path)
        .args(&[
            "-g",
            "warhammer_2",
            "-p",
            &format!("{0}", &config.packfile.as_ref().unwrap().to_str().unwrap()),
            "packfile",
            "-E",
            &format!("{0}", &config.out_dir.to_str().unwrap()),
            "-",
            "db",
        ])
        .status()
        .map_err(|e| Wh2LuaError::IoError(e))
        .and_then(|exit_status| {
            if exit_status.success() {
                Ok(())
            } else {
                Err(Wh2LuaError::UnexpectedExitStatus(exit_status))
            }
        })
}
