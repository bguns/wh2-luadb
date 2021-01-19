use crate::config::Config;
use crate::log::Log;
use crate::wh2_lua_error::Wh2LuaError;

use clap::{load_yaml, App};
use colored::Colorize;
use walkdir::WalkDir;

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};

use rpfm_lib;
use rpfm_lib::packedfile::table::db::DB;
use rpfm_lib::packedfile::table::DecodedData;
use rpfm_lib::packfile::{PackFile, PathType};
use rpfm_lib::schema::Field;

mod config;
mod log;
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
    let preprocessed_tables = run_rpfm(&config)?;

    Log::info("Writing data to Lua scripts...");

    #[cfg(not(debug_assertions))]
    Log::set_single_line_log(true);

    for table in preprocessed_tables {
        rpfm_db_to_lua(&config, &table)?;
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

// strip everything before "<db>/<table>/<db_file>"
fn strip_db_prefix_from_path(path: &Path) -> PathBuf {
    let prefix_path = path.parent().and_then(Path::parent).and_then(Path::parent);

    let relative_path = if let Some(prefix) = prefix_path {
        path.strip_prefix(prefix).unwrap()
    } else {
        path.clone()
    };

    PathBuf::from(relative_path)
}

fn run_rpfm(config: &Config) -> Result<Vec<TotalWarDbPreProcessed>, Wh2LuaError> {
    // If a packfile is specified, we extract the packfile. The input directory for futher steps is the same as the output directory
    let rpfm_in_dir = if let Some(ref packfile) = config.packfile {
        Log::rpfm(&format!(
            "Extracting db folder from packfile: {}",
            packfile.display()
        ));
        // Run rpfm extract
        rpfm_packfile(&config)?;
        config.out_dir.clone()
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
            in_dir.clone()
        } else {
            return Err(Wh2LuaError::ConfigError(format!("Neither packfile nor input directory parameters found in config and/or command arguments.")));
        }
    };

    let mut result: Vec<TotalWarDbPreProcessed> = Vec::new();

    #[cfg(not(debug_assertions))]
    Log::set_single_line_log(true);

    for entry in WalkDir::new(rpfm_in_dir.as_path()).min_depth(3) {
        let entry = entry.unwrap();
        if entry.path().extension().is_none() {
            let relative_path = strip_db_prefix_from_path(&entry.path());

            let mut output_file_path = config.out_dir.clone();
            output_file_path.push("lua_db");
            output_file_path.push(relative_path.strip_prefix("db").unwrap());
            output_file_path = output_file_path.with_extension("lua");
            fs::create_dir_all(&output_file_path.parent().unwrap())?;

            Log::rpfm(&format!("Loading file: {}", relative_path.display()));

            Log::debug(&format!("Input file: {}", entry.path().display()));

            result.push(TotalWarDbPreProcessed::load_from_file(
                &config,
                &entry.path(),
                &output_file_path,
            )?);
        }
    }

    Log::set_single_line_log(false);

    Ok(result)
}

fn rpfm_packfile(config: &Config) -> Result<(), Wh2LuaError> {
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

enum TableData {
    KeyValue(BTreeMap<String, Vec<(Field, DecodedData)>>),
    FlatArray(Vec<Vec<(Field, DecodedData)>>),
}

struct TotalWarDbPreProcessed {
    pub data: TableData,
    pub output_file_path: PathBuf,
}

impl TotalWarDbPreProcessed {
    pub fn load_from_file(
        config: &Config,
        rpfm_db_file: &Path,
        output_file_path: &Path,
    ) -> Result<Self, Wh2LuaError> {
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

        Ok(Self::from_rpfm_db(&config, &db, &output_file_path)?)
    }

    pub fn from_rpfm_db(
        _config: &Config,
        rpfm_db: &DB,
        output_file_path: &Path,
    ) -> Result<Self, Wh2LuaError> {
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

            let mut processed_data: BTreeMap<String, Vec<(Field, DecodedData)>> = BTreeMap::new();

            for row in rpfm_data {
                let key_string = row[key_field_index].data_to_string();
                processed_data.insert(key_string.clone(), Vec::new());
                for (field, data) in rpfm_fields.iter().zip(row.iter()) {
                    processed_data
                        .get_mut(&key_string)
                        .unwrap()
                        .push((field.clone(), data.clone()));
                }
            }

            TableData::KeyValue(processed_data)
        } else {
            let mut processed_data: Vec<Vec<(Field, DecodedData)>> = Vec::new();
            for row in rpfm_data {
                let mut processed_row: Vec<(Field, DecodedData)> = Vec::new();
                for (field, data) in rpfm_fields.iter().zip(row.iter()) {
                    processed_row.push((field.clone(), data.clone()));
                }
                processed_data.push(processed_row);
            }
            TableData::FlatArray(processed_data)
        };

        Ok(TotalWarDbPreProcessed {
            data,
            output_file_path: PathBuf::from(&output_file_path),
        })
    }
}

