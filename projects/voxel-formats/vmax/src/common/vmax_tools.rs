use crate::VMaxViewBox;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The part of a contents file's tool settings (`.vmaxb` `tools`) a render
/// reads: the work area `vp`, outside which Voxel Max hides an object's
/// voxels. Every other tool setting is editor state and is skipped.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxTools {
    /// The work area ([`VMaxViewBox`]), inclusive.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub vp: Option<VMaxViewBox>,
}
