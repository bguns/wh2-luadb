use crate::config::Config;
use crate::log::Log;
use crate::lua_writer::LuaWriter;
use crate::rpfm::Rpfm;
use crate::wh2_lua_error::Wh2LuaError;

use clap::{load_yaml, App};

use crossterm::event::read;
use crossterm::style::Colorize;

use std::fs;
use std::io::Write;
use std::process::Command;

mod config;
mod log;
mod lua_writer;
mod manifest;
mod rpfm;
mod tw_db_pp;
mod util;
mod wh2_lua_error;

fn main() {
    let res = do_the_things();

    match res {
        // On error, log the error and wait for keypress to quit
        Err(ref error) => {
            Log::error(&error);
            match read().unwrap() {
                _ => {}
            }
        }

        // On success, only wait for keystroke in debug mode, and launch game if needed
        Ok(ref config) => {
            Log::print_overwritten_files();
            Log::info("all gucci!");
            #[cfg(debug_assertions)]
            match read().unwrap() {
                _ => {}
            }

            Log::debug(&format!("Config: launch_game = {}", config.launch_game));
            if config.launch_game {
                Command::new("./Warhammer2_real.exe").output().unwrap();
            }
        }
    }
}

/// Runs the application code, breaking off and returning a Wh2LuaError as soon as an unrecoverable error is encoutered.
/// Returns the Config struct on success for the app to use in end/cleanup step.
fn do_the_things() -> Result<Config, Wh2LuaError> {
    // Load the CLAP configuration. This happens at compile time
    let yaml = load_yaml!("cli.yaml");

    let matches = App::from(yaml).get_matches();

    Log::info("Loading config...");
    let config = Config::from_matches(&matches)?;
    prepare_output_dir(&config)?;
    Log::info(&format!("Config {}", "OK".green()));

    Log::info("Loading files with RPFM...");
    let preprocessed_packfiles = Rpfm::load(&config)?;

    Log::info("Writing data to Lua scripts...");
    let mut packfile_names: Vec<_> = preprocessed_packfiles.keys().cloned().collect();
    packfile_names.reverse();

    for packfile_name in packfile_names {
        Log::info(&format!("Generating Lua data for {}", packfile_name));

        #[cfg(not(debug_assertions))]
        Log::set_single_line_log(true);

        for table in preprocessed_packfiles.get(&packfile_name).unwrap() {
            let lua_script = LuaWriter::convert_tw_db_to_lua_script(&config, &table)?;
            let out_path = table.output_file_path(&config);
            let mut file = fs::File::open(out_path)?;
            file.write(lua_script.as_bytes())?;
        }

        Log::set_single_line_log(false);
    }

    Ok(config)
}

/// Creates the output directory if it doesn ot exists. Returns an error if the output dir is not empty (and the force flag is not set)
fn prepare_output_dir(config: &Config) -> Result<(), Wh2LuaError> {
    fs::create_dir_all(&config.out_dir)?;
    // Directory is empty if its iterator has no elements
    if !&config.force && !&config.out_dir.read_dir()?.next().is_none() {
        return Err(Wh2LuaError::OutDirNotEmpty(config.out_dir.clone()));
    }
    Ok(())
}
