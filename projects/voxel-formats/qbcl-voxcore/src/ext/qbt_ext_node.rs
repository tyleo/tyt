#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Per scene-node provenance in the `qbt` ext, keyed by hierarchy node id.
/// The name, position, grid, and per-voxel masks derive from the scene at
/// write time.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum QbtExtNode {
    /// A matrix node: a single voxel grid.
    Matrix {
        /// `[x, y, z]` local scale.
        #[cfg_attr(feature = "serde", serde(rename = "local-scale"))]
        local_scale: [u32; 3],

        /// `[x, y, z]` pivot, in voxel coordinates.
        pivot: [f32; 3],
    },

    /// A model node: groups child nodes, which are native.
    Model,

    /// A compound node: a baked voxel grid plus child nodes.
    Compound {
        /// `[x, y, z]` local scale.
        #[cfg_attr(feature = "serde", serde(rename = "local-scale"))]
        local_scale: [u32; 3],

        /// `[x, y, z]` pivot, in voxel coordinates.
        pivot: [f32; 3],
    },

    /// A node whose type id the qbcl crate does not model, preserved verbatim.
    Unknown {
        /// Node type id, as stored.
        #[cfg_attr(feature = "serde", serde(rename = "type-id"))]
        type_id: u32,

        /// Node data bytes.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Vec::is_empty")
        )]
        data: Vec<u8>,
    },
}
