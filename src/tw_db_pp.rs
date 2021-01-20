use std::collections::BTreeMap;

use rpfm_lib::packedfile::table::DecodedData;
use rpfm_lib::schema::Field;

use std::path::{Path, PathBuf};

pub enum TableData {
    KeyValue(BTreeMap<String, Vec<(Field, DecodedData)>>),
    FlatArray(Vec<Vec<(Field, DecodedData)>>),
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
