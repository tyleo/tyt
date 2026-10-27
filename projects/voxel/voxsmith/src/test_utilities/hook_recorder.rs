use branded_id::U32Id;
use std::collections::HashMap;
use ty_math::{TyTransformF64, TyVector3I32, TyVector3U32};
use voxcore::{
    BVoxHierarchyNode, BVoxObject, BVoxPalette, BVoxVoxel, Result, VoxExt, VoxMain, VoxState,
};

/// Records the node, object, and palette-retain hooks a main fires, one line
/// apiece.
#[derive(Debug, Default)]
pub struct HookRecorder(Vec<String>);

impl HookRecorder {
    /// The hook lines recorded on `main`, in firing order.
    pub(crate) fn events(main: &VoxMain<HookRecorder>) -> Vec<&str> {
        main.ext().0.iter().map(String::as_str).collect()
    }
}

impl VoxExt for HookRecorder {
    fn hierarchy_node_did_retain(
        &mut self,
        _state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> Result<()> {
        self.0.push(format!("node {node_id} retained"));

        Ok(())
    }

    fn hierarchy_node_will_release(
        &mut self,
        _state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> Result<()> {
        self.0.push(format!("node {node_id} released"));

        Ok(())
    }

    fn hierarchy_node_name_did_set(
        &mut self,
        _state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
        _old_name: &str,
    ) -> Result<()> {
        self.0.push(format!("node {node_id} name set"));

        Ok(())
    }

    fn hierarchy_node_transform_did_set(
        &mut self,
        _state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
        _old_transform: TyTransformF64,
    ) -> Result<()> {
        self.0.push(format!("node {node_id} transform set"));

        Ok(())
    }

    fn hierarchy_node_children_did_set(
        &mut self,
        _state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
        _old_child_node_ids: &[U32Id<BVoxHierarchyNode>],
        _old_child_object_ids: &[U32Id<BVoxObject>],
    ) -> Result<()> {
        self.0.push(format!("node {node_id} children set"));

        Ok(())
    }

    fn root_hierarchy_node_ids_did_set(
        &mut self,
        _state: &VoxState,
        _old_root_ids: &[U32Id<BVoxHierarchyNode>],
    ) -> Result<()> {
        self.0.push("roots set".to_owned());

        Ok(())
    }

    fn object_did_retain(&mut self, _state: &VoxState, object_id: U32Id<BVoxObject>) -> Result<()> {
        self.0.push(format!("object {object_id} retained"));

        Ok(())
    }

    fn object_will_release(
        &mut self,
        _state: &VoxState,
        object_id: U32Id<BVoxObject>,
    ) -> Result<()> {
        self.0.push(format!("object {object_id} released"));

        Ok(())
    }

    fn object_did_move(
        &mut self,
        _state: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_index: usize,
    ) -> Result<()> {
        self.0
            .push(format!("object {object_id} moved from {old_index}"));

        Ok(())
    }

    fn object_name_did_set(
        &mut self,
        _state: &VoxState,
        object_id: U32Id<BVoxObject>,
        _old_name: &str,
    ) -> Result<()> {
        self.0.push(format!("object {object_id} name set"));

        Ok(())
    }

    fn object_origin_did_set(
        &mut self,
        _state: &VoxState,
        object_id: U32Id<BVoxObject>,
        _old_origin: TyVector3I32,
    ) -> Result<()> {
        self.0.push(format!("object {object_id} origin set"));

        Ok(())
    }

    fn object_voxels_did_remap(
        &mut self,
        _state: &VoxState,
        object_id: U32Id<BVoxObject>,
        _old_bounds: TyVector3U32,
        _voxel_ids: &HashMap<U32Id<BVoxVoxel>, U32Id<BVoxVoxel>>,
    ) -> Result<()> {
        self.0.push(format!("object {object_id} voxels remapped"));

        Ok(())
    }

    fn object_voxels_did_resample(
        &mut self,
        _state: &VoxState,
        object_id: U32Id<BVoxObject>,
        _old_bounds: TyVector3U32,
        _old_voxel_ids: &[U32Id<BVoxVoxel>],
    ) -> Result<()> {
        self.0.push(format!("object {object_id} voxels resampled"));

        Ok(())
    }

    fn palette_did_retain(
        &mut self,
        _state: &VoxState,
        palette_id: U32Id<BVoxPalette>,
    ) -> Result<()> {
        self.0.push(format!("palette {palette_id} retained"));

        Ok(())
    }
}
