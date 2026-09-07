#![deny(rustdoc::broken_intra_doc_links)]

//! Reads and writes voxel file formats through the voxcore state.
//!
//! Each format's `-voxcore` bridge crate owns its conversion. This crate
//! fronts them. [`ReadFormat`] and [`WriteFormat`] pick a bridge. A document
//! travels as a list of [`VoxDocumentFile`]. A state's ext moves between the
//! bridge's typed ext and whatever ext type the caller's
//! [`VoxMain`](voxcore::VoxMain) carries: through voxcore's
//! [`VoxExtBlockCodec`](voxcore::ext::VoxExtBlockCodec) on a read and
//! [`VoxExt`](voxcore::ext::VoxExt) on a write. [`read()`] and
//! [`write()`] move a document through a bridge over the caller's
//! [`Dependencies`]. [`load()`] and [`save()`] start and end at a path
//! instead of the files. [`check_document_files()`] returns each check as
//! voxcore's [`VoxCheck`](voxcore::check::VoxCheck), so a renderer elsewhere
//! lays every format's checks out the same way. Each format feature enables
//! its bridge's `codec` and `ext` features. The `vmax` and `voxj` modules
//! hold those formats' writer options.

#[cfg(not(any(
    feature = "goxl",
    feature = "mvox",
    feature = "qbcl",
    feature = "vmax",
    feature = "voxj"
)))]
compile_error!(
    "voxconv needs at least one format feature enabled: goxl, mvox, qbcl, vmax, or voxj"
);

// Public API

mod check_document_files;
mod dependencies;
mod directory_entry;
mod error;
mod list_dir;
mod load;
mod read;
mod read_document_files;
mod read_file;
mod read_format;
mod result;
mod save;
mod vox_document_file;
mod write;
mod write_document_files;
mod write_file;
mod write_format;

pub use check_document_files::*;
pub use dependencies::*;
pub use directory_entry::*;
pub use error::*;
pub use list_dir::*;
pub use load::*;
pub use read::*;
pub use read_document_files::*;
pub use read_file::*;
pub use read_format::*;
pub use result::*;
pub use save::*;
pub use vox_document_file::*;
pub use write::*;
pub use write_document_files::*;
pub use write_file::*;
pub use write_format::*;

// Optional API

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;

#[cfg(feature = "vmax")]
pub mod vmax;

#[cfg(feature = "voxj")]
pub mod voxj;

#[cfg(feature = "voxj")]
mod voxj_version_from_bytes;

#[cfg(feature = "voxj")]
pub use voxj_version_from_bytes::*;

// Internal API

mod internal;
pub(crate) use internal::*;
