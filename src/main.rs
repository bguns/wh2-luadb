use crate::config::Config;
use crate::log::Log;
use crate::wh2_lua_error::Wh2LuaError;

use clap::{load_yaml, App};

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

    let config = Config::from_matches(&matches);

    prepare_output_dir(&config)?;

    Log::info("Running RPFM Command");

    if let Some(_) = config.packfile {
        rpfm_packfile(&config)?;
    } else {
        unimplemented!("not yet implemented")
    };

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
