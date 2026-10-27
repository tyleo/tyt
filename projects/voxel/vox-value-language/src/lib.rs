#![deny(rustdoc::broken_intra_doc_links)]

//! A typed expression language over per-swatch, per-voxel, per-face, and
//! per-corner values.

// Public API

mod checker;
mod environment;
mod error;
mod evaluator;
mod parser;
mod result;

pub use checker::*;
pub use environment::*;
pub use error::*;
pub use evaluator::*;
pub use parser::*;
pub use result::*;

// Internal API

mod function;
mod lexer;

pub(crate) use function::*;
pub(crate) use lexer::*;

// Test support

#[cfg(test)]
mod test;

#[cfg(test)]
pub(crate) use test::*;
