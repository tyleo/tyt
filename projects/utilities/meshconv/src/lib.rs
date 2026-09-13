#![deny(rustdoc::broken_intra_doc_links)]

//! Reads and writes mesh file formats through the meshdoc state.
//!
//! Each format's `-meshdoc` bridge crate owns its conversion. This crate
//! fronts them. A format is a marker type implementing [`Format`], one per
//! enabled format feature. [`ReadFormat`] and [`WriteFormat`] pick one at
//! runtime, and their `with` methods hand the marker to a visitor as a type.
//! A document travels as a list of [`MeshDocumentFile`]. [`read()`] and
//! [`write()`] move a document through a format over the caller's
//! [`Dependencies`] as a bare [`MeshMain`](meshdoc::MeshMain): a read drops
//! the format's ext, and a write synthesizes one through the bridge's
//! `to_x_mesh_main`. [`load()`] and [`save()`] start and end at a path.
//! The `ext` feature, on by default, opens the `ext` module, whose
//! `_with_ext` pairs carry the format's ext boxed in a `MeshconvMeshMain`.
//! [`check_document_files()`] returns each check as meshdoc's
//! [`MeshCheck`](meshdoc::check::MeshCheck), so a renderer elsewhere lays
//! every format's checks out the same way. Each format feature enables its
//! bridge's `codec` feature and opens a module of its name holding the
//! marker and the format's writer options.

#[cfg(not(feature = "gltf"))]
compile_error!("meshconv needs at least one format feature enabled: gltf");

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
mod mesh_document_file;
mod read;
mod read_document_files;
mod read_file;
mod read_format;
mod read_format_visitor;
mod result;
mod save;
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
pub use mesh_document_file::*;
pub use read::*;
pub use read_document_files::*;
pub use read_file::*;
pub use read_format::*;
pub use read_format_visitor::*;
pub use result::*;
pub use save::*;
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

#[cfg(feature = "gltf")]
pub mod gltf;

// Test support.

#[cfg(all(test, feature = "impl"))]
mod test;

#[cfg(all(test, feature = "impl"))]
pub(crate) use test::*;
