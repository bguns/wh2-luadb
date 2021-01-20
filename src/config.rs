use clap::ArgMatches;
use std::path::PathBuf;

use crate::log::Log;
use crate::rpfm::Rpfm;
use crate::Wh2LuaError;

use rpfm_lib::schema::Schema;

pub struct Config {
    pub schema: Schema,
    pub packfile: Option<PathBuf>,
    pub in_dir: Option<PathBuf>,
    pub out_dir: PathBuf,
    pub script_check: Option<String>,
}

impl Config {
    pub fn from_matches(matches: &ArgMatches) -> Result<Config, Wh2LuaError> {
        let packfile_path = if let Some(packfile) = matches.value_of("packfile") {
            let packfile_path = PathBuf::from(packfile);
            if !packfile_path.exists() {
                return Err(Wh2LuaError::ConfigError(format!(
                    "Packfile with specified path not found: {}",
                    packfile_path.display()
                )));
            }
            Some(packfile_path)
        } else {
            None
        };

        let in_dir_path = if let Some(directory) = matches.value_of("input-directory") {
            let in_dir_path = PathBuf::from(directory);
            if !in_dir_path.exists() {
                return Err(Wh2LuaError::ConfigError(format!(
                    "Input directory with specified path not found: {}",
                    in_dir_path.display()
                )));
            }
            Some(in_dir_path)
        } else {
            None
        };

        let out_dir_path = if let Some(output_dir) = matches.value_of("output-directory") {
            PathBuf::from(output_dir)
        } else {
            if let Some(ref packfile) = packfile_path {
                let packfile_dir = packfile.parent().unwrap();
                let packfile_name = packfile.file_stem().unwrap();
                let mut dir = PathBuf::from(packfile_dir);
                dir.push(&format!("{0}_lua_ext", packfile_name.to_str().unwrap()));
                Log::info(&format!(
                    "Output directory (derived from packfile name): {}",
                    dir.to_str().unwrap()
                ));
                dir
            } else {
                if let Some(ref in_dir) = in_dir_path {
                    Log::info(&format!("Outpt directory (not specified in config/arguments, using same as input directory): {}", in_dir.to_str().unwrap()));
                    in_dir.clone()
                } else {
                    return Err(Wh2LuaError::ConfigError(format!(
                        "No packfile or input directory specified in config/arguments."
                    )));
                }
            }
        };

        let script_check = matches.value_of("script-check").map(str::to_string);

        let schema = Rpfm::load_schema()?;

        Ok(Config {
            schema,
            packfile: packfile_path,
            in_dir: in_dir_path,
            out_dir: out_dir_path,
            script_check,
        })
    }
}
