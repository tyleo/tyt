use crate::VoxjMain;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The root of a Voxel Json document.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VoxjFile {
    pub version: u32,

    pub main: VoxjMain,
}
