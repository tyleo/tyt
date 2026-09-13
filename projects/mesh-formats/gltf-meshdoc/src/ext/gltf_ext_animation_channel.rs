use branded_id::U32Id;
use meshdoc::BMeshHierarchyNode;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An animation channel preserved in the `gltf` ext. The property driven is
/// the sampler's output kind.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GltfExtAnimationChannel {
    /// Index into the animation's samplers.
    pub sampler: u32,

    /// The node driven.
    #[cfg_attr(feature = "serde", serde(rename = "target-node-id"))]
    pub target_node_id: U32Id<BMeshHierarchyNode>,

    /// The channel's `extras`, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extras: Option<Value>,
}
