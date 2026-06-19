use crate::{VXBrush, VXCamera, VXSnapshot, VXTools};
use serde::{Deserialize, Serialize};

/// Voxel object payload of a `contents*.vmaxb` binary plist. Beyond the voxel
/// `snapshots`, the top level carries the editor state Voxel Max requires to open
/// and import the object (the content `uuid`/version and `tools`/`brush`/`cam`).
/// Decode the snapshots into voxels with `vmax_codec::decode_snapshots`; rebuild
/// from preserved state with [`VXObjectState`](crate::VXObjectState).
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct VXObjectData {
    /// Per-chunk voxel snapshots — the baked geometry edit log.
    #[serde(default)]
    pub snapshots: Vec<VXSnapshot>,
    /// Object content UUID (`uuid`).
    #[serde(default)]
    pub uuid: String,
    /// Codable version (`v`).
    #[serde(default)]
    pub v: i64,
    /// Tool state (`tools`); preserved for Voxel Max, absent from the voxj geometry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<VXTools>,
    /// Brush palette (`brush`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brush: Option<VXBrush>,
    /// Per-object camera (`cam`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cam: Option<VXCamera>,
}
