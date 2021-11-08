use crate::config::Config;
use crate::log::Log;
use crate::tw_db_pp::{LuaValue, TableData, TotalWarDbPreProcessed};
use crate::util;
use crate::wh2_lua_error::Wh2LuaError;

use walkdir::WalkDir;

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rpfm_error;

use rpfm_lib;
use rpfm_lib::packedfile::table::db::DB;
use rpfm_lib::packedfile::table::DecodedData;
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packfile::packedfile::{PackedFile, RawPackedFile};
use rpfm_lib::packfile::{PFHFileType, PFHVersion, PackFile};
use rpfm_lib::schema;
use rpfm_lib::schema::Schema;

pub struct Rpfm;

impl Rpfm {
    pub fn load_schema(game_name: &str) -> Result<Schema, Wh2LuaError> {
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
        Log::debug(&format!("Loading schema for {}", game_name));
        Ok(Schema::load(&rpfm_lib::SUPPORTED_GAMES[game_name].schema)?)
    }

    pub fn load(
        config: &Config,
    ) -> Result<BTreeMap<String, Vec<TotalWarDbPreProcessed>>, Wh2LuaError> {
        Log::debug("Loading files with RPFM...");

        let result = if config.packfiles.is_some() && config.packfiles.as_ref().unwrap().len() > 0 {
            Self::process_packfiles(config, config.packfiles.as_ref().unwrap())?
        } else if let Some(ref in_dir) = config.in_dir {
            Self::process_in_dir(config, in_dir)?
        } else {
            return Err(Wh2LuaError::ConfigError(format!("Neither packfile nor input directory parameters found in config and/or command arguments.")));
        };

        Ok(result)
    }

