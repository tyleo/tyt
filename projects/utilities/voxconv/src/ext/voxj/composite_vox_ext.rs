use crate::ext::VoxconvExt;
use branded_id::U32Id;
use voxcore::{BVoxVoxel, VoxExt};

/// Several exts as one: a Voxel Json document's `ext` block with each entry as
/// an ext, in the block's order. Every hook forwards to every entry, so a
/// decoded ext follows a mutation while an inert one cannot.
/// `composite_vox_ext_from_voxj_vox_ext` builds one from a block, and
/// [`voxj_vox_ext_from_ext`](crate::ext::voxj_vox_ext_from_ext) takes it back,
/// erroring when two entries share a key.
#[derive(Clone, Debug, Default)]
pub struct CompositeVoxExt {
    /// The exts, in the block's order.
    pub exts: Vec<Box<dyn VoxconvExt>>,
}

impl VoxExt for CompositeVoxExt {
    fn hierarchy_node_did_retain(&mut self, index: usize) {
        for ext in &mut self.exts {
            ext.hierarchy_node_did_retain(index);
        }
    }

    fn hierarchy_node_will_release(&mut self, index: usize) {
        for ext in &mut self.exts {
            ext.hierarchy_node_will_release(index);
        }
    }

    fn object_did_retain(&mut self, index: usize) {
        for ext in &mut self.exts {
            ext.object_did_retain(index);
        }
    }

    fn object_will_release(&mut self, index: usize) {
        for ext in &mut self.exts {
            ext.object_will_release(index);
        }
    }

    fn object_did_move(&mut self, from: usize, to: usize) {
        for ext in &mut self.exts {
            ext.object_did_move(from, to);
        }
    }

    fn palette_did_retain(&mut self, index: usize) {
        for ext in &mut self.exts {
            ext.palette_did_retain(index);
        }
    }

    fn palette_will_release(&mut self, index: usize) {
        for ext in &mut self.exts {
            ext.palette_will_release(index);
        }
    }

    fn palette_did_move(&mut self, from: usize, to: usize) {
        for ext in &mut self.exts {
            ext.palette_did_move(from, to);
        }
    }

    fn material_did_retain(&mut self, palette: usize, index: usize) {
        for ext in &mut self.exts {
            ext.material_did_retain(palette, index);
        }
    }

    fn materials_will_release(&mut self, palette: usize, indices: &[usize]) {
        for ext in &mut self.exts {
            ext.materials_will_release(palette, indices);
        }
    }

    fn materials_did_repaint(&mut self, palette: usize, remap: &[(usize, usize)]) {
        for ext in &mut self.exts {
            ext.materials_did_repaint(palette, remap);
        }
    }

    fn voxel_did_retain(&mut self, object: usize, voxel: U32Id<BVoxVoxel>) {
        for ext in &mut self.exts {
            ext.voxel_did_retain(object, voxel);
        }
    }

    fn voxel_will_release(&mut self, object: usize, voxel: U32Id<BVoxVoxel>) {
        for ext in &mut self.exts {
            ext.voxel_will_release(object, voxel);
        }
    }
}
