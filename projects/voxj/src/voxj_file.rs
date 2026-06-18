use crate::VoxjMain;

/// The root of a Voxel Json document.
#[derive(Clone, Debug, PartialEq)]
pub struct VoxjFile {
    pub version: u32,
    pub main: VoxjMain,
}
