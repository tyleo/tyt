//! Reads and writes the Qubicle voxel editor's binary file formats.
//!
//! Each format is a default-on Cargo feature: [`qb`] for Qubicle Binary
//! (`.qb`), [`qbt`] for Qubicle Binary Tree (`.qbt`), and [`qbcl`] for Qubicle
//! Construction Library (`.qbcl`). See the `qbcl` crate for the data types.
//! The `.qbt` and `.qbcl` readers and writers take their zlib codec through
//! [`DecompressZlib`] and [`CompressZlib`]. `DependenciesImpl`, behind the
//! default `impl` feature, supplies both over `flate2`.

#[cfg(not(any(feature = "qb", feature = "qbt", feature = "qbcl")))]
compile_error!("qbcl-codec needs at least one format feature enabled: qb, qbt, or qbcl");

// Public API

#[cfg(feature = "qb")]
pub mod qb;

#[cfg(feature = "qbcl")]
pub mod qbcl;

#[cfg(feature = "qbt")]
pub mod qbt;

mod compress_zlib;
mod decompress_zlib;
mod error;
mod result;

pub use compress_zlib::*;
pub use decompress_zlib::*;
pub use error::*;
pub use result::*;

// Optional API

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;

// Internal API

mod internal;
pub(crate) use internal::*;
