use crate::GltfExtMorphTarget;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// Per-primitive state preserved in the `gltf` ext, keyed by the
/// primitive's id.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GltfExtPrimitive {
    /// The `JOINTS_n` streams, keyed by set, one entry per vertex.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub joints: BTreeMap<u32, Vec<[u16; 4]>>,

    /// The `WEIGHTS_n` streams, keyed by set, one entry per vertex.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub weights: BTreeMap<u32, Vec<[f32; 4]>>,

    /// The morph targets, in stored order.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub targets: Vec<GltfExtMorphTarget>,

    /// The primitive's `extras`, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extras: Option<Value>,

    /// The primitive's extensions, preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Map::is_empty")
    )]
    pub extensions: Map<String, Value>,
}
