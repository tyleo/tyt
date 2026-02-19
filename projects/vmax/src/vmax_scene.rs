use crate::{VMaxGroup, VMaxObject};

/// A Voxel Max scene parsed from `scene.json`.
#[derive(Clone, Debug, PartialEq)]
pub struct VMaxScene {
    pub groups: Vec<VMaxGroup>,
    pub objects: Vec<VMaxObject>,
}
