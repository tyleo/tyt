// Shared by the single-file formats.

#[cfg(any(feature = "goxl", feature = "mvox", feature = "qbcl", feature = "voxj"))]
mod single_file_bytes;

#[cfg(any(feature = "goxl", feature = "mvox", feature = "qbcl", feature = "voxj"))]
pub(crate) use single_file_bytes::*;

// Shared by the formats whose ext moves through the block form.

#[cfg(any(feature = "goxl", feature = "mvox", feature = "qbcl", feature = "vmax"))]
mod into_ext_slot;

#[cfg(any(feature = "goxl", feature = "mvox", feature = "qbcl", feature = "vmax"))]
pub(crate) use into_ext_slot::*;

// One folder per format.

#[cfg(feature = "goxl")]
mod goxl;

#[cfg(feature = "goxl")]
pub(crate) use goxl::*;

#[cfg(feature = "mvox")]
mod mvox;

#[cfg(feature = "mvox")]
pub(crate) use mvox::*;

#[cfg(feature = "qbcl")]
mod qbcl;

#[cfg(feature = "qbcl")]
pub(crate) use qbcl::*;

#[cfg(feature = "vmax")]
mod vmax;

#[cfg(feature = "vmax")]
pub(crate) use vmax::*;

#[cfg(feature = "voxj")]
mod voxj;

#[cfg(feature = "voxj")]
pub(crate) use voxj::*;

// Test support.

#[cfg(test)]
mod test_state;

#[cfg(test)]
pub(crate) use test_state::*;
