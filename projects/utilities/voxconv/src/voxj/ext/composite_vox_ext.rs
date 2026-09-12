use crate::ext::VoxconvExt;
use branded_id::U32Id;
use std::collections::HashMap;
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxVoxel, Result, VoxExt,
    VoxGcRemap, VoxState,
};

/// Several exts as one: a Voxel Json document's `ext` block with each entry as
/// an ext, in the block's order. Every hook forwards to every entry and stops
/// at the first refusal. A decoded ext follows a mutation. An inert one
/// cannot.
/// `composite_vox_ext_from_voxj_vox_ext` builds one from a block, and
/// [`voxj_vox_ext_from_ext`](crate::voxj::ext::voxj_vox_ext_from_ext) takes it
/// back, erroring when two entries share a key.
#[derive(Clone, Debug, Default)]
pub struct CompositeVoxExt {
    /// The exts, in the block's order.
    pub exts: Vec<Box<dyn VoxconvExt>>,
}

impl VoxExt for CompositeVoxExt {
    fn hierarchy_node_did_retain(
        &mut self,
        main: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.hierarchy_node_did_retain(main, node_id)?;
        }
        Ok(())
    }

    fn hierarchy_node_will_release(
        &mut self,
        main: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.hierarchy_node_will_release(main, node_id)?;
        }
        Ok(())
    }

    fn object_did_retain(&mut self, main: &VoxState, object_id: U32Id<BVoxObject>) -> Result<()> {
        for ext in &mut self.exts {
            ext.object_did_retain(main, object_id)?;
        }
        Ok(())
    }

    fn object_will_release(&mut self, main: &VoxState, object_id: U32Id<BVoxObject>) -> Result<()> {
        for ext in &mut self.exts {
            ext.object_will_release(main, object_id)?;
        }
        Ok(())
    }

    fn palette_did_retain(
        &mut self,
        main: &VoxState,
        palette_id: U32Id<BVoxPalette>,
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.palette_did_retain(main, palette_id)?;
        }
        Ok(())
    }

    fn palette_will_release(
        &mut self,
        main: &VoxState,
        palette_id: U32Id<BVoxPalette>,
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.palette_will_release(main, palette_id)?;
        }
        Ok(())
    }

    fn material_did_retain(
        &mut self,
        main: &VoxState,
        palette_id: U32Id<BVoxPalette>,
        material_id: U32Id<BVoxMaterial>,
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.material_did_retain(main, palette_id, material_id)?;
        }
        Ok(())
    }

    fn materials_will_release(
        &mut self,
        main: &VoxState,
        palette_id: U32Id<BVoxPalette>,
        material_ids: &[U32Id<BVoxMaterial>],
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.materials_will_release(main, palette_id, material_ids)?;
        }
        Ok(())
    }

    fn materials_did_repaint(
        &mut self,
        main: &VoxState,
        palette_id: U32Id<BVoxPalette>,
        replacement_ids: &HashMap<U32Id<BVoxMaterial>, U32Id<BVoxMaterial>>,
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.materials_did_repaint(main, palette_id, replacement_ids)?;
        }
        Ok(())
    }

    fn voxel_did_retain(
        &mut self,
        main: &VoxState,
        object_id: U32Id<BVoxObject>,
        voxel_id: U32Id<BVoxVoxel>,
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.voxel_did_retain(main, object_id, voxel_id)?;
        }
        Ok(())
    }

    fn voxel_will_release(
        &mut self,
        main: &VoxState,
        object_id: U32Id<BVoxObject>,
        voxel_id: U32Id<BVoxVoxel>,
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.voxel_will_release(main, object_id, voxel_id)?;
        }
        Ok(())
    }

    fn did_gc(&mut self, main: &VoxState, remap: &VoxGcRemap) -> Result<()> {
        for ext in &mut self.exts {
            ext.did_gc(main, remap)?;
        }
        Ok(())
    }
}
