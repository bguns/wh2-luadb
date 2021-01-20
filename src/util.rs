use std::path::{Path, PathBuf};

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
