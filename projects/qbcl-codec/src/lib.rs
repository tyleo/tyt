//! Reads and writes the Qubicle voxel editor's binary file formats.
//!
//! Each format is a default-on Cargo feature: [`qb`] for Qubicle Binary
//! (`.qb`) and [`qbt`] for Qubicle Binary Tree (`.qbt`). See the `qbcl` crate
//! for the data types.

mod error;
mod result;

pub use error::*;
pub use result::*;

#[cfg(any(feature = "qb", feature = "qbt"))]
mod internal;
#[cfg(any(feature = "qb", feature = "qbt"))]
pub(crate) use internal::*;

#[cfg(feature = "qb")]
pub mod qb;

#[cfg(feature = "qbt")]
pub mod qbt;
