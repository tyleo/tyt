#![deny(rustdoc::broken_intra_doc_links)]

//! Data types for the Qubicle voxel editor's binary file formats.
//!
//! Each format is a default-on Cargo feature: [`qb`] for Qubicle Binary
//! (`.qb`), [`qbt`] for Qubicle Binary Tree (`.qbt`), and [`qbcl`] for Qubicle
//! Construction Library (`.qbcl`). The [`validation`] module checks a decoded
//! file's grids against their declared sizes. The `qbcl-codec` crate reads and
//! writes the bytes.

pub mod validation;

#[cfg(feature = "qb")]
pub mod qb;

#[cfg(feature = "qbcl")]
pub mod qbcl;

#[cfg(feature = "qbt")]
pub mod qbt;
