#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// Per scene-node provenance in the `qubicle-qbt` ext, one variant per node
/// type. The geometry and colors of a matrix or compound become a native
/// object; this holds the name, placement, scale, pivot, and per-voxel
/// visibility masks the voxcore object cannot represent. Aligned by index with
/// the hierarchy nodes.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub enum QubicleQbtNode {
    /// A matrix node: a single voxel grid. Its size is the object's grid
    /// bounds.
    Matrix {
        /// Matrix name.
        #[cfg_attr(
            feature = "ext",
            serde(default, skip_serializing_if = "String::is_empty")
        )]
        name: String,

        /// `[x, y, z]` position in the scene.
        position: [i32; 3],

        /// `[x, y, z]` local scale.
        #[cfg_attr(feature = "ext", serde(rename = "local-scale"))]
        local_scale: [u32; 3],

        /// `[x, y, z]` pivot, in voxel coordinates.
        pivot: [f32; 3],

        /// Per solid voxel, its visibility mask, in the object's live-voxel
        /// raster order.
        #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
        masks: Vec<u8>,
    },

    /// A model node: groups child nodes, which are native.
    Model,

    /// A compound node: a baked voxel grid plus child nodes.
    Compound {
        /// Matrix name.
        #[cfg_attr(
            feature = "ext",
            serde(default, skip_serializing_if = "String::is_empty")
        )]
        name: String,

        /// `[x, y, z]` position in the scene.
        position: [i32; 3],

        /// `[x, y, z]` local scale.
        #[cfg_attr(feature = "ext", serde(rename = "local-scale"))]
        local_scale: [u32; 3],

        /// `[x, y, z]` pivot, in voxel coordinates.
        pivot: [f32; 3],

        /// Per solid voxel, its visibility mask, in the object's live-voxel
        /// raster order.
        #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
        masks: Vec<u8>,
    },

    /// A node whose type id the qbcl crate does not model, preserved verbatim.
    Unknown {
        /// Node type id, as stored.
        #[cfg_attr(feature = "ext", serde(rename = "type-id"))]
        type_id: u32,

        /// Node data bytes.
        #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
        data: Vec<u8>,
    },
}
