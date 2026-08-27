use crate::{Dependencies, DeserializePrefs, DirPrefs, load_prefs_from_dir};
use std::{io::Result as IOResult, path::Path};

/// Loads the `key` prefs from each `(dir, file_name)` source in order. Sources
/// that supply no prefs are omitted.
pub fn load_sources_prefs<T>(
    dependencies: &impl Dependencies,
    codec: &impl DeserializePrefs<T>,
    sources: &[(&Path, &str)],
    key: &str,
) -> IOResult<Vec<DirPrefs<T>>> {
    let mut layers = Vec::new();

    for &(dir, file_name) in sources {
        let Some(prefs) = load_prefs_from_dir(dependencies, codec, dir, file_name, key)? else {
            continue;
        };

        layers.push(DirPrefs {
            dir: dir.to_path_buf(),
            prefs,
        });
    }

    Ok(layers)
}
