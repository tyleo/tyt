use crate::{Dependencies, DeserializePrefs, DirPrefs, load_hierarchy_prefs, load_user_prefs};
use std::io::Result as IOResult;

/// Loads preference layers for `key` in application order: user first, then
/// the hierarchy from the git root down to cwd.
pub fn load_application_prefs<T: DeserializePrefs>(
    dependencies: &impl Dependencies,
    file_name: &str,
    key: &str,
) -> IOResult<Vec<DirPrefs<T>>> {
    let user = load_user_prefs(dependencies, file_name, key)?;

    let hierarchy = load_hierarchy_prefs(dependencies, file_name, key)?;

    let mut layers = Vec::new();

    if let Some(prefs) = user.prefs {
        layers.push(DirPrefs {
            dir: user.dir,
            prefs,
        });
    }

    layers.extend(hierarchy);

    Ok(layers)
}
