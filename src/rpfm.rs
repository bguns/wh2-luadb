use crate::config::Config;
use crate::log::Log;
use crate::tw_db_pp::{LuaValue, TableData, TotalWarDbPreProcessed};
use crate::util;
use crate::wh2_lua_error::Wh2LuaError;

use walkdir::WalkDir;

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use rpfm_lib;
use rpfm_lib::packedfile::table::db::DB;
use rpfm_lib::packedfile::table::DecodedData;
use rpfm_lib::packfile::{PackFile, PathType};
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

    pub fn load(config: &Config) -> Result<Vec<TotalWarDbPreProcessed>, Wh2LuaError> {
        Log::debug("Loading files with RPFM...");
        // If a packfile is specified, we extract the packfile. The input directory for futher steps is the same as the output directory
        let rpfm_in_dir: PathBuf = if let Some(ref packfile) = config.packfile {
            Log::rpfm(&format!(
                "Extracting db folder from packfile: {}",
                packfile.display()
            ));
            // Run rpfm extract
            Self::load_packfile(&config)?;
            [config.out_dir.as_path(), Path::new("db")].iter().collect()
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
                [in_dir.as_path(), Path::new("db")].iter().collect()
            } else {
                return Err(Wh2LuaError::ConfigError(format!("Neither packfile nor input directory parameters found in config and/or command arguments.")));
            }
        };

        let mut result: Vec<TotalWarDbPreProcessed> = Vec::new();

        #[cfg(not(debug_assertions))]
        Log::set_single_line_log(true);

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

                result.push(Self::pre_process_db_file(
                    &config,
                    &entry.path(),
                    &output_file_path,
                )?);
            }
        }

        Log::set_single_line_log(false);

        Ok(result)
    }

    fn load_packfile(config: &Config) -> Result<(), Wh2LuaError> {
        let mut packfile = PackFile::open_packfiles(
            &[config.packfile.as_ref().unwrap().clone()],
            true,
            false,
            false,
        )?;
        let paths = vec![PathType::Folder(vec!["db".to_string()])];

        packfile.extract_packed_files_by_type(&paths, &config.out_dir)?;

        Ok(())
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
