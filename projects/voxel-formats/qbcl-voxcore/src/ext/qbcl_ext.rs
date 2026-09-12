use crate::{
    ext::{QbclExtMetadata, QbclExtNode, QbclExtThumbnail},
    rekey_hierarchy_nodes, synthesized_qbcl_ext_node,
};
use branded_id::U32Id;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, mem};
use voxcore::{
    BVoxHierarchyNode, Error as VoxError, Result as VoxResult, VoxExt, VoxGcRemap, VoxState,
};

/// The `qbcl` ext payload stashed on a [`VoxMain`](voxcore::VoxMain):
/// the Qubicle Construction Library `.qbcl` state with no native voxcore home,
/// kept so a file loaded from a `.qbcl` package can be written back exactly.
///
/// Matrix and compound grids become native objects sharing one palette, and
/// the scene tree becomes the hierarchy nodes. This holds the rest, one entry
/// per hierarchy node keyed by its id. The entries follow the state through
/// the [`VoxExt`](voxcore::VoxExt) hooks.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct QbclExt {
    /// The version of Qubicle that wrote the file, packed.
    #[cfg_attr(feature = "serde", serde(rename = "program-version"))]
    pub program_version: u32,

    /// The file-format version.
    #[cfg_attr(feature = "serde", serde(rename = "file-version"))]
    pub file_version: u32,

    /// The preview thumbnail from the header.
    #[cfg_attr(feature = "serde", serde(default))]
    pub thumbnail: QbclExtThumbnail,

    /// The seven free-text metadata strings.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: QbclExtMetadata,

    /// The 16-byte header chunk of unconfirmed purpose, preserved verbatim.
    pub guid: [u8; 16],

    /// Per scene-node provenance, keyed by hierarchy node id. Every node has
    /// an entry.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub nodes: BTreeMap<U32Id<BVoxHierarchyNode>, QbclExtNode>,
}

/// Nothing per object or voxel: the writer derives those from the scene. A
/// hook on a node the map does not know, or a retain of one it already
/// knows, refuses because the ext was out of step.
impl VoxExt for QbclExt {
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
            .insert(node_id, synthesized_qbcl_ext_node(node))
            .is_some()
        {
            return Err(VoxError::Ext {
                reason: format!(
                    "qbcl ext already has an entry for node {}",
                    node_id.to_u32()
                ),
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
                reason: format!("qbcl ext has no entry for node {}", node_id.to_u32()),
            });
        }
        Ok(())
    }

    fn did_gc(&mut self, _state: &VoxState, remap: &VoxGcRemap) -> VoxResult<()> {
        self.nodes = rekey_hierarchy_nodes("qbcl", mem::take(&mut self.nodes), remap)?;
        Ok(())
    }
}
