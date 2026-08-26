use crate::{
    Dependencies, DeserializePrefs, Prefs, load_git_prefs, load_hierarchy_prefs, load_user_prefs,
};
use std::io::Result as IOResult;

/// Loads preferences for `key` from every `file_name` location.
pub fn load_prefs<T>(
    dependencies: &impl Dependencies,
    codec: &impl DeserializePrefs<T>,
    file_name: &str,
    key: &str,
) -> IOResult<Prefs<T>> {
    let user = load_user_prefs(dependencies, codec, file_name, key)?;

    let git_root = load_git_prefs(dependencies, codec, file_name, key)?;

    let hierarchy = load_hierarchy_prefs(dependencies, codec, file_name, key)?;

    Ok(Prefs {
        user,
        git_root,
        hierarchy,
    })
}
