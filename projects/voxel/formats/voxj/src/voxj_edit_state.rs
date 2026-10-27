use crate::VoxjEditObject;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Editor state for a Voxel Json document.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VoxjEditState {
    /// Per-object editor state, aligned by index with the runtime objects.
    pub objects: Vec<VoxjEditObject>,
}
