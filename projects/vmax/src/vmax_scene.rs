use crate::VMaxObject;

/// A Voxel Max scene parsed from `scene.json`.
#[derive(Clone, Debug, PartialEq)]
pub struct VMaxScene {
    pub objects: Vec<VMaxObject>,
}
