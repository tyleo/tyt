use crate::{MVoxExtFrame, MVoxExtShapeModel};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The per-kind body of a scene node in the `mvox` ext, one variant per
/// scene-graph chunk. The kind is provenance: a node with one child node is a
/// transform or a group by what the file said. The child links themselves come
/// from the voxcore node at write.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum MVoxExtNodeBody {
    /// An `nTRN` transform node: its layer and its animation frames. The
    /// voxcore node's transform projects the first frame, and the writer
    /// errors when the two no longer agree.
    Transform {
        /// The layer this node belongs to, or `-1` for none.
        layer: i32,

        /// The animation frames; a static node has exactly one.
        frames: Vec<MVoxExtFrame>,
    },

    /// An `nGRP` group node.
    Group,

    /// An `nSHP` shape node: the models it draws, in stored order.
    Shape {
        /// The shape models.
        models: Vec<MVoxExtShapeModel>,
    },
}
