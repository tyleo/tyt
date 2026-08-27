use crate::{
    Dependencies, DeserializePrefs, DirPrefs, Prefs, PrefsPaths, load_hierarchy_prefs,
    load_prefs_from_dir,
};
use std::io::Result as IOResult;

/// Loads preferences for `key` from every `file_name` location in `paths`.
pub fn load_prefs<T>(
    dependencies: &impl Dependencies,
    codec: &impl DeserializePrefs<T>,
    paths: &PrefsPaths,
    file_name: &str,
    key: &str,
) -> IOResult<Prefs<T>> {
    let user = match &paths.user {
        Some(dir) => {
            load_prefs_from_dir(dependencies, codec, dir, file_name, key)?.map(|prefs| DirPrefs {
                dir: dir.clone(),
                prefs,
            })
        }
        None => None,
    };

    let (git_root, hierarchy) = match &paths.git_root {
        Some(root) => (
            load_prefs_from_dir(dependencies, codec, root, file_name, key)?.map(|prefs| DirPrefs {
                dir: root.clone(),
                prefs,
            }),
            load_hierarchy_prefs(dependencies, codec, root, &paths.cwd, file_name, key)?,
        ),
        None => (None, Vec::new()),
    };

    Ok(Prefs {
        user,
        git_root,
        hierarchy,
    })
}
