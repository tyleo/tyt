//! The side effects the commands perform, injected so they run over any
//! filesystem and output. [`DependenciesImpl`] binds std's filesystem and
//! standard output.

#[allow(clippy::module_inception)]
mod dependencies;
mod dependencies_impl;
mod terminal_columns;
mod write_stdout;

pub use dependencies::*;
pub use dependencies_impl::*;
pub use terminal_columns::*;
pub use write_stdout::*;

// Re-exported so a caller can name every trait `Dependencies` requires.
pub use voxconv::{DirectoryEntry, ListDir, ReadFile, WriteFile};
