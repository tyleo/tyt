#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// The per-kind body of a scene node in the `qubicle-qbcl` ext, one variant per
/// node type. The geometry and colors of a matrix or compound become a native
/// object; this holds the placement, pivot, and the per-voxel visibility masks
/// the voxcore object cannot represent, plus a model's opaque transform chunk.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub enum QubicleQbclNodeBody {
    /// A matrix node: a single voxel grid. Its size is the object's grid
    /// bounds.
    Matrix {
        /// `[x, y, z]` position in the scene.
        position: [i32; 3],

        /// `[x, y, z]` pivot, in voxel coordinates.
        pivot: [f32; 3],

        /// Per solid voxel, its visibility mask, in the object's live-voxel
        /// raster order.
        #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
        masks: Vec<u8>,
    },

    /// A model node: groups child nodes.
    Model {
        /// The 36-byte transform chunk Qubicle writes after a model header,
        /// preserved verbatim.
        #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
        transform: Vec<u8>,
    },

    /// A compound node: a baked voxel grid plus child nodes.
    Compound {
        /// `[x, y, z]` position in the scene.
        position: [i32; 3],

        /// `[x, y, z]` pivot, in voxel coordinates.
        pivot: [f32; 3],

        /// Per solid voxel, its visibility mask, in the object's live-voxel
        /// raster order.
        #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
        masks: Vec<u8>,
    },
}

impl Default for QubicleQbclNodeBody {
    fn default() -> Self {
        QubicleQbclNodeBody::Model {
            transform: Vec::new(),
        }
    }
}
