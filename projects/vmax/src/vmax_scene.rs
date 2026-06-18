use crate::{VMaxGroup, VMaxObject};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A Voxel Max scene parsed from `scene.json`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxScene {
    #[cfg_attr(feature = "serde", serde(default))]
    pub groups: Vec<VMaxGroup>,
    pub objects: Vec<VMaxObject>,
}
