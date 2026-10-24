//! The side effects the commands perform, injected so they run over any
//! filesystem and output. [`DependenciesImpl`] binds std's filesystem and
//! standard output and error.

// Public API

mod create_dir_all;
mod create_dir_link;
mod create_temp_dir;
#[allow(clippy::module_inception)]
mod dependencies;
mod dependencies_impl;
mod display_image;
mod path_kind;
mod read_path_kind;
mod remove_dir;
mod rename_path;
mod resolve_prefs_paths;
mod run_program;
mod terminal_columns;
mod write_stderr;
mod write_stdout;

pub use create_dir_all::*;
pub use create_dir_link::*;
pub use create_temp_dir::*;
pub use dependencies::*;
pub use dependencies_impl::*;
pub use display_image::*;
pub use path_kind::*;
pub use read_path_kind::*;
pub use remove_dir::*;
pub use rename_path::*;
pub use resolve_prefs_paths::*;
pub use run_program::*;
pub use terminal_columns::*;
pub use write_stderr::*;
pub use write_stdout::*;

// Re-exported so a caller can name every trait `Dependencies` requires.
// meshconv's same-named file traits are reached through `meshconv`.
pub use voxconv::{DirectoryEntry, ListDir, ReadFile, WriteFile};
