use crate::VoxjMain;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The root of a Voxel Json document.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VoxjFile {
    /// Format version.
    pub version: u32,

    /// The document body.
    pub main: VoxjMain,
}
