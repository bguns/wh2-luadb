use crate::config::Config;
use crate::log::Log;
use crate::lua_writer::LuaWriter;
use crate::rpfm::Rpfm;
use crate::wh2_lua_error::Wh2LuaError;

use clap::{load_yaml, App};
use colored::Colorize;

use std::fs;

mod config;
mod log;
mod lua_writer;
mod rpfm;
mod tw_db_pp;
mod util;
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

    Log::info("Loading files with RPFM...");
    let preprocessed_tables = Rpfm::load(&config)?;

    Log::info("Writing data to Lua scripts...");

    #[cfg(not(debug_assertions))]
    Log::set_single_line_log(true);

    for table in preprocessed_tables {
        LuaWriter::write_tw_db_to_lua_file(&config, &table)?;
    }

    Log::set_single_line_log(false);

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
