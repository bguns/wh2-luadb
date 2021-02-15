use crate::config::Config;
use crate::log::Log;
use crate::lua_writer::LuaWriter;
use crate::rpfm::Rpfm;
use crate::wh2_lua_error::Wh2LuaError;

use clap::{load_yaml, App};

use crossterm::event::read;

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
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
                Log::info("Starting Total War: Warhammer II...");
                if Path::new("./Warhammer2_real.exe").exists() {
                    Command::new("./Warhammer2_real.exe").output().unwrap();
                } else {
                    Command::new("./Warhammer2.exe").output().unwrap();
                }
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

    let config = Config::from_matches(&matches)?;
    prepare_output_dir(&config)?;

    let preprocessed_packfiles = Rpfm::load(&config)?;

    let mut packfile_names: Vec<_> = preprocessed_packfiles.keys().cloned().collect();
    packfile_names.reverse();

    if config.write_files_to_disk {
        for packfile_name in packfile_names {
            #[cfg(not(debug_assertions))]
            Log::set_single_line_log(true);

            for table in preprocessed_packfiles.get(&packfile_name).unwrap() {
                let file_name = table.script_file_path.last().unwrap().clone();
                Log::info(&format!(
                    "Generating Lua script for {} - {}/{}",
                    packfile_name,
                    &table.table_name,
                    // Drop .lua suffix
                    &file_name[..file_name.len() - 4]
                ));
                let lua_script = LuaWriter::convert_tw_db_to_lua_script(&config, &table)?;
                let out_path = table.output_file_path(&config);

                fs::create_dir_all(&out_path.parent().unwrap())?;

                if out_path.exists() {
                    Log::add_overwritten_file(format!("{}", out_path.display()));
                }

                Log::debug(&format!("Writing file: {}", out_path.display()));
                let mut file = fs::File::create(out_path)?;
                file.write(lua_script.as_bytes())?;
            }

            Log::info(&format!(
                "Generating Lua script for {} - DONE",
                packfile_name
            ));
            Log::set_single_line_log(false);
        }
    } else {
        // target_packfile_path -> (source_packfile_name, lua_script)
        let mut scripts_to_pack: HashMap<Vec<String>, (String, String)> = HashMap::new();

        for packfile_name in packfile_names {
            #[cfg(not(debug_assertions))]
            Log::set_single_line_log(true);

            for table in preprocessed_packfiles.get(&packfile_name).unwrap() {
                let file_name = table.script_file_path.last().unwrap().clone();
                Log::info(&format!(
                    "Generating Lua script for {} - {}/{}",
                    packfile_name,
                    &table.table_name,
                    // Drop .lua suffix
                    &file_name[..file_name.len() - 4]
                ));
                let lua_script = LuaWriter::convert_tw_db_to_lua_script(&config, &table)?;
                if let Some(overwritten) = scripts_to_pack.insert(
                    table.script_file_path.clone(),
                    (packfile_name.clone(), lua_script),
                ) {
                    Log::set_single_line_log(false);
                    Log::warning(&format!(
                        "Packfile {} overwrites script {} from packfile {}",
                        packfile_name,
                        &table.script_file_path.join("/"),
                        overwritten.0
                    ));
                    #[cfg(not(debug_assertions))]
                    Log::set_single_line_log(true);
                }
            }

            Log::info(&format!(
                "Generating Lua script for {} - DONE",
                packfile_name
            ));
            Log::set_single_line_log(false);
        }

        let mut packfile = Rpfm::generate_packfile_with_script(scripts_to_pack)?;

        let out_packfile_path: PathBuf = if config.launch_game {
            ["data", "lua_db_generated.pack"].iter().collect()
        } else {
            let mut out_path = config.out_dir.clone();
            out_path.push("lua_db_generated.pack");
            out_path
        };

        packfile.save(Some(out_packfile_path))?;
    }

    Ok(config)
}

/// Creates the output directory if it doesn ot exists. Returns an error if the output dir is not empty (and the force flag is not set)
fn prepare_output_dir(config: &Config) -> Result<(), Wh2LuaError> {
    fs::create_dir_all(&config.out_dir)?;
    // Directory is empty if its iterator has no elements
    if config.write_files_to_disk && !config.force && !&config.out_dir.read_dir()?.next().is_none()
    {
        return Err(Wh2LuaError::OutDirNotEmpty(config.out_dir.clone()));
    }
    Ok(())
}
