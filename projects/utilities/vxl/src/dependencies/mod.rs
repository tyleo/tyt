//! The side effects the commands perform, injected so they run over any
//! filesystem and output. [`DependenciesImpl`] binds std's filesystem and
//! standard output.

#[allow(clippy::module_inception)]
mod dependencies;
mod dependencies_impl;
mod resolve_prefs_paths;
mod terminal_columns;
mod write_stdout;

pub use dependencies::*;
pub use dependencies_impl::*;
pub use resolve_prefs_paths::*;
pub use terminal_columns::*;
pub use write_stdout::*;

// Re-exported so a caller can name every trait `Dependencies` requires.
// meshconv's same-named file traits are reached through `meshconv`.
pub use voxconv::{DirectoryEntry, ListDir, ReadFile, WriteFile};
