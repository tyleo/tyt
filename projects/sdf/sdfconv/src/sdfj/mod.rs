//! SDF Json: the [`Sdfj`] format and [`SdfjSerialization`] picking the
//! container.

// Public API

#[allow(clippy::module_inception)]
mod sdfj;
mod sdfj_dependencies;
mod sdfj_serialization;

pub use sdfj::*;
pub use sdfj_dependencies::*;
pub use sdfj_serialization::*;

// Optional API

#[cfg(feature = "impl")]
mod sdfj_dependencies_impl;

// Internal API

mod from_sdfj_error;
