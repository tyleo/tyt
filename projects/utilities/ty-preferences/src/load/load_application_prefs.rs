use crate::{
    Dependencies, DeserializePrefs, DirPrefs, PrefsPaths, load_hierarchy_prefs, load_prefs_from_dir,
};
use std::io::Result as IOResult;

/// Loads preference layers for `key` in application order: user first, then the
/// hierarchy from the git root down to cwd.
pub fn load_application_prefs<T>(
    dependencies: &impl Dependencies,
    codec: &impl DeserializePrefs<T>,
    paths: &PrefsPaths,
    file_name: &str,
    key: &str,
) -> IOResult<Vec<DirPrefs<T>>> {
    let mut layers = Vec::new();

    if let Some(user_dir) = &paths.user
        && let Some(prefs) = load_prefs_from_dir(dependencies, codec, user_dir, file_name, key)?
    {
        layers.push(DirPrefs {
            dir: user_dir.clone(),
            prefs,
        });
    }

    if let Some(git_root) = &paths.git_root {
        layers.extend(load_hierarchy_prefs(
            dependencies,
            codec,
            git_root,
            &paths.cwd,
            file_name,
            key,
        )?);
    }

    Ok(layers)
}
