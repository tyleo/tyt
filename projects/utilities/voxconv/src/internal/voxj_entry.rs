use crate::ext::VoxconvExt;
#[cfg(feature = "goxl")]
use goxl_voxcore::ext::GoxlExt;
#[cfg(feature = "mvox")]
use mvox_voxcore::ext::MVoxExt;
#[cfg(feature = "qbcl")]
use qbcl_voxcore::ext::{QbExt, QbclExt, QbtExt};
use serde::{Serialize, de::DeserializeOwned};
#[cfg(feature = "vmax")]
use vmax_voxcore::ext::VMaxExt;

/// A format's ext as an entry of a Voxel Json document's `ext` block: the value
/// under the format's key, transcoded through serde. The keys live here because
/// voxconv is where the formats meet Voxel Json. A bridge knows nothing of its
/// key.
pub trait VoxjEntry: VoxconvExt + Serialize + DeserializeOwned + Clone {
    /// The key the entry sits under.
    const KEY: &'static str;
}

#[cfg(feature = "goxl")]
impl VoxjEntry for GoxlExt {
    const KEY: &'static str = "goxl";
}

#[cfg(feature = "mvox")]
impl VoxjEntry for MVoxExt {
    const KEY: &'static str = "mvox";
}

#[cfg(feature = "qbcl")]
impl VoxjEntry for QbExt {
    const KEY: &'static str = "qb";
}

#[cfg(feature = "qbcl")]
impl VoxjEntry for QbtExt {
    const KEY: &'static str = "qbt";
}

#[cfg(feature = "qbcl")]
impl VoxjEntry for QbclExt {
    const KEY: &'static str = "qbcl";
}

#[cfg(feature = "vmax")]
impl VoxjEntry for VMaxExt {
    const KEY: &'static str = "vmax";
}
