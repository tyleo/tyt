use crate::ext::VMaxExt;
use std::any::Any;
use voxcore::{
    VoxMap,
    ext::{FollowListing, Result, VoxExt, encode_entry},
};

/// The Voxel Max ext as a state's ext. Its block is the `vmax` entry. The
/// node, palette, and object lists follow their listings. Each palette's slot
/// list follows the palette's materials. A retained node takes a default
/// entry, which the writer fills in like a synthesized node. A retained
/// material takes slot 0.
impl VoxExt for VMaxExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        encode_entry(self)
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
