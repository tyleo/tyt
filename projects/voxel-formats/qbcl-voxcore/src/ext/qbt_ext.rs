use crate::{QbtExtNode, rekey_hierarchy_nodes, synthesized_qbt_ext_node};
use branded_id::U32Id;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, mem};
use voxcore::{
    BVoxHierarchyNode, Error as VoxError, Result as VoxResult, VoxExt, VoxGcRemap, VoxState,
};

/// The `qbt` ext payload stashed on a [`VoxMain`](voxcore::VoxMain):
/// the Qubicle Binary Tree `.qbt` state with no native voxcore home, kept so a
/// file loaded from a `.qbt` package can be written back exactly.
///
/// Matrix and compound grids become native objects sharing one palette, and
/// the scene tree becomes the hierarchy nodes. This holds the rest, one entry
/// per hierarchy node keyed by its id. The entries follow the state through
/// the [`VoxExt`](voxcore::VoxExt) hooks.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct QbtExt {
    /// The `(major, minor)` version from the header.
    pub version: (u8, u8),

    /// The `[x, y, z]` global scale applied to the whole model.
    #[cfg_attr(feature = "serde", serde(rename = "global-scale"))]
    pub global_scale: [f32; 3],

    /// The `COLORMAP` palette, in stored order, as `[r, g, b, a]` entries;
    /// empty when voxels store colors directly.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "color-map", default, skip_serializing_if = "Vec::is_empty")
    )]
    pub color_map: Vec<[u8; 4]>,

    /// Per scene-node provenance, keyed by hierarchy node id. Every node has
    /// an entry.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub nodes: BTreeMap<U32Id<BVoxHierarchyNode>, QbtExtNode>,
}

/// Nothing per object or voxel: the writer derives those from the scene. A
/// hook on a node the map does not know, or a retain of one it already
/// knows, refuses because the ext was out of step.
impl VoxExt for QbtExt {
    fn hierarchy_node_did_retain(
        &mut self,
        state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> VoxResult<()> {
        let node = state
            .hierarchy_node(node_id)
            .expect("a retained node is live");
        if self
            .nodes
            .insert(node_id, synthesized_qbt_ext_node(node))
            .is_some()
        {
            return Err(VoxError::Ext {
                reason: format!("qbt ext already has an entry for node {}", node_id.to_u32()),
            });
        }
        Ok(())
    }

    fn hierarchy_node_will_release(
        &mut self,
        _state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> VoxResult<()> {
        if self.nodes.remove(&node_id).is_none() {
            return Err(VoxError::Ext {
                reason: format!("qbt ext has no entry for node {}", node_id.to_u32()),
            });
        }
        Ok(())
    }

    fn did_gc(&mut self, _state: &VoxState, remap: &VoxGcRemap) -> VoxResult<()> {
        self.nodes = rekey_hierarchy_nodes("qbt", mem::take(&mut self.nodes), remap)?;
        Ok(())
    }
}
