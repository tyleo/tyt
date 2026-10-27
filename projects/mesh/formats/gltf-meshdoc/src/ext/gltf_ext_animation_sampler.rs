use crate::GltfExtAnimationOutput;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An animation sampler preserved in the `gltf` ext.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GltfExtAnimationSampler {
    /// The interpolation, `LINEAR`, `STEP`, or `CUBICSPLINE`.
    pub interpolation: String,

    /// The keyframe times, in seconds.
    pub input: Vec<f32>,

    /// The keyframe values.
    pub output: GltfExtAnimationOutput,

    /// The sampler's `extras`, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extras: Option<Value>,
}
