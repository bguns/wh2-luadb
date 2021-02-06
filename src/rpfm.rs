use crate::config::Config;
use crate::log::Log;
use crate::tw_db_pp::{LuaValue, TableData, TotalWarDbPreProcessed};
use crate::util;
use crate::wh2_lua_error::Wh2LuaError;

use directories::ProjectDirs;

use walkdir::WalkDir;

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use rpfm_error;

use rpfm_lib;
use rpfm_lib::packedfile::table::db::DB;
use rpfm_lib::packedfile::table::DecodedData;
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packfile::packedfile::RawPackedFile;
use rpfm_lib::packfile::PackFile;
use rpfm_lib::schema;
use rpfm_lib::schema::Schema;

pub struct Rpfm;

impl Rpfm {
    pub fn load_schema() -> Result<Schema, Wh2LuaError> {
        Log::rpfm("Checking for schema update...");
        match Schema::check_update() {
            Ok(schema::APIResponseSchema::NoLocalFiles) => {
                Log::rpfm("No schema files found locally. Downloading...");
                Schema::update_schema_repo()?;
                Log::rpfm("Schema downloaded!")
            }
            Ok(schema::APIResponseSchema::NewUpdate) => {
                Log::rpfm("Updated schema found. Downloading update...");
                Schema::update_schema_repo()?;
                Log::rpfm("Schema updated!");
            }
            Ok(schema::APIResponseSchema::NoUpdate) => {
                Log::rpfm("Schema up to date");
            }
            Err(e) => {
                return Err(Wh2LuaError::RpfmError(e));
            }
        }

        Ok(Schema::load(
            &rpfm_lib::SUPPORTED_GAMES["warhammer_2"].schema,
        )?)
    }

