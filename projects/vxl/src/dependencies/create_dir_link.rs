use std::{io::Result as IOResult, path::Path};

/// Creates symlinks to directories.
pub trait CreateDirLink {
    /// Creates a symlink at `link` to the directory `target`. A relative
    /// `target` resolves from `link`'s directory. Creates `link`'s missing
    /// parents.
    fn create_dir_link(&self, target: &Path, link: &Path) -> IOResult<()>;
}
