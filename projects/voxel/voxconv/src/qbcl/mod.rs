//! Qubicle: the [`Qb`], [`Qbt`], and [`Qbcl`] formats and the dependencies
//! their codec takes.

// Public API

mod qb;
#[allow(clippy::module_inception)]
mod qbcl;
mod qbcl_dependencies;
mod qbt;

pub use qb::*;
pub use qbcl::*;
pub use qbcl_dependencies::*;
pub use qbt::*;

// Optional API

#[cfg(feature = "ext")]
mod ext;

#[cfg(feature = "impl")]
mod qbcl_dependencies_impl;

// Internal API

mod from_qbcl_error;
