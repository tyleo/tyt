use crate::{VoxjHierarchyNode, VoxjObject, VoxjPalette, VoxjValuePool};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The runtime scene of a Voxel Json document.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct VoxjRuntimeState {
    /// The shared value pools palettes draw from, referenced by array index.
    pub value_pools: Vec<VoxjValuePool>,

    /// The palettes objects sample, referenced by array index.
    pub palettes: Vec<VoxjPalette>,

    /// The voxel objects, referenced by array index.
    pub objects: Vec<VoxjObject>,

    /// The hierarchy nodes, referenced by array index.
    pub nodes: Vec<VoxjHierarchyNode>,

    /// The scene's roots, as indices into [`nodes`](Self::nodes).
    pub root_nodes: Vec<usize>,
}
