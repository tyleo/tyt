/// Whether a pattern matches directories only or files and directories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitIgnoreRegexKind {
    /// Matches only directories. A trailing `/` in the pattern sets this.
    Directory,

    /// Matches files or directories.
    FileOrDirectory,
}
