// Public API

mod expression;
mod parse;
mod parse_expression;
mod parse_failure;
mod program;

pub use expression::*;
pub use parse::*;
pub use parse_expression::*;
pub use parse_failure::*;
pub use program::*;

// Internal API

mod binary_operator;
mod comparison_operator;
mod logical_operator;
#[allow(clippy::module_inception)]
mod parser;
mod syntax_binding;
mod syntax_node;
mod unary_operator;
mod unexpected;

pub(crate) use binary_operator::*;
pub(crate) use comparison_operator::*;
pub(crate) use logical_operator::*;
pub(crate) use parser::*;
pub(crate) use syntax_binding::*;
pub(crate) use syntax_node::*;
pub(crate) use unary_operator::*;
pub(crate) use unexpected::*;

// Test support

#[cfg(test)]
mod syntax_node_parser_ext;
