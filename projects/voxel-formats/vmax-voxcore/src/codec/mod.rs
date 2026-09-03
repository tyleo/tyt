//! Reads and writes the files of a `.vmax` package, gated behind the `codec`
//! feature.

mod from_vmax_package;
mod to_vmax_package;

pub use from_vmax_package::*;
pub use to_vmax_package::*;
