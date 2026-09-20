#![deny(rustdoc::broken_intra_doc_links)]

//! A typed expression language over per-swatch, per-voxel, per-face, and
//! per-corner values.

mod environment;
mod error;
mod function;
mod lexer;
mod parser;
mod result;

pub use environment::*;
pub use error::*;
pub use parser::*;
pub use result::*;
