#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A morph target preserved in the `gltf` ext: its displacement streams,
/// one entry per vertex where present.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GltfExtMorphTarget {
    /// The position displacements, if the target has them.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub positions: Option<Vec<[f32; 3]>>,

    /// The normal displacements, if the target has them.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub normals: Option<Vec<[f32; 3]>>,

    /// The tangent displacements, if the target has them.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub tangents: Option<Vec<[f32; 3]>>,
}
