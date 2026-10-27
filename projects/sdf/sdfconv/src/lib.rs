#![deny(rustdoc::broken_intra_doc_links)]

//! Reads and writes SDF file formats through the sdfcore state.
//!
//! Each format's `-sdfcore` bridge crate owns its conversion. This crate
//! fronts them. A format is a marker type implementing [`Format`], one per
//! enabled format feature. [`ReadFormat`] and [`WriteFormat`] pick one at
//! runtime, and their `with` methods hand the marker to a visitor as a type.
//! [`read()`] and [`write()`] move a document's bytes through a format over the
//! caller's [`Dependencies`] as an [`SdfMain`](sdfcore::SdfMain). [`load()`]
//! and [`save()`] start and end at a path instead of the bytes. [`load()`] can
//! also load the document's fonts through its format. Each format feature
//! enables its bridge's `codec` feature and opens a module of its name holding
//! the marker and the format's writer options.

#[cfg(not(feature = "sdfj"))]
compile_error!("sdfconv needs at least one format feature enabled: sdfj");

// Public API

mod dependencies;
mod error;
mod format;
mod forward_dependencies;
mod load;
mod load_options;
mod read;
mod read_file;
mod read_format;
mod read_format_visitor;
mod result;
mod save;
mod write;
mod write_file;
mod write_format;
mod write_format_visitor;

pub use dependencies::*;
pub use error::*;
pub use format::*;
pub use forward_dependencies::*;
pub use load::*;
pub use load_options::*;
pub use read::*;
pub use read_file::*;
pub use read_format::*;
pub use read_format_visitor::*;
pub use result::*;
pub use save::*;
pub use write::*;
pub use write_file::*;
pub use write_format::*;
pub use write_format_visitor::*;

// Optional API

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;

#[cfg(feature = "sdfj")]
pub mod sdfj;

// Test support

#[cfg(all(test, feature = "impl"))]
mod test;

#[cfg(all(test, feature = "impl"))]
pub(crate) use test::*;
