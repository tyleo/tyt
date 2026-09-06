#![deny(rustdoc::broken_intra_doc_links)]

//! Reads and writes voxel file formats through the voxcore state.
//!
//! Each format's `-voxcore` bridge crate owns its conversion. This crate
//! fronts them. [`ReadFormat`] and [`WriteFormat`] pick a bridge. A document
//! travels as a list of [`VoxDocumentFile`]. A state's ext moves between the
//! bridge's typed ext and whatever slot the caller's
//! [`VoxMain`](voxcore::VoxMain) uses, through voxcore's
//! [`VoxExtSlot`](voxcore::ext::VoxExtSlot). The `codec` module, behind the
//! `codec` feature, holds the functions that move a document through a
//! bridge and the document checks, each over the caller's dependencies. A
//! check comes back as voxcore's [`VoxCheck`](voxcore::check::VoxCheck), so
//! a renderer elsewhere lays every format's checks out the same way. The
//! `vmax` and `voxj` modules hold those formats' writer options.

#[cfg(not(feature = "_format"))]
compile_error!(
    "voxconv needs at least one format feature enabled: goxl, mvox, qbcl, vmax, or voxj"
);

// Public API

mod error;
mod read_format;
mod result;
mod vox_document_file;
mod write_format;

pub use error::*;
pub use read_format::*;
pub use result::*;
pub use vox_document_file::*;
pub use write_format::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;

#[cfg(feature = "vmax")]
pub mod vmax;

#[cfg(feature = "voxj")]
pub mod voxj;
