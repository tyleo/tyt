use std::path::PathBuf;

/// What sits at a path, read without following a final symlink.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathKind {
    /// Nothing.
    Missing,

    /// A regular file.
    File,

    /// A directory.
    Directory,

    /// A symlink.
    Symlink {
        /// The target, relative to the link's directory when not absolute.
        target: PathBuf,
    },
}