fn rpfm_db_to_lua(config: &Config, table_data: &TotalWarDbPreProcessed) -> Result<(), Wh2LuaError> {
    let mut result = String::new();
    let mut indent: usize = 0;

    if let Some(script_check) = &config.script_check {
        result.push_str("local result = {}\n\n");
        result.push_str(&format!("if vfs.exists(\"{}\") {{\n", script_check));
        indent += 1;
        result.push_str(&format!("{}result = {{\n", "  ".repeat(indent)));
    } else {
        result.push_str(&format!("{}local result = {{\n", "  ".repeat(indent)));
    }

    indent += 1;

    Log::info(&format!(
        "Creating script: {}",
        &strip_db_prefix_from_path(&table_data.output_file_path).display()
    ));

    match &table_data.data {
        TableData::KeyValue(kv_table_data) => {
            result.push_str(&lua_key_value_table(&kv_table_data, indent)?);
        }
        TableData::FlatArray(arr_table_data) => {
            result.push_str(&lua_array_table(&arr_table_data, indent)?);
        }
    }

    while indent > 1 {
        indent -= 1;
        result.push_str(&format!("{}}}\n", "  ".repeat(indent)));
    }

    result.push_str("}\n\n");
    result.push_str("return result");

    let mut out_file = fs::File::create(&table_data.output_file_path)?;
    out_file.write_all(result.as_bytes())?;

    Ok(())
}

fn lua_key_value_table(
    kv_table_data: &BTreeMap<String, Vec<(Field, DecodedData)>>,
    indent: usize,
) -> Result<String, Wh2LuaError> {
    let mut result = String::new();

    for (key, value) in kv_table_data.iter() {
        result.push_str(&format!("{}[\"{}\"] = {{ ", "  ".repeat(indent), key));
        for (field, data) in value.iter() {
            result.push_str(&decoded_data_to_lua_entry(&field.get_name(), &data)?);
        }
        result.push_str("},\n");
    }

    Ok(result)
}

fn lua_array_table(
    arr_table_data: &[Vec<(Field, DecodedData)>],
    indent: usize,
) -> Result<String, Wh2LuaError> {
    let mut result = String::new();
    for row in arr_table_data {
        result.push_str(&format!("{}{{ ", "  ".repeat(indent)));
        for (field, data) in row {
            result.push_str(&decoded_data_to_lua_entry(field.get_name(), &data)?);
        }
        result.push_str("},\n");
    }
    Ok(result)
}

fn decoded_data_to_lua_entry(field_name: &str, data: &DecodedData) -> Result<String, Wh2LuaError> {
    match data {
        DecodedData::Boolean(value) => Ok(format!("[\"{}\"] = {}, ", field_name, value)),
        DecodedData::F32(value) => Ok(format!("[\"{}\"] = {}, ", field_name, value)),
        DecodedData::I16(value) => Ok(format!("[\"{}\"] = {}, ", field_name, value)),
        DecodedData::I32(value) => Ok(format!("[\"{}\"] = {}, ", field_name, value)),
        DecodedData::I64(value) => Ok(format!("[\"{}\"] = {}, ", field_name, value)),
        DecodedData::StringU8(value) => Ok(format!("[\"{}\"] = \"{}\", ", field_name, value)),
        DecodedData::StringU16(value) => Ok(format!("[\"{}\"] = \"{}\", ", field_name, value)),
        DecodedData::OptionalStringU8(value) => {
            Ok(format!("[\"{}\"] = \"{}\", ", field_name, value))
        }
        DecodedData::OptionalStringU16(value) => {
            Ok(format!("[\"{}\"] = \"{}\", ", field_name, value))
        }
        DecodedData::SequenceU16(_) | DecodedData::SequenceU32(_) => {
            return Err(Wh2LuaError::LuaError(
                "Cannot convert recursive (sequence) fields to Lua".to_string(),
            ))
        }
    }
}
