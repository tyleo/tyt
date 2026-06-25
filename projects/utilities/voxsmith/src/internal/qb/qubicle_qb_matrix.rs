use serde::{Deserialize, Serialize};

/// Per-matrix provenance preserved in the `qubicle-qb` ext, aligned by index with
/// the objects and hierarchy nodes, in stored order.
///
/// A matrix's geometry and colors become a native object; this keeps the name,
/// scene position, and the per-voxel visibility bytes the voxcore object cannot
/// represent. The grid size is the object's bounds.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct QubicleQbMatrix {
    /// Matrix name.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,

    /// `[x, y, z]` position in the scene.
    pub position: [i32; 3],

    /// Per solid voxel, its visibility byte, in the object's live-voxel raster
    /// order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub visibility: Vec<u8>,
}
