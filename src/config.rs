use clap::ArgMatches;
use std::path::PathBuf;

use crate::log::Log;
use crate::Wh2LuaError;

pub struct Config {
    pub rpfm_path: PathBuf,
    pub packfile: Option<PathBuf>,
    pub in_dir: Option<PathBuf>,
    pub out_dir: PathBuf,
}

impl Config {
    pub fn from_matches(matches: &ArgMatches) -> Result<Config, Wh2LuaError> {
        let rpfm_path: PathBuf = if let Some(rpfm) = matches.value_of("rpfm-path") {
            PathBuf::from(rpfm)
        } else {
            return Err(Wh2LuaError::ConfigError(
                "RPFM path not provided in config or command line arguments.".to_string(),
            ));
        };

        if !rpfm_path.exists() {
            return Err(Wh2LuaError::RpfmPathError(rpfm_path.clone()));
        }

        let packfile_path = if let Some(packfile) = matches.value_of("packfile") {
            Some(PathBuf::from(packfile))
        } else {
            None
        };

        let in_dir_path = if let Some(directory) = matches.value_of("directory") {
            Some(PathBuf::from(directory))
        } else {
            None
        };

        let out_dir_path = if let Some(output_dir) = matches.value_of("output-dir") {
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

        Ok(Config {
            rpfm_path,
            packfile: packfile_path,
            in_dir: in_dir_path,
            out_dir: out_dir_path,
        })
    }
}
