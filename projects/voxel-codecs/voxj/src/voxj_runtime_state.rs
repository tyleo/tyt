use crate::{VoxjHierarchyNode, VoxjObject, VoxjPalette, VoxjValuePool};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The runtime scene of a Voxel Json document: the voxel objects, the shared
/// value pools their palettes draw from, the palettes they sample, the
/// hierarchy that places them, and the roots of that hierarchy. Held in
/// [`VoxjMain::runtime_state`](crate::VoxjMain::runtime_state), separate from
/// the optional editor `edit_state` and `ext`.
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

    /// Indices into [`nodes`](Self::nodes); the scene's roots.
    pub root_nodes: Vec<usize>,
}
