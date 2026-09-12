use qbcl::qbcl::QbclModel;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The per-kind body of a scene node in the `qbcl` ext. The position, grid,
/// and per-voxel masks derive from the scene at write time.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum QbclExtNodeBody {
    /// A matrix node: a single voxel grid.
    Matrix {
        /// `[x, y, z]` pivot, in voxel coordinates.
        pivot: [f32; 3],
    },

    /// A model node: groups child nodes.
    Model {
        /// The 36-byte transform chunk Qubicle writes after a model header,
        /// preserved verbatim.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Vec::is_empty")
        )]
        transform: Vec<u8>,
    },

    /// A compound node: a baked voxel grid plus child nodes.
    Compound {
        /// `[x, y, z]` pivot, in voxel coordinates.
        pivot: [f32; 3],
    },
}

impl Default for QbclExtNodeBody {
    fn default() -> Self {
        QbclExtNodeBody::Model {
            transform: QbclModel::DEFAULT_TRANSFORM.to_vec(),
        }
    }
}
