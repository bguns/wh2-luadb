use crate::config::Config;
use crate::log::Log;
use crate::wh2_lua_error::Wh2LuaError;

use clap::{load_yaml, App};
use colored::Colorize;
use walkdir::WalkDir;

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

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
        Log::rpfm(&format!(
            "Extracting db folder from packfile: {}",
            packfile.display()
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
                    in_dir.display()
                )));
            }
            in_dir.clone()
        } else {
            return Err(Wh2LuaError::ConfigError(format!("Neither packfile nor input directory parameters found in config and/or command arguments.")));
        }
    };

    for entry in WalkDir::new(rpfm_in_dir.as_path()).min_depth(3) {
        let entry = entry.unwrap();
        if entry.path().extension().is_none() {
            rpfm_to_tsv(&config, &entry.path())?;
        }
    }

    Ok(())
}

fn rpfm_packfile(config: &Config) -> Result<(), Wh2LuaError> {
    Command::new(&config.rpfm_path)
        .args(&[
            "-g",
            "warhammer_2",
            "-p",
            &format!("{0}", &config.packfile.as_ref().unwrap().display()),
            "packfile",
            "-E",
            &format!("{0}", &config.out_dir.display()),
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

fn rpfm_to_tsv(config: &Config, db_file_path: &Path) -> Result<(), Wh2LuaError> {
    let prefix_path = db_file_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent);

    let relative_path = if let Some(prefix) = prefix_path {
        db_file_path.strip_prefix(prefix).unwrap()
    } else {
        db_file_path.clone()
    };

    let mut output_file_path = config.out_dir.clone();
    output_file_path.push(relative_path);
    output_file_path = output_file_path.with_extension("tsv");
    fs::create_dir_all(&output_file_path.parent().unwrap())?;

    Log::info(&format!("Processing file: {}", relative_path.display()));

    let rpfm_resulting_tsv_file_name = db_file_path.with_extension("tsv");
    Command::new(&config.rpfm_path)
        .args(&[
            "-g",
            "warhammer_2",
            "table",
            "-e",
            &format!("{}", db_file_path.display()),
        ])
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .map_err(|e| Wh2LuaError::IoError(e))
        .and_then(|exit_status| {
            if exit_status.success() {
                std::fs::rename(rpfm_resulting_tsv_file_name, output_file_path)?;
                Ok(())
            } else {
                Err(Wh2LuaError::UnexpectedExitStatus(exit_status))
            }
        })
}
