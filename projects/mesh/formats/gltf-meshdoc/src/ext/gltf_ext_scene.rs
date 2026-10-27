use branded_id::U32Id;
use meshdoc::BMeshHierarchyNode;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A scene preserved in the `gltf` ext, in stored order. Scenes have no
/// native meshdoc home. Meshdoc's roots are the nodes nothing places.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GltfExtScene {
    /// Scene name.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub name: Option<String>,

    /// The nodes the scene lists, in stored order.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "node-ids", default, skip_serializing_if = "Vec::is_empty")
    )]
    pub node_ids: Vec<U32Id<BMeshHierarchyNode>>,

    /// The scene's `extras`, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extras: Option<Value>,

    /// The scene's extensions, preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Map::is_empty")
    )]
    pub extensions: Map<String, Value>,
}
