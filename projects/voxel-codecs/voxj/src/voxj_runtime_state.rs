use crate::{VoxjHierarchyNode, VoxjObject, VoxjPalette};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The runtime scene of a Voxel Json document: the voxel objects, the palettes
/// they sample, the hierarchy that places them, and the roots of that
/// hierarchy. Held in
/// [`VoxjMain::runtime_state`](crate::VoxjMain::runtime_state), separate from
/// the optional editor `edit_state` and `ext`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct VoxjRuntimeState {
    /// The voxel objects, referenced by array index.
    pub objects: Vec<VoxjObject>,

    /// The palettes objects sample, referenced by array index.
    pub palettes: Vec<VoxjPalette>,

    /// The hierarchy nodes, referenced by array index.
    pub hierarchy_nodes: Vec<VoxjHierarchyNode>,

    /// Indices into [`hierarchy_nodes`](Self::hierarchy_nodes); the scene's
    /// roots.
    pub root_hierarchy_nodes: Vec<usize>,
}
