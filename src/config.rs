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
    pub write_files_to_disk: bool,
    pub launch_game: bool,
}

impl Config {
    pub fn from_matches(matches: &ArgMatches) -> Result<Config, Wh2LuaError> {
        Log::info("Parsing config...");

        let packfile_paths = Self::try_load_packfile_paths(matches)?;

        // Packfiles get priority. Only look at in_dir if we're not working with packfile(s)
        let in_dir_path = if packfile_paths.is_none() {
            Self::try_parse_in_dir_arg(matches)?
        } else {
            None
        };

        let out_dir_path = Self::calculate_out_dir(matches, &packfile_paths, &in_dir_path)?;

        let script_check = matches.value_of("script-check").map(str::to_string);

        let mod_core_prefix = matches.value_of("core-prefix").map(str::to_string);

        let base_mod = matches.is_present("base-data");

        let force = matches.is_present("force");

        let write_files_to_disk = matches.is_present("unpacked");

        let launch_game = Self::calculate_should_launch_game(matches);

        let schema = Rpfm::load_schema()?;

        Log::info("Config OK");

        Ok(Config {
            schema,
            packfiles: packfile_paths,
            in_dir: in_dir_path,
            out_dir: out_dir_path,
            script_check,
            mod_core_prefix,
            base_mod,
            force,
            write_files_to_disk,
            launch_game,
        })
    }

    fn try_load_packfile_paths(matches: &ArgMatches) -> Result<Option<Vec<PathBuf>>, Wh2LuaError> {
        Log::debug("Trying to load packfile paths...");
        let packfile_path_arg = matches.value_of("packfile");
        let in_dir_path_arg = matches.value_of("input-directory");

        let packfile_paths = if packfile_path_arg.is_none() && in_dir_path_arg.is_none() {
            Self::try_load_packfile_names_from_kmm_last_used_profile()?
        } else if let Some(packfile) = packfile_path_arg {
            Self::try_parse_single_packfile_path_from_arg(packfile)?.map(|packfile| vec![packfile])
        } else {
            None
        };

        Ok(packfile_paths)
    }

    fn try_load_packfile_names_from_kmm_last_used_profile(
    ) -> Result<Option<Vec<PathBuf>>, Wh2LuaError> {
        Log::debug("Looking for packfile paths in KMM last used mods profile...");
        let mut packfiles: Vec<PathBuf> = Vec::new();

        let kmm_last_used_file: PathBuf = match ProjectDirs::from("", "", "Kaedrin Mod Manager") {
            Some(dirs) => [
                dirs.config_dir().parent().unwrap(),
                &Path::new("Profiles"),
                &Path::new("Warhammer2"),
                &Path::new("profile_LastUsedMods.txt"),
            ]
            .iter()
            .collect(),
            None => {
                return Err(Wh2LuaError::ConfigError(
                    "No packfile or input dir specified, and KMM profiles dir cannot be found"
                        .to_string(),
                ))
            }
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

        return if packfiles.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(packfiles))
        };
    }

    fn try_parse_single_packfile_path_from_arg(
        packfile_path_str: &str,
    ) -> Result<Option<PathBuf>, Wh2LuaError> {
        Log::debug("Parsing packfile from arguments...");
        let packfile_path = PathBuf::from(packfile_path_str);
        if !packfile_path.exists() {
            return Err(Wh2LuaError::ConfigError(format!(
                "Packfile with specified path not found: {}",
                packfile_path.display()
            )));
        }
        Ok(Some(packfile_path))
    }

    fn try_parse_in_dir_arg(matches: &ArgMatches) -> Result<Option<PathBuf>, Wh2LuaError> {
        Log::debug("Trying to parse input directory from arguments...");
        if let Some(directory) = matches.value_of("input-directory") {
            let in_dir_path = PathBuf::from(directory);
            if !in_dir_path.exists() {
                return Err(Wh2LuaError::ConfigError(format!(
                    "Input directory with specified path not found: {}",
                    in_dir_path.display()
                )));
            }
            Ok(Some(in_dir_path))
        } else {
            Ok(None)
        }
    }

    fn calculate_out_dir(
        matches: &ArgMatches,
        packfile_paths: &Option<Vec<PathBuf>>,
        in_dir_path: &Option<PathBuf>,
    ) -> Result<PathBuf, Wh2LuaError> {
        Log::debug("Calculating output directory...");
        if let Some(output_dir) = matches.value_of("output-directory") {
            Ok(PathBuf::from(output_dir))
        } else {
            // If there is only a single packfile specified, use its name as the output directory
            if packfile_paths.is_some() && packfile_paths.as_ref().unwrap().len() == 1 {
                let packfile = packfile_paths.as_ref().unwrap().get(0).unwrap();
                Ok(Self::generate_output_directory_from_packfile(packfile)?)
            } else {
                // If an input directory is specified without output directory, use the input diretory as output directory
                if let Some(ref in_dir) = in_dir_path {
                    Log::debug(&format!("Outpt directory not specified in config/arguments, using same as input directory): {}", in_dir.to_str().unwrap()));
                    Ok(in_dir.clone())
                } else {
                    // Fallback: use ./lua_db_export directory
                    Log::debug(&format!(
                        "Output directory not specified, using .\\lua_db_export"
                    ));
                    let mut path = std::env::current_dir()?;
                    path.push("lua_db_export");
                    Ok(path)
                }
            }
        }
    }

    fn generate_output_directory_from_packfile(
        packfile_path: &PathBuf,
    ) -> Result<PathBuf, Wh2LuaError> {
        let packfile_dir = packfile_path.parent().unwrap();
        let packfile_name = packfile_path.file_stem().unwrap();
        let mut dir = PathBuf::from(packfile_dir);
        dir.push(&format!("{0}_lua_ext", packfile_name.to_str().unwrap()));
        Log::debug(&format!(
            "Output directory (derived from packfile name): {}",
            dir.to_str().unwrap()
        ));
        Ok(dir)
    }

    fn calculate_should_launch_game(matches: &ArgMatches) -> bool {
        let packfile_path_arg = matches.value_of("packfile");
        let in_dir_path_arg = matches.value_of("input-directory");

        packfile_path_arg.is_none()
            && in_dir_path_arg.is_none()
            && Path::new("./Warhammer2_real.exe").exists()
    }
}
