//! Reads and writes the files of a `.vmax` package, gated behind the `codec`
//! feature. With the `ext` feature, `from_vmax_package_with_ext` and
//! `to_vmax_package_with_ext` move a package through the typed path.

mod from_vmax_package;
mod to_vmax_package;

pub use from_vmax_package::*;
pub use to_vmax_package::*;

#[cfg(feature = "ext")]
mod from_vmax_package_with_ext;

#[cfg(feature = "ext")]
mod to_vmax_package_with_ext;

#[cfg(feature = "ext")]
pub use from_vmax_package_with_ext::*;

#[cfg(feature = "ext")]
pub use to_vmax_package_with_ext::*;

// Re-exported so a caller can name the dependencies the functions here take
// and bind them through the codec's impl behind `impl`.
pub use vmax_codec::dependencies;
