use crate::config::Config;
use crate::log::Log;
use crate::tw_db_pp::{TableData, TotalWarDbPreProcessed};
use crate::util;
use crate::wh2_lua_error::Wh2LuaError;

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;

use rpfm_lib::packedfile::table::DecodedData;
use rpfm_lib::schema::Field;

pub struct LuaWriter {}

impl LuaWriter {
    pub fn write_tw_db_to_lua_file(
        config: &Config,
        table_data: &TotalWarDbPreProcessed,
    ) -> Result<(), Wh2LuaError> {
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
            &util::strip_db_prefix_from_path(&table_data.output_file_path).display()
        ));

        match &table_data.data {
            TableData::KeyValue(kv_table_data) => {
                result.push_str(&Self::lua_key_value_table(&kv_table_data, indent)?);
            }
            TableData::FlatArray(arr_table_data) => {
                result.push_str(&Self::lua_array_table(&arr_table_data, indent)?);
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
                result.push_str(&Self::decoded_data_to_lua_entry(&field.get_name(), &data)?);
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
                result.push_str(&Self::decoded_data_to_lua_entry(field.get_name(), &data)?);
            }
            result.push_str("},\n");
        }
        Ok(result)
    }

    fn decoded_data_to_lua_entry(
        field_name: &str,
        data: &DecodedData,
    ) -> Result<String, Wh2LuaError> {
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
}
