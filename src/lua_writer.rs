use crate::config::Config;
use crate::log::Log;
use crate::tw_db_pp::{LuaValue, TableData, TotalWarDbPreProcessed};
use crate::util;
use crate::wh2_lua_error::Wh2LuaError;

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;

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
        kv_table_data: &BTreeMap<LuaValue, Vec<(LuaValue, LuaValue)>>,
        indent: usize,
    ) -> Result<String, Wh2LuaError> {
        let mut result = String::new();

        for (key, values) in kv_table_data.iter() {
            result.push_str(&format!(
                "{}[{}] = {{ ",
                "  ".repeat(indent),
                key.to_lua_value()
            ));
            for (k, v) in values.iter() {
                result.push_str(&Self::lua_key_value_entry(k, v));
            }
            result.push_str("},\n");
        }

        Ok(result)
    }

    fn lua_array_table(
        arr_table_data: &[Vec<(LuaValue, LuaValue)>],
        indent: usize,
    ) -> Result<String, Wh2LuaError> {
        let mut result = String::new();
        for row in arr_table_data {
            result.push_str(&format!("{}{{ ", "  ".repeat(indent)));
            for (k, v) in row {
                result.push_str(&Self::lua_key_value_entry(k, v));
            }
            result.push_str("},\n");
        }
        Ok(result)
    }

    fn lua_key_value_entry(key: &LuaValue, value: &LuaValue) -> String {
        format!("[{}] = {}, ", key.to_lua_value(), value.to_lua_value())
    }
}
