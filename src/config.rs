use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use clap::ArgMatches;
use directories::ProjectDirs;

use crate::log::Log;
use crate::rpfm::Rpfm;
use crate::Wh2LuaError;

use rpfm_lib::schema::Schema;

pub struct Config {
    pub schema: Schema,
    pub packfiles: Option<Vec<PathBuf>>,
    pub in_dir: Option<PathBuf>,
    pub out_dir: PathBuf,
    pub script_check: Option<String>,
    pub mod_core_prefix: Option<String>,
    pub base_mod: bool,
    pub force: bool,
}

impl Config {
    pub fn from_matches(matches: &ArgMatches) -> Result<Config, Wh2LuaError> {
        let packfile_path_arg = matches.value_of("packfile");
        let in_dir_path_arg = matches.value_of("input-directory");

        let packfile_paths = if packfile_path_arg.is_none() && in_dir_path_arg.is_none() {
            let mut packfiles: Vec<PathBuf> = Vec::new();

            Log::info("No packfile and no in dir specified, looking for KMM last used profile...");
            let kmm_last_used_file: PathBuf =
                match ProjectDirs::from("", "", "Kaedrin Mod Manager") {
                    Some(dirs) => [
                        dirs.config_dir().parent().unwrap(),
                        &Path::new("Profiles"),
                        &Path::new("Warhammer2"),
                        &Path::new("profile_LastUsedMods.txt"),
                    ]
                    .iter()
                    .collect(),
                    None => return Err(Wh2LuaError::ConfigError(
                        "No packfile or input dir specified, and KMM profiles dir cannot be found"
                            .to_string(),
                    )),
                };

            if !kmm_last_used_file.exists() {
                return Err(Wh2LuaError::ConfigError(
                    "No packfile or input dir specified, and KMM profiles dir cannot be found"
                        .to_string(),
                ));
            }

            let mut packfiles_names: Vec<String>;
            {
                let file = fs::File::open(kmm_last_used_file)?;
                let reader = BufReader::new(&file);
                packfiles_names = reader.lines().collect::<Result<_, _>>()?;
            }
            packfiles_names.reverse();

            for name in packfiles_names {
                Log::debug(&format!("Packfile in KMM last profile: {}", name));
                packfiles.push([Path::new("data"), Path::new(&name)].iter().collect());
            }

            Some(packfiles)
        } else if let Some(packfile) = packfile_path_arg {
            let packfile_path = PathBuf::from(packfile);
            if !packfile_path.exists() {
                return Err(Wh2LuaError::ConfigError(format!(
                    "Packfile with specified path not found: {}",
                    packfile_path.display()
                )));
            }
            Some(vec![packfile_path])
        } else {
            None
        };

        let in_dir_path = if packfile_paths.is_none() {
            if let Some(directory) = in_dir_path_arg {
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
            }
        } else {
            None
        };

        let out_dir_path = if let Some(output_dir) = matches.value_of("output-directory") {
            PathBuf::from(output_dir)
        } else {
            if packfile_paths.is_some() && packfile_paths.as_ref().unwrap().len() == 1 {
                let packfile = packfile_paths.as_ref().unwrap().get(0).unwrap();
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
                    Log::info(&format!("Outpt directory not specified in config/arguments, using same as input directory): {}", in_dir.to_str().unwrap()));
                    in_dir.clone()
                } else {
                    Log::info(&format!(
                        "Output directory not specified, using .\\lua_db_export"
                    ));
                    let mut path = std::env::current_dir()?;
                    path.push("lua_db_export");
                    path
                }
            }
        };

        let script_check = matches.value_of("script-check").map(str::to_string);

        let mod_core_prefix = matches.value_of("core-prefix").map(str::to_string);

        let base_mod = matches.is_present("base-data");

        let force = matches.is_present("force");

        let schema = Rpfm::load_schema()?;

        Ok(Config {
            schema,
            packfiles: packfile_paths,
            in_dir: in_dir_path,
            out_dir: out_dir_path,
            script_check,
            mod_core_prefix,
            base_mod,
            force,
        })
    }
}
