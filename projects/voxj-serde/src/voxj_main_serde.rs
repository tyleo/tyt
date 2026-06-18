use crate::{VoxjHierarchyNodeSerde, VoxjObjectSerde, VoxjPaletteSerde};
use serde::{Deserialize, Serialize};
use voxj::VoxjMain;

/// Serde-compatible parity type for [`VoxjMain`].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoxjMainSerde {
    pub objects: Vec<VoxjObjectSerde>,
    pub palettes: Vec<VoxjPaletteSerde>,
    pub hierarchy_nodes: Vec<VoxjHierarchyNodeSerde>,
    pub root_hierarchy_nodes: Vec<usize>,
}

impl From<VoxjMain> for VoxjMainSerde {
    fn from(v: VoxjMain) -> Self {
        Self {
            objects: v.objects.into_iter().map(Into::into).collect(),
            palettes: v.palettes.into_iter().map(Into::into).collect(),
            hierarchy_nodes: v.hierarchy_nodes.into_iter().map(Into::into).collect(),
            root_hierarchy_nodes: v.root_hierarchy_nodes,
        }
    }
}

impl From<VoxjMainSerde> for VoxjMain {
    fn from(v: VoxjMainSerde) -> Self {
        Self {
            objects: v.objects.into_iter().map(Into::into).collect(),
            palettes: v.palettes.into_iter().map(Into::into).collect(),
            hierarchy_nodes: v.hierarchy_nodes.into_iter().map(Into::into).collect(),
            root_hierarchy_nodes: v.root_hierarchy_nodes,
        }
    }
}
