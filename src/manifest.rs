use std::path::Path;

use crate::Wh2LuaError;

use csv::ReaderBuilder;

use serde::{Deserialize, Serialize};

/// This struct represents the entire **Manifest.txt** from the /data folder.
///
/// Private for now, because I see no public use for this.
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest(pub Vec<ManifestEntry>);

/// This struct represents a Manifest Entry.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// The path of the file, relative to /data.
    pub relative_path: String,

    /// The size in bytes of the file.
    pub size: u64,

    /// If the file comes with the base game (1), or with one of its dlc (0). Not in all games.
    pub belongs_to_base_game: Option<u8>,
}

// Implementation of `Manifest`.
impl Manifest {
    /// This function returns a parsed version of the `manifest.txt` in the folder you provided, if exists and is parseable.
    pub fn _read_from_folder(path: &Path) -> Result<Self, Wh2LuaError> {
        let manifest_path = path.join("manifest.txt");

        let mut reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .quoting(false)
            .has_headers(false)
            .flexible(true)
            .from_path(&manifest_path)?;

        // Due to "flexible" not actually working when doing serde-backed deserialization (took some time to figure this out)
        // the deserialization has to be done manually.
        let mut entries = vec![];
        for record in reader.records() {
            let record = record?;

            // We only know these manifest formats.
            if record.len() != 2 && record.len() != 3 {
                return Err(Wh2LuaError::RpfmError(rpfm_error::Error::from(
                    rpfm_error::ErrorKind::ManifestError,
                )));
            } else {
                let mut manifest_entry = ManifestEntry::default();
                manifest_entry.relative_path = record
                    .get(0)
                    .ok_or_else(|| rpfm_error::Error::from(rpfm_error::ErrorKind::ManifestError))?
                    .to_owned();
                manifest_entry.size = record
                    .get(1)
                    .ok_or_else(|| rpfm_error::Error::from(rpfm_error::ErrorKind::ManifestError))?
                    .parse()?;

                // In newer games, a third field has been added.
                if record.len() == 3 {
                    manifest_entry.belongs_to_base_game = record
                        .get(2)
                        .ok_or_else(|| {
                            rpfm_error::Error::from(rpfm_error::ErrorKind::ManifestError)
                        })?
                        .parse()
                        .ok();
                } else {
                    manifest_entry.belongs_to_base_game = None;
                }

                entries.push(manifest_entry);
            }
        }

        let manifest = Self(entries);
        Ok(manifest)
    }
}
