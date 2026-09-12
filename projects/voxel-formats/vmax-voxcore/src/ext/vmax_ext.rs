use crate::{VMaxExtNode, VMaxExtObjectState, VMaxExtPalette};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use vmax::VMaxSceneJsonFile;
use voxcore::VoxExt;

/// The `vmax` ext payload stashed on a [`VoxMain`](voxcore::VoxMain): the
/// Voxel Max state with no native voxcore home, kept so a document loaded from
/// a Voxel Max package can be written back exactly.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxExt {
    /// The Voxel Max scene with its hierarchy emptied, holding only the
    /// scene-level state recorded around the objects and groups that voxcore
    /// represents natively.
    pub scene: VMaxSceneJsonFile,

    /// Per-node provenance, aligned by index with the hierarchy nodes, groups
    /// before objects.
    #[cfg_attr(feature = "serde", serde(rename = "hierarchy-nodes"))]
    pub hierarchy_nodes: Vec<VMaxExtNode>,

    /// Per-palette provenance, aligned by index with the palettes. A palette
    /// added after the load holds nothing.
    pub palettes: Vec<Option<VMaxExtPalette>>,

    /// Per-object editor state, aligned by index with the objects. An object
    /// with no `contents*.vmaxb` holds nothing.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "object-states",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub object_states: Vec<Option<VMaxExtObjectState>>,
}

/// The Voxel Max ext as a state's ext. The node, palette, and object lists
/// follow their listings. Each palette's slot list follows the palette's
/// materials. A retained node takes a default entry, which the writer fills
/// in like a synthesized node. A retained material takes slot 0.
impl VoxExt for VMaxExt {
    fn hierarchy_node_did_retain(&mut self, index: usize) {
        self.hierarchy_nodes.insert(index, VMaxExtNode::default());
    }

    fn hierarchy_node_will_release(&mut self, index: usize) {
        self.hierarchy_nodes.remove(index);
    }

    fn object_did_retain(&mut self, index: usize) {
        self.object_states.insert(index, None);
    }

    fn object_will_release(&mut self, index: usize) {
        self.object_states.remove(index);
    }

    fn object_did_move(&mut self, from: usize, to: usize) {
        let state = self.object_states.remove(from);
        self.object_states.insert(to, state);
    }

    fn palette_did_retain(&mut self, index: usize) {
        self.palettes.insert(index, None);
    }

    fn palette_will_release(&mut self, index: usize) {
        self.palettes.remove(index);
    }

    fn palette_did_move(&mut self, from: usize, to: usize) {
        let palette = self.palettes.remove(from);
        self.palettes.insert(to, palette);
    }

    fn material_did_retain(&mut self, palette: usize, index: usize) {
        if let Some(palette) = &mut self.palettes[palette] {
            palette.slots.insert(index, 0);
        }
    }

    fn materials_will_release(&mut self, palette: usize, indices: &[usize]) {
        if let Some(palette) = &mut self.palettes[palette] {
            for &index in indices {
                palette.slots.remove(index);
            }
        }
    }
}
