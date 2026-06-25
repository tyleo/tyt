//! Reads and writes the Qubicle voxel editor's binary file formats.
//!
//! Each format is a default-on Cargo feature: [`qb`] for Qubicle Binary
//! (`.qb`), [`qbt`] for Qubicle Binary Tree (`.qbt`), and [`qbcl`] for Qubicle
//! Construction Library (`.qbcl`). See the `qbcl` crate for the data types.

mod error;
mod result;

pub use error::*;
pub use result::*;

#[cfg(any(feature = "qb", feature = "qbt", feature = "qbcl"))]
mod internal;
#[cfg(any(feature = "qb", feature = "qbt", feature = "qbcl"))]
pub(crate) use internal::*;

#[cfg(feature = "qb")]
pub mod qb;

#[cfg(feature = "qbcl")]
pub mod qbcl;

#[cfg(feature = "qbt")]
pub mod qbt;
