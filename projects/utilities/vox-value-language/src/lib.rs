#![deny(rustdoc::broken_intra_doc_links)]
// The lexer lands ahead of the parser that reads it.
#![allow(dead_code, unused_imports)]

//! A typed expression language over per-swatch, per-voxel, per-face, and
//! per-corner values.

mod error;
mod function;
mod lexer;
mod parse_failure;
mod result;

pub use error::*;
pub use parse_failure::*;
pub use result::*;
