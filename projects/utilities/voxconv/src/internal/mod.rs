// Shared by the single-file formats.

#[cfg(any(feature = "goxl", feature = "mvox", feature = "qbcl", feature = "voxj"))]
mod single_file_bytes;

#[cfg(any(feature = "goxl", feature = "mvox", feature = "qbcl", feature = "voxj"))]
pub(crate) use single_file_bytes::*;

// Shared by the ext pairs.
#[cfg(feature = "ext")]
mod box_ext;

#[cfg(feature = "ext")]
pub(crate) use box_ext::*;

// Shared by the ext pairs of the formats with a keyed ext. The keys exist
// only with Voxel Json.
#[cfg(all(
    feature = "ext",
    any(feature = "goxl", feature = "mvox", feature = "qbcl", feature = "vmax")
))]
mod find_ext;
#[cfg(all(
    feature = "ext",
    feature = "voxj",
    any(feature = "goxl", feature = "mvox", feature = "qbcl", feature = "vmax")
))]
mod json;
#[cfg(all(
    feature = "ext",
    feature = "voxj",
    any(feature = "goxl", feature = "mvox", feature = "qbcl", feature = "vmax")
))]
mod voxj_entry;

#[cfg(all(
    feature = "ext",
    any(feature = "goxl", feature = "mvox", feature = "qbcl", feature = "vmax")
))]
pub(crate) use find_ext::*;
#[cfg(all(
    feature = "ext",
    feature = "voxj",
    any(feature = "goxl", feature = "mvox", feature = "qbcl", feature = "vmax")
))]
pub(crate) use json::*;
#[cfg(all(
    feature = "ext",
    feature = "voxj",
    any(feature = "goxl", feature = "mvox", feature = "qbcl", feature = "vmax")
))]
pub(crate) use voxj_entry::*;

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

#[cfg(all(test, feature = "impl"))]
mod memory_files;

#[cfg(all(test, feature = "impl"))]
pub(crate) use memory_files::*;
