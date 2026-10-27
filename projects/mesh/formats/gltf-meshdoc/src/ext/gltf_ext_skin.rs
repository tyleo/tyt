use branded_id::U32Id;
use meshdoc::BMeshHierarchyNode;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A skin preserved in the `gltf` ext, in stored order, its joints and
/// skeleton by node id.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GltfExtSkin {
    /// Skin name.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub name: Option<String>,

    /// The inverse bind matrices, column-major, one per joint, if the skin
    /// has them.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "inverse-bind-matrices",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub inverse_bind_matrices: Option<Vec<[f32; 16]>>,

    /// The joint nodes, in stored order.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "joint-node-ids",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub joint_node_ids: Vec<U32Id<BMeshHierarchyNode>>,

    /// The skeleton root node, if declared.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "skeleton-node-id",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub skeleton_node_id: Option<U32Id<BMeshHierarchyNode>>,

    /// The skin's `extras`, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extras: Option<Value>,

    /// The skin's extensions, preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Map::is_empty")
    )]
    pub extensions: Map<String, Value>,
}
