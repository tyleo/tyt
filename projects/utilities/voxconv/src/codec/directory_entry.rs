use std::path::PathBuf;

/// One entry of a listed directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectoryEntry {
    /// The entry's path, the listed directory joined with its name.
    pub path: PathBuf,

    /// Whether the entry is itself a directory.
    pub is_dir: bool,
}
