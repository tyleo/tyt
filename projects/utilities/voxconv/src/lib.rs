#![deny(rustdoc::broken_intra_doc_links)]

//! Reads and writes voxel file formats through the voxcore state.
//!
//! Each format's `-voxcore` bridge crate owns its conversion. This crate fronts
//! them. A format is a marker type implementing [`Format`], one per enabled
//! format feature. [`ReadFormat`] and [`WriteFormat`] pick one at runtime, and
//! their `with` methods hand the marker to a visitor as a type. A document
//! travels as a list of [`VoxDocumentFile`]. [`read()`] and [`write()`] move a
//! document through a format over the caller's [`Dependencies`] as a bare
//! [`VoxMain`](voxcore::VoxMain): a read takes the format's ext off, and a
//! write gives the state a synthesized ext through the bridge's
//! `to_x_vox_main` and writes it through the bridge's one typed path.
//! [`load()`] and [`save()`] start and end at a path instead of the files.
//! The `ext` feature, on by default, opens the `ext` module, where
//! `FormatExt` extends each format with its typed path and the `_with_ext`
//! pairs carry the format's ext boxed in a `VoxconvVoxMain`.
//! [`check_document_files()`] returns each check as voxcore's
//! [`VoxCheck`](voxcore::check::VoxCheck), so a renderer elsewhere lays every
//! format's checks out the same way. Each format feature enables its bridge's
//! `codec` feature. The `ext` feature enables every enabled bridge's `serde`
//! feature for the Voxel Json transcode. Each format feature opens a module of
//! its name holding the marker and the format's writer options. Voxel Json's
//! `ext` block types sit under `voxj::ext`.

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
mod format;
mod forward_dependencies;
mod installed_format;
mod list_dir;
mod load;
mod read;
mod read_document_files;
mod read_file;
mod read_format;
mod read_format_visitor;
mod result;
mod save;
mod vox_document_file;
mod write;
mod write_document_files;
mod write_file;
mod write_format;
mod write_format_visitor;

pub use check_document_files::*;
pub use dependencies::*;
pub use directory_entry::*;
pub use error::*;
pub use format::*;
pub use forward_dependencies::*;
pub use installed_format::*;
pub use list_dir::*;
pub use load::*;
pub use read::*;
pub use read_document_files::*;
pub use read_file::*;
pub use read_format::*;
pub use read_format_visitor::*;
pub use result::*;
pub use save::*;
pub use vox_document_file::*;
pub use write::*;
pub use write_document_files::*;
pub use write_file::*;
pub use write_format::*;
pub use write_format_visitor::*;

// Optional API

#[cfg(feature = "ext")]
pub mod ext;

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;

// One module per format feature.

#[cfg(feature = "goxl")]
pub mod goxl;

#[cfg(feature = "mvox")]
pub mod mvox;

#[cfg(feature = "qbcl")]
pub mod qbcl;

#[cfg(feature = "vmax")]
pub mod vmax;

#[cfg(feature = "voxj")]
pub mod voxj;

// Test support.

#[cfg(test)]
mod test;

#[cfg(test)]
pub(crate) use test::*;
