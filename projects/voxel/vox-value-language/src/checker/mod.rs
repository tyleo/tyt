// Public API

mod check;
mod check_expression;
mod check_expression_in_context;
mod check_failure;
mod checked_expression;
mod checked_program;
mod reads;

pub use check::*;
pub use check_expression::*;
pub use check_expression_in_context::*;
pub use check_failure::*;
pub use checked_expression::*;
pub use checked_program::*;
pub use reads::*;

// Internal API

mod check_result;
mod check_root;
mod checked;
mod checked_binding;
mod checked_kind;
mod checked_node;
mod elementwise_function;
mod fold;
mod number_value;
mod pending;
mod reduction;
mod rounding;

pub(crate) use check_result::*;
pub(crate) use check_root::*;
pub(crate) use checked::*;
pub(crate) use checked_binding::*;
pub(crate) use checked_kind::*;
pub(crate) use checked_node::*;
pub(crate) use elementwise_function::*;
pub(crate) use fold::*;
pub(crate) use number_value::*;
pub(crate) use pending::*;
pub(crate) use reduction::*;
pub(crate) use rounding::*;

// Test support

#[cfg(test)]
mod checked_node_checker_ext;
