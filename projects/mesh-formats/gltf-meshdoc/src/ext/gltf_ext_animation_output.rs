#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The keyframe values of an animation sampler, which say what the channel
/// playing it drives.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum GltfExtAnimationOutput {
    /// Node translations.
    Translations(Vec<[f32; 3]>),

    /// Node rotations, as unit quaternions.
    Rotations(Vec<[f32; 4]>),

    /// Node scales.
    Scales(Vec<[f32; 3]>),

    /// Morph target weights.
    MorphTargetWeights(Vec<f32>),
}

impl GltfExtAnimationOutput {
    /// The channel target path glTF names this output by.
    pub fn path(&self) -> &'static str {
        match self {
            GltfExtAnimationOutput::Translations(_) => "translation",
            GltfExtAnimationOutput::Rotations(_) => "rotation",
            GltfExtAnimationOutput::Scales(_) => "scale",
            GltfExtAnimationOutput::MorphTargetWeights(_) => "weights",
        }
    }
}
