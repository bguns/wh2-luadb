use std::path::{Path, PathBuf};

use crate::wh2_lua_error::Wh2LuaError;

// strip everything before "<db>/<table>/<db_file>"
pub fn strip_db_prefix_from_path(path: &Path) -> PathBuf {
    let prefix_path = path.parent().and_then(Path::parent).and_then(Path::parent);

    let relative_path = if let Some(prefix) = prefix_path {
        path.strip_prefix(prefix).unwrap()
    } else {
        path.clone()
    };

    PathBuf::from(relative_path)
}

pub fn get_parent_folder_name(path: &Path) -> Result<&str, Wh2LuaError> {
    path.parent()
        .and_then(Path::file_name)
        .and_then(|file_name_os_string| file_name_os_string.to_str())
        .ok_or_else(|| {
            Wh2LuaError::ConfigError(format!(
                "Unable to get parent folder name for path: {}",
                path.display()
            ))
        })
}
