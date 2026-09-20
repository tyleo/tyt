#![deny(rustdoc::broken_intra_doc_links)]

//! A typed expression language over per-swatch, per-voxel, per-face, and
//! per-corner values.

mod checker;
mod environment;
mod error;
mod evaluator;
mod function;
mod lexer;
mod parser;
mod result;

pub use checker::*;
pub use environment::*;
pub use error::*;
pub use evaluator::*;
pub use parser::*;
pub use result::*;