    fn decode_db_packed_file(
        raw_packed_file: &mut RawPackedFile,
        schema: &Schema,
    ) -> Result<DB, Wh2LuaError> {
        match PackedFileType::get_packed_file_type(raw_packed_file.get_path()) {
            PackedFileType::DB => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let name = raw_packed_file.get_path().get(1).ok_or_else(|| {
                    Wh2LuaError::RpfmError(rpfm_error::Error::from(
                        rpfm_error::ErrorKind::DBTableIsNotADBTable,
                    ))
                })?;
                let packed_file = DB::read(&data, &name, &schema, false)?;
                Ok(packed_file)
            }
            _ => {
                return Err(Wh2LuaError::RpfmError(rpfm_error::Error::from(
                    rpfm_error::ErrorKind::DBTableIsNotADBTable,
                )))
            }
        }
    }

    pub fn load(
        config: &Config,
    ) -> Result<BTreeMap<String, Vec<TotalWarDbPreProcessed>>, Wh2LuaError> {
        Log::debug("Loading files with RPFM...");

        let mut result: BTreeMap<String, Vec<TotalWarDbPreProcessed>> = BTreeMap::new();
        let mut packfiles: Vec<PathBuf> = Vec::new();

        // if no packfile or in_dir is specified, we load KMM last_profile packfiles
        if config.packfile.is_none() && config.in_dir.is_none() {
            Log::debug("No packfile and no in dir specified, looking for KMM last used profile...");
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
        } else if let Some(ref packfile) = config.packfile {
            packfiles.push(packfile.clone());
        }

        if packfiles.len() > 0 {
            // Run rpfm load
            for packfile_path in packfiles {
                Log::info(&format!("Processing packfile: {}", packfile_path.display()));
                let packfile = Self::load_packfile(&packfile_path)?;

                let mut pf_processed_result: Vec<TotalWarDbPreProcessed> = Vec::new();

                let mut packed_db_files =
                    packfile.get_packed_files_by_type(PackedFileType::DB, true);

                for pf in packed_db_files.iter_mut() {
                    let pf_file_name = pf.get_path().last().unwrap().clone();
                    Log::rpfm(&format!(
                        "Getting DB from: {}, type is {} ",
                        pf_file_name,
                        PackedFileType::get_packed_file_type_by_data(pf)
                    ));
                    let db = Self::decode_db_packed_file(pf.get_ref_mut_raw(), &config.schema)?;

                    let mut file_name_without_extension = pf_file_name.to_string();
                    let mut table_folder = "mod".to_string();
                    if file_name_without_extension == "data__" {
                        if config.base_mod {
                            table_folder = "core".to_string();
                        } else {
                            table_folder = "mod_core".to_string();
                            if let Some(core_prefix) = &config.mod_core_prefix {
                                file_name_without_extension =
                                    format!("{}_{}", core_prefix, &file_name_without_extension);
                            } else {
                                file_name_without_extension = format!(
                                    "{}_{}",
                                    packfile_path.file_stem().unwrap().to_str().unwrap(),
                                    &file_name_without_extension
                                );
                            } /*else {
                                  return Err(Wh2LuaError::ConfigError(format!("A (core) data__ file was found in the input files, but the --base flag is not set,\n  and no --core-prefix or --packfile was specified.\n  No sensible output filename could be determined.")));
                              }*/
                        }
                    }

                    let mut output_file_path = config.out_dir.clone();
                    output_file_path.push("lua_db");
                    output_file_path.push(table_folder);
                    output_file_path.push(db.get_table_name());
                    output_file_path.push(file_name_without_extension);
                    output_file_path = output_file_path.with_extension("lua");
                    fs::create_dir_all(&output_file_path.parent().unwrap())?;

                    if output_file_path.exists() {
                        Log::add_overwritten_file(format!("{}", output_file_path.display()));
                    }

                    pf_processed_result.push(Self::convert_rpfm_db_to_preprocessed_db(
                        &db,
                        &output_file_path,
                    )?);
                }

                result.insert(
                    packfile_path
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                    pf_processed_result,
                );
            }
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
                let rpfm_in_dir: PathBuf = [in_dir.as_path(), Path::new("db")].iter().collect();

                let mut dir_result: Vec<TotalWarDbPreProcessed> = Vec::new();

                for entry in WalkDir::new(rpfm_in_dir.as_path()).min_depth(2) {
                    let entry = entry.unwrap();
                    if entry.path().extension().is_none() {
                        let relative_path = util::strip_db_prefix_from_path(&entry.path());

                        let mut file_name_without_extension = entry
                            .path()
                            .file_stem()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string();
                        let mut table_folder = "mod".to_string();

                        if file_name_without_extension == "data__" {
                            if config.base_mod {
                                table_folder = "core".to_string();
                            } else {
                                table_folder = "mod_core".to_string();
                                if let Some(core_prefix) = &config.mod_core_prefix {
                                    file_name_without_extension =
                                        format!("{}_{}", core_prefix, &file_name_without_extension);
                                } else if let Some(packfile) = &config.packfile {
                                    file_name_without_extension = format!(
                                        "{}_{}",
                                        packfile.file_stem().unwrap().to_str().unwrap(),
                                        &file_name_without_extension
                                    );
                                } else {
                                    return Err(Wh2LuaError::ConfigError(format!("A (core) data__ file was found in the input files, but the --base flag is not set,\n  and no --core-prefix or --packfile was specified.\n  No sensible output filename could be determined.")));
                                }
                            }
                        }

                        let mut output_file_path = config.out_dir.clone();
                        output_file_path.push("lua_db");
                        output_file_path.push(table_folder);
                        output_file_path.push(relative_path.parent().unwrap().file_name().unwrap());
                        output_file_path.push(file_name_without_extension);
                        output_file_path = output_file_path.with_extension("lua");
                        fs::create_dir_all(&output_file_path.parent().unwrap())?;

                        if output_file_path.exists() {
                            Log::add_overwritten_file(format!("{}", output_file_path.display()));
                        }

                        Log::rpfm(&format!("Loading file: {}", relative_path.display()));

                        Log::debug(&format!("Input file: {}", entry.path().display()));

                        dir_result.push(Self::pre_process_db_file(
                            &config,
                            &entry.path(),
                            &output_file_path,
                        )?);
                    }
                }

                result.insert("directory".to_string(), dir_result);
            } else {
                return Err(Wh2LuaError::ConfigError(format!("Neither packfile nor input directory parameters found in config and/or command arguments.")));
            }
        };

        /*#[cfg(not(debug_assertions))]
        Log::set_single_line_log(true);

        Log::set_single_line_log(false);*/

        Ok(result)
    }

    fn load_packfile(packfile_path: &PathBuf) -> Result<PackFile, Wh2LuaError> {
        let packfile = PackFile::open_packfiles(&[packfile_path.clone()], true, false, false)?;

        Ok(packfile)
    }

    pub fn pre_process_db_file(
        config: &Config,
        rpfm_db_file: &Path,
        output_file_path: &Path,
    ) -> Result<TotalWarDbPreProcessed, Wh2LuaError> {
        Log::debug(&format!(
            "Pre-processing db file {} to output {}",
            rpfm_db_file.display(),
            output_file_path.display()
        ));
        let mut data = vec![];

        {
            let mut file = BufReader::new(fs::File::open(rpfm_db_file)?);
            file.read_to_end(&mut data)?;
        }

        let table_name = rpfm_db_file
            .parent()
            .and_then(Path::file_name)
            .unwrap()
            .to_str()
            .unwrap();

        let db = DB::read(&data, table_name, &config.schema, false)?;

        Ok(Self::convert_rpfm_db_to_preprocessed_db(
            &db,
            &output_file_path,
        )?)
    }

    fn decoded_data_to_lua_value(data: &DecodedData) -> LuaValue {
        match data {
            DecodedData::Boolean(value) => LuaValue::Boolean(*value),
            DecodedData::F32(value) => LuaValue::Number(value.to_string()),
            DecodedData::I16(value) => LuaValue::Number(value.to_string()),
            DecodedData::I32(value) => LuaValue::Number(value.to_string()),
            DecodedData::I64(value) => LuaValue::Number(value.to_string()),
            DecodedData::StringU8(value)
            | DecodedData::StringU16(value)
            | DecodedData::OptionalStringU8(value)
            | DecodedData::OptionalStringU16(value) => LuaValue::Text(value.to_string()),
            DecodedData::SequenceU16(_) => LuaValue::Text("SequenceU16".to_string()),
            DecodedData::SequenceU32(_) => LuaValue::Text("SequenceU32".to_string()),
        }
    }

    fn convert_rpfm_db_to_preprocessed_db(
        rpfm_db: &DB,
        output_file_path: &Path,
    ) -> Result<TotalWarDbPreProcessed, Wh2LuaError> {
        let rpfm_fields = rpfm_db.get_ref_definition().get_fields_processed();
        let rpfm_data = rpfm_db.get_ref_table_data();
        let is_single_key = rpfm_fields
            .iter()
            .filter(|field| field.get_is_key())
            .count()
            == 1;

        let data = if is_single_key {
            let key_field_index = rpfm_fields
                .iter()
                .position(|field| field.get_is_key())
                .unwrap();

            let mut processed_data: BTreeMap<LuaValue, Vec<(LuaValue, LuaValue)>> = BTreeMap::new();

            for row in rpfm_data {
                let key_data = Self::decoded_data_to_lua_value(&row[key_field_index]);
                processed_data.insert(key_data.clone(), Vec::new());
                for (field, data) in rpfm_fields.iter().zip(row.iter()) {
                    processed_data.get_mut(&key_data).unwrap().push((
                        LuaValue::Text(field.get_name().to_string()),
                        Self::decoded_data_to_lua_value(data),
                    ));
                }
            }

            TableData::KeyValue(processed_data)
        } else {
            let mut processed_data: Vec<Vec<(LuaValue, LuaValue)>> = Vec::new();
            for row in rpfm_data {
                let mut processed_row: Vec<(LuaValue, LuaValue)> = Vec::new();
                for (field, data) in rpfm_fields.iter().zip(row.iter()) {
                    processed_row.push((
                        LuaValue::Text(field.get_name().to_string()),
                        Self::decoded_data_to_lua_value(data),
                    ));
                }
                processed_data.push(processed_row);
            }
            TableData::FlatArray(processed_data)
        };

        let table_name = output_file_path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        Ok(TotalWarDbPreProcessed::new(
            table_name,
            data,
            &output_file_path,
        ))
    }
}