#![deny(rustdoc::broken_intra_doc_links)]

// Public API

/// The subcommands [`Vxl`] and the `vxl integration` group dispatch to.
pub mod commands;
pub mod dependencies;

mod agent_skill;
mod error;
mod result;
mod vxl;

pub use agent_skill::*;
pub use dependencies::*;
pub use error::*;
pub use result::*;
pub use vxl::*;

// Internal API

mod internal;
pub(crate) use internal::*;

// Test support

#[cfg(test)]
mod test;

#[cfg(test)]
pub(crate) use test::*;
