use crate::GltfExtPrimitive;
use branded_id::U32Id;
use meshdoc::BMeshPrimitive;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// Per-mesh state preserved in the `gltf` ext, keyed by the object the glTF
/// mesh became.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GltfExtMesh {
    /// The mesh's default morph target weights, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub weights: Option<Vec<f32>>,

    /// Per-primitive state, keyed by primitive id.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub primitives: BTreeMap<U32Id<BMeshPrimitive>, GltfExtPrimitive>,

    /// The mesh's `extras`, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extras: Option<Value>,

    /// The mesh's extensions, preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Map::is_empty")
    )]
    pub extensions: Map<String, Value>,
}
