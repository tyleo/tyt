use crate::{GltfExtAnimationChannel, GltfExtAnimationSampler};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// An animation preserved in the `gltf` ext, in stored order, its channels
/// targeting nodes by id and its keyframes in glTF's Y-up axes.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GltfExtAnimation {
    /// Animation name.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub name: Option<String>,

    /// The channels, in stored order.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub channels: Vec<GltfExtAnimationChannel>,

    /// The samplers, in stored order.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub samplers: Vec<GltfExtAnimationSampler>,

    /// The animation's `extras`, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extras: Option<Value>,

    /// The animation's extensions, preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Map::is_empty")
    )]
    pub extensions: Map<String, Value>,
}