    fn process_packfiles(
        config: &Config,
        packfiles: &[PathBuf],
    ) -> Result<BTreeMap<String, Vec<TotalWarDbPreProcessed>>, Wh2LuaError> {
        Log::debug("Processing packfiles...");
        let mut result: BTreeMap<String, Vec<TotalWarDbPreProcessed>> = BTreeMap::new();

        for packfile_path in packfiles {
            Log::info(&format!(
                "Processing packfile: {}",
                packfile_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            ));
            let packfile = PackFile::open_packfiles(&[packfile_path.clone()], true, false, false)?;

            let mut pf_processed_result: Vec<TotalWarDbPreProcessed> = Vec::new();

            let mut packed_db_files = packfile.get_packed_files_by_type(PackedFileType::DB, true);

            if packed_db_files.len() == 0 {
                Log::rpfm("No db files found");
            } else {
                #[cfg(not(debug_assertions))]
                Log::set_single_line_log(true);

                for pf in packed_db_files.iter_mut() {
                    Log::rpfm(&format!(
                        "Processing db files for {} - {}",
                        packfile_path
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        pf.get_path().join("/")
                    ));
                    let pf_file_name = pf.get_path().last().unwrap().clone();
                    let decode_result =
                        Self::decode_db_packed_file(pf.get_ref_mut_raw(), &config.schema);

                    if decode_result.is_err() {
                        Log::set_single_line_log(false);

                        Log::warning(&format!(
                            "Could not process db table {} - {}. The table will be skipped.",
                            packfile_path
                                .file_name()
                                .unwrap()
                                .to_string_lossy()
                                .to_string(),
                            pf.get_path().join("/")
                        ));
                        Log::warning(&format!("Problem was: {}", decode_result.err().unwrap()));
                        Log::warning(&format!("(If the mod this table belongs to actually works and doesn't crash the game, the table likely does nothing and this is unlikely to cause any problems)"));

                        #[cfg(not(debug_assertions))]
                        Log::set_single_line_log(true);

                        continue;
                    }

                    let db = decode_result.unwrap();

                    let script_file_path = Self::create_script_file_path(
                        config,
                        db.get_ref_table_name(),
                        &pf_file_name,
                        Some(packfile_path),
                    )?;

                    pf_processed_result.push(Self::convert_rpfm_db_to_preprocessed_db(
                        &db,
                        db.get_ref_table_name(),
                        script_file_path,
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

                Log::rpfm(&format!(
                    "Processing db files for {} - DONE",
                    packfile_path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                ));

                Log::set_single_line_log(false);
            }
        }

        Ok(result)
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

    fn create_script_file_path(
        config: &Config,
        db_table: &str,
        db_file_name: &str,
        packfile_path: Option<&PathBuf>,
    ) -> Result<Vec<String>, Wh2LuaError> {
        let mut file_name_without_extension = db_file_name.to_string();
        let mut table_folder = "mod".to_string();

        if file_name_without_extension == "data__" {
            if config.base_mod {
                table_folder = "core".to_string();
            } else {
                table_folder = "mod_core".to_string();
                if let Some(core_prefix) = &config.mod_core_prefix {
                    file_name_without_extension =
                        format!("{}_{}", core_prefix, &file_name_without_extension);
                } else if let Some(file_path) = packfile_path {
                    file_name_without_extension = format!(
                        "{}_{}",
                        file_path.file_stem().unwrap().to_str().unwrap(),
                        &file_name_without_extension
                    );
                } else {
                    return Err(Wh2LuaError::ConfigError(format!("A (core) data__ file was found in the input files, but the --base flag is not set,\n  and no --core-prefix or --packfile was specified.\n  No sensible output filename could be determined.")));
                }
            }
        }

        let mut output_file_path = Vec::new();
        output_file_path.push("lua_db".to_string());
        output_file_path.push(table_folder);
        output_file_path.push(db_table.to_string());
        output_file_path.push(format!("{}.lua", file_name_without_extension));

        Ok(output_file_path)
    }

    fn process_in_dir(
        config: &Config,
        in_dir: &PathBuf,
    ) -> Result<BTreeMap<String, Vec<TotalWarDbPreProcessed>>, Wh2LuaError> {
        Log::debug(&format!(
            "Processing extracted db files in input directory: {}",
            in_dir.display()
        ));
        if !in_dir.exists() {
            return Err(Wh2LuaError::ConfigError(format!(
                "Input directory not found at specified path: {}",
                in_dir.display()
            )));
        }

        let in_dir_name = in_dir.file_name().unwrap().to_string_lossy().to_string();

        let rpfm_in_dir: PathBuf = [in_dir.as_path(), Path::new("db")].iter().collect();

        let mut dir_result: Vec<TotalWarDbPreProcessed> = Vec::new();

        #[cfg(not(debug_assertions))]
        Log::set_single_line_log(true);

        for entry in WalkDir::new(rpfm_in_dir.as_path()).min_depth(2) {
            let entry = entry.unwrap();
            if entry.path().extension().is_none() {
                let relative_path = util::strip_db_prefix_from_path(&entry.path());
                let db_table = util::get_parent_folder_name(&relative_path)?;

                let db_file_name = entry.path().file_stem().unwrap().to_str().unwrap();

                let script_file_path =
                    Self::create_script_file_path(config, db_table, db_file_name, None)?;

                Log::rpfm(&format!("Processing file: {}", relative_path.display()));

                dir_result.push(Self::pre_process_db_file(
                    &config,
                    &entry.path(),
                    script_file_path,
                )?);
            }
        }

        Log::rpfm("Processing files - DONE");

        Log::set_single_line_log(false);

        let mut result: BTreeMap<String, Vec<TotalWarDbPreProcessed>> = BTreeMap::new();

        result.insert(in_dir_name, dir_result);

        Ok(result)
    }

    pub fn pre_process_db_file(
        config: &Config,
        rpfm_db_file: &Path,
        script_file_path: Vec<String>,
    ) -> Result<TotalWarDbPreProcessed, Wh2LuaError> {
        Log::debug(&format!(
            "Pre-processing db file {} to output {}",
            rpfm_db_file.display(),
            script_file_path.join("/")
        ));
        let mut data = vec![];

        {
            let mut file = BufReader::new(fs::File::open(rpfm_db_file)?);
            file.read_to_end(&mut data)?;
        }

        let table_name = util::get_parent_folder_name(rpfm_db_file)?;

        let db_result = DB::read(&data, table_name, &config.schema, false);
        if db_result.is_err() {
            let error = db_result.err().unwrap();
            if error.kind() == &rpfm_error::ErrorKind::TableEmptyWithNoDefinition
                || error.kind() == &rpfm_error::ErrorKind::SchemaDefinitionNotFound
            {
                Log::set_single_line_log(false);
                Log::warning(&format!(
                    "RPFM could not load table {} due to a missing definition in the schema, returning empty table",
                    table_name
                ));
                #[cfg(not(debug_assertions))]
                Log::set_single_line_log(true);

                Ok(TotalWarDbPreProcessed::new(
                    table_name,
                    TableData::FlatArray(vec![vec![]]),
                    script_file_path,
                ))
            } else {
                Err(error.into())
            }
        } else {
            let db = db_result.unwrap();
            Ok(Self::convert_rpfm_db_to_preprocessed_db(
                &db,
                db.get_ref_table_name(),
                script_file_path,
            )?)
        }
    }

    fn convert_rpfm_db_to_preprocessed_db(
        rpfm_db: &DB,
        table_name: &str,
        script_file_path: Vec<String>,
    ) -> Result<TotalWarDbPreProcessed, Wh2LuaError> {
        Log::debug(&format!(
            "Converting {} to pre-processed table...",
            table_name
        ));
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

        Ok(TotalWarDbPreProcessed::new(
            table_name,
            data,
            script_file_path,
        ))
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

    pub fn generate_packfile_with_script(
        scripts_to_pack: HashMap<Vec<String>, (String, String)>,
    ) -> Result<PackFile, Wh2LuaError> {
        let mut packfile = PackFile::new_with_name("lua_db_generated.pack", PFHVersion::PFH5);
        packfile.set_pfh_file_type(PFHFileType::Movie);
        for (path, value) in scripts_to_pack.iter() {
            let mut script_path = vec!["script".to_string()];
            path.iter().for_each(|e| script_path.push(e.clone()));
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64;

            let raw_packed_file = RawPackedFile::read_from_vec(
                script_path,
                "lua_db_generated.pack".to_string(),
                timestamp,
                false,
                value.1.as_bytes().to_vec(),
            );
            packfile.add_packed_file(&PackedFile::new_from_raw(&raw_packed_file), true)?;
        }

        Ok(packfile)
    }
}
