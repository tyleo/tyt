use crate::{MagicaVoxelFrame, MagicaVoxelShapeModel};
#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// The per-kind body of a scene node in the `magica-voxel` ext, one variant per
/// scene-graph chunk. The voxcore node keeps a deduplicated structural view of
/// the same references; this holds their exact, possibly repeated form so the
/// scene graph rebuilds unchanged.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub enum MagicaVoxelNodeBody {
    /// An `nTRN` transform node: the id of the child it places, its layer, and
    /// its animation frames.
    Transform {
        /// The id of the child node this transform places.
        child: i32,

        /// The layer this node belongs to, or `-1` for none.
        layer: i32,

        /// The animation frames; a static node has exactly one.
        frames: Vec<MagicaVoxelFrame>,
    },

    /// An `nGRP` group node: the ids of its child nodes, in stored order.
    Group {
        /// The ids of the child nodes.
        children: Vec<i32>,
    },

    /// An `nSHP` shape node: the models it draws, in stored order.
    Shape {
        /// The shape models.
        models: Vec<MagicaVoxelShapeModel>,
    },
}

impl Default for MagicaVoxelNodeBody {
    fn default() -> Self {
        MagicaVoxelNodeBody::Group {
            children: Vec::new(),
        }
    }
}
