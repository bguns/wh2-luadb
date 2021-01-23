use std::collections::BTreeMap;

use std::path::{Path, PathBuf};

pub enum LuaValue {
    Number(String),
    Text(String),
    Boolean(bool),
}

pub enum TableData {
    KeyValue(BTreeMap<String, Vec<(String, LuaValue)>>),
    FlatArray(Vec<Vec<(String, LuaValue)>>),
}

pub struct TotalWarDbPreProcessed {
    pub data: TableData,
    pub output_file_path: PathBuf,
}

impl TotalWarDbPreProcessed {
    pub fn new(data: TableData, output_file_path: &Path) -> Self {
        Self {
            data,
            output_file_path: PathBuf::from(output_file_path),
        }
    }
}
