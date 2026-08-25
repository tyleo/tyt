use crate::{DirPrefs, OptionalDirPrefs};
use std::path::Path;

/// Preferences loaded from every location of a config file.
#[derive(Clone, Debug)]
pub struct Prefs<T> {
    /// Layer from the user home directory.
    pub user: OptionalDirPrefs<T>,

    /// Layer from the git root, or `None` outside a repository. When parsed,
    /// its prefs also appear as the first `hierarchy` entry.
    pub git_root: Option<OptionalDirPrefs<T>>,

    /// Layers from the git root down to cwd that supplied prefs, furthest
    /// from cwd first.
    pub hierarchy: Vec<DirPrefs<T>>,
}

impl<T> Prefs<T> {
    /// Returns `(dir, prefs)` layers in application order: user first, then
    /// the hierarchy from the git root down to cwd.
    pub fn application_order(&self) -> impl Iterator<Item = (&Path, &T)> {
        let user = self
            .user
            .prefs
            .as_ref()
            .map(|prefs| (self.user.dir.as_path(), prefs));

        let hierarchy = self
            .hierarchy
            .iter()
            .map(|layer| (layer.dir.as_path(), &layer.prefs));

        user.into_iter().chain(hierarchy)
    }
}
