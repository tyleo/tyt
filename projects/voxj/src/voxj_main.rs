use crate::{VoxjHierarchyNode, VoxjObject, VoxjPalette, VoxjValue};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The body of a Voxel Json document.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct VoxjMain {
    pub objects: Vec<VoxjObject>,
    pub palettes: Vec<VoxjPalette>,
    pub hierarchy_nodes: Vec<VoxjHierarchyNode>,
    /// Indices into [`hierarchy_nodes`](Self::hierarchy_nodes); the scene's
    /// roots.
    pub root_hierarchy_nodes: Vec<usize>,
    /// Optional namespace for user-defined extensions, conventionally
    /// vendor-keyed. The core format assigns it no meaning and guarantees
    /// nothing about its contents; consumers ignore extensions they do not
    /// recognize.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ext: Option<VoxjValue>,
}
