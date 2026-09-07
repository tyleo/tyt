use crate::{VMaxExtNode, VMaxExtObjectState, VMaxExtPalette};
#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};
use std::any::Any;
use vmax::VMaxSceneJsonFile;
#[cfg(not(feature = "ext"))]
use voxcore::ext::Error;
#[cfg(feature = "ext")]
use voxcore::ext::encode_entry;
use voxcore::{
    VoxMap,
    ext::{FollowListing, Result, VoxExt},
};

/// The `vmax` ext payload stashed on a [`VoxMain`](voxcore::VoxMain): the
/// Voxel Max state with no native voxcore home, kept so a document loaded from
/// a Voxel Max package can be written back exactly.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct VMaxExt {
    /// The Voxel Max scene with its hierarchy emptied, holding only the
    /// scene-level state recorded around the objects and groups that voxcore
    /// represents natively.
    pub scene: VMaxSceneJsonFile,

    /// Per-node provenance, aligned by index with the hierarchy nodes, groups
    /// before objects.
    #[cfg_attr(feature = "ext", serde(rename = "hierarchy-nodes"))]
    pub hierarchy_nodes: Vec<VMaxExtNode>,

    /// Per-palette provenance, aligned by index with the palettes. A palette
    /// added after the load holds nothing.
    pub palettes: Vec<Option<VMaxExtPalette>>,

    /// Per-object editor state, aligned by index with the objects. An object
    /// with no `contents*.vmaxb` holds nothing.
    #[cfg_attr(
        feature = "ext",
        serde(
            rename = "object-states",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub object_states: Vec<Option<VMaxExtObjectState>>,
}

/// The Voxel Max ext as a state's ext. Its block is the `vmax` entry.
/// Encoding it needs the `ext` feature. The node, palette, and object lists
/// follow their listings, and each palette's slot list follows the palette's
/// materials. A retained node takes a default entry, which the writer fills
/// in like a synthesized node. A retained material takes slot 0.
impl VoxExt for VMaxExt {
    #[cfg(feature = "ext")]
    fn to_vox_ext(&self) -> Result<VoxMap> {
        encode_entry(self)
    }

    #[cfg(not(feature = "ext"))]
    fn to_vox_ext(&self) -> Result<VoxMap> {
        Err(Error::Invalid(
            "the Voxel Max ext encodes its block only with the `ext` feature".to_owned(),
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn VoxExt> {
        Box::new(self.clone())
    }

    fn hierarchy_node_did_retain(&mut self, index: usize) {
        self.hierarchy_nodes.follow_retain(index);
    }

    fn hierarchy_node_will_release(&mut self, index: usize) {
        self.hierarchy_nodes.follow_release(index);
    }

    fn object_did_retain(&mut self, index: usize) {
        self.object_states.follow_retain(index);
    }

    fn object_will_release(&mut self, index: usize) {
        self.object_states.follow_release(index);
    }

    fn object_did_move(&mut self, from: usize, to: usize) {
        self.object_states.follow_move(from, to);
    }

    fn palette_did_retain(&mut self, index: usize) {
        self.palettes.follow_retain(index);
    }

    fn palette_will_release(&mut self, index: usize) {
        self.palettes.follow_release(index);
    }

    fn palette_did_move(&mut self, from: usize, to: usize) {
        self.palettes.follow_move(from, to);
    }

    fn material_did_retain(&mut self, palette: usize, index: usize) {
        if let Some(Some(palette)) = self.palettes.get_mut(palette) {
            palette.slots.follow_retain(index);
        }
    }

    fn materials_will_release(&mut self, palette: usize, indices: &[usize]) {
        if let Some(Some(palette)) = self.palettes.get_mut(palette) {
            for &index in indices {
                palette.slots.follow_release(index);
            }
        }
    }
}
