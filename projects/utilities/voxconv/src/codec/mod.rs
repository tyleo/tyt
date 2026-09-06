//! Moves a document through its format's bridge, gated behind the `codec`
//! feature. The filesystem and the format codecs come from the caller.

// Public API

mod check_document_files;
mod dependencies;
mod directory_entry;
mod list_dir;
mod load;
mod read;
mod read_document_files;
mod read_file;
mod save;
mod write;
mod write_document_files;
mod write_file;

pub use check_document_files::*;
pub use dependencies::*;
pub use directory_entry::*;
pub use list_dir::*;
pub use load::*;
pub use read::*;
pub use read_document_files::*;
pub use read_file::*;
pub use save::*;
pub use write::*;
pub use write_document_files::*;
pub use write_file::*;

// Optional API

#[cfg(feature = "voxj")]
mod voxj_version_from_bytes;

#[cfg(feature = "voxj")]
pub use voxj_version_from_bytes::*;

// Internal API

mod internal;
pub(crate) use internal::*;
