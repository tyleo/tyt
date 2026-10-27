use std::path::PathBuf;

/// The state of `.claude`, which Claude Code reads in place of `.agents`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaudeLink {
    /// Nothing.
    Missing,

    /// A symlink to `.agents`.
    Linked,

    /// A symlink to anywhere else.
    LinkedElsewhere {
        /// The target as written.
        target: PathBuf,
    },

    /// A real directory.
    Directory,

    /// A regular file.
    File,
}
