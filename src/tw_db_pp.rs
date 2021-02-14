use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::config::Config;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LuaValue {
    Number(String),
    Text(String),
    Boolean(bool),
}

impl LuaValue {
    pub fn to_lua_value(&self) -> String {
        match &self {
            &LuaValue::Boolean(value) => format!("{}", value),
            &LuaValue::Number(value) => format!("{}", value),
            &LuaValue::Text(value) => format!("\"{}\"", value),
        }
    }
}

pub enum TableData {
    KeyValue(BTreeMap<LuaValue, Vec<(LuaValue, LuaValue)>>),
    FlatArray(Vec<Vec<(LuaValue, LuaValue)>>),
}

pub struct TotalWarDbPreProcessed {
    pub table_name: String,
    pub script_file_path: Vec<String>,
    pub data: TableData,
}

impl TotalWarDbPreProcessed {
    pub fn new(table_name: &str, data: TableData, script_file_path: Vec<String>) -> Self {
        Self {
            table_name: table_name.to_string(),
            script_file_path,
            data,
        }
    }

    pub fn output_file_path(&self, config: &Config) -> PathBuf {
        let mut output_file_path = config.out_dir.clone();
        self.script_file_path
            .iter()
            .for_each(|e| output_file_path.push(e));
        output_file_path
    }
}
