use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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
    pub output_file_path: PathBuf,
    pub data: TableData,
    pub indexed_fields: Vec<LuaValue>,
    pub built_indexes: Vec<TableIndex>,
}

impl TotalWarDbPreProcessed {
    pub fn new(table_name: &str, data: TableData, output_file_path: &Path) -> Self {
        Self {
            table_name: table_name.to_string(),
            output_file_path: PathBuf::from(output_file_path),
            data,
            indexed_fields: Vec::new(),
            built_indexes: Vec::new(),
        }
    }
}

pub struct TableIndex {
    pub field: LuaValue,
    pub data: BTreeMap<LuaValue, Vec<LuaValue>>,
}
