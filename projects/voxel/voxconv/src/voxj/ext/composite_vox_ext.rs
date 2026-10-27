use crate::ext::VoxconvExt;
use branded_id::U32Id;
use std::collections::HashMap;
use ty_math::{TyTransformF64, TyVector3I32, TyVector3U32};
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxVoxel, Result, VoxExt,
    VoxGcRemap, VoxState,
};

/// Several exts as one: a Voxel Json document's `ext` block with each entry as
/// an ext, in the block's order. Every hook forwards to every entry and stops
/// at the first refusal. A decoded ext follows a mutation. An inert one cannot.
/// `composite_vox_ext_from_voxj_vox_ext` builds one from a block, and
/// [`voxj_vox_ext_from_ext`](crate::voxj::ext::voxj_vox_ext_from_ext()) takes
/// it back, erroring when two entries share a key.
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

    fn hierarchy_node_name_did_set(
        &mut self,
        main: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
        old_name: &str,
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.hierarchy_node_name_did_set(main, node_id, old_name)?;
        }
        Ok(())
    }

    fn hierarchy_node_transform_did_set(
        &mut self,
        main: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
        old_transform: TyTransformF64,
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.hierarchy_node_transform_did_set(main, node_id, old_transform)?;
        }
        Ok(())
    }

    fn hierarchy_node_children_did_set(
        &mut self,
        main: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
        old_child_node_ids: &[U32Id<BVoxHierarchyNode>],
        old_child_object_ids: &[U32Id<BVoxObject>],
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.hierarchy_node_children_did_set(
                main,
                node_id,
                old_child_node_ids,
                old_child_object_ids,
            )?;
        }
        Ok(())
    }

    fn root_hierarchy_node_ids_did_set(
        &mut self,
        main: &VoxState,
        old_root_ids: &[U32Id<BVoxHierarchyNode>],
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.root_hierarchy_node_ids_did_set(main, old_root_ids)?;
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

    fn object_did_move(
        &mut self,
        main: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_index: usize,
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.object_did_move(main, object_id, old_index)?;
        }
        Ok(())
    }

    fn object_name_did_set(
        &mut self,
        main: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_name: &str,
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.object_name_did_set(main, object_id, old_name)?;
        }
        Ok(())
    }

    fn object_origin_did_set(
        &mut self,
        main: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_origin: TyVector3I32,
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.object_origin_did_set(main, object_id, old_origin)?;
        }
        Ok(())
    }

    fn object_voxels_did_remap(
        &mut self,
        main: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_bounds: TyVector3U32,
        voxel_ids: &HashMap<U32Id<BVoxVoxel>, U32Id<BVoxVoxel>>,
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.object_voxels_did_remap(main, object_id, old_bounds, voxel_ids)?;
        }
        Ok(())
    }

    fn object_voxels_did_resample(
        &mut self,
        main: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_bounds: TyVector3U32,
        old_voxel_ids: &[U32Id<BVoxVoxel>],
    ) -> Result<()> {
        for ext in &mut self.exts {
            ext.object_voxels_did_resample(main, object_id, old_bounds, old_voxel_ids)?;
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

#[cfg(test)]
mod tests {
    use crate::{
        ext::{VoxconvExt, VoxconvVoxMain, box_ext},
        voxj::ext::CompositeVoxExt,
    };
    use branded_id::U32Id;
    use std::collections::HashMap;
    use ty_math::{TyTransformF64, TyVector3I32, TyVector3U32};
    use voxcore::{
        BVoxHierarchyNode, BVoxObject, BVoxVoxel, Result, VoxExt, VoxHierarchyNode, VoxMain,
        VoxObject, VoxState,
    };

    /// Records the name of each setter hook it sees.
    #[derive(Clone, Debug, Default)]
    struct Recorder(Vec<&'static str>);

    impl VoxExt for Recorder {
        fn hierarchy_node_name_did_set(
            &mut self,
            _state: &VoxState,
            _node_id: U32Id<BVoxHierarchyNode>,
            _old_name: &str,
        ) -> Result<()> {
            self.0.push("node name");
            Ok(())
        }

        fn hierarchy_node_transform_did_set(
            &mut self,
            _state: &VoxState,
            _node_id: U32Id<BVoxHierarchyNode>,
            _old_transform: TyTransformF64,
        ) -> Result<()> {
            self.0.push("node transform");
            Ok(())
        }

        fn hierarchy_node_children_did_set(
            &mut self,
            _state: &VoxState,
            _node_id: U32Id<BVoxHierarchyNode>,
            _old_child_node_ids: &[U32Id<BVoxHierarchyNode>],
            _old_child_object_ids: &[U32Id<BVoxObject>],
        ) -> Result<()> {
            self.0.push("node children");
            Ok(())
        }

        fn root_hierarchy_node_ids_did_set(
            &mut self,
            _state: &VoxState,
            _old_root_ids: &[U32Id<BVoxHierarchyNode>],
        ) -> Result<()> {
            self.0.push("roots");
            Ok(())
        }

        fn object_did_move(
            &mut self,
            _state: &VoxState,
            _object_id: U32Id<BVoxObject>,
            _old_index: usize,
        ) -> Result<()> {
            self.0.push("object move");
            Ok(())
        }

        fn object_name_did_set(
            &mut self,
            _state: &VoxState,
            _object_id: U32Id<BVoxObject>,
            _old_name: &str,
        ) -> Result<()> {
            self.0.push("object name");
            Ok(())
        }

        fn object_origin_did_set(
            &mut self,
            _state: &VoxState,
            _object_id: U32Id<BVoxObject>,
            _old_origin: TyVector3I32,
        ) -> Result<()> {
            self.0.push("object origin");
            Ok(())
        }

        fn object_voxels_did_remap(
            &mut self,
            _state: &VoxState,
            _object_id: U32Id<BVoxObject>,
            _old_bounds: TyVector3U32,
            _voxel_ids: &HashMap<U32Id<BVoxVoxel>, U32Id<BVoxVoxel>>,
        ) -> Result<()> {
            self.0.push("object voxels");
            Ok(())
        }

        fn object_voxels_did_resample(
            &mut self,
            _state: &VoxState,
            _object_id: U32Id<BVoxObject>,
            _old_bounds: TyVector3U32,
            _old_voxel_ids: &[U32Id<BVoxVoxel>],
        ) -> Result<()> {
            self.0.push("object resample");
            Ok(())
        }
    }

    /// Every setter hook reaches every entry through the boxed composite.
    #[test]
    fn forwards_every_setter_hook_to_every_entry() {
        let composite = CompositeVoxExt {
            exts: vec![Box::new(Recorder::default()), Box::new(Recorder::default())],
        };

        let mut main: VoxconvVoxMain = box_ext(VoxMain::default().put_ext(composite));

        let object = VoxObject::new(String::new(), TyVector3U32::splat(1)).unwrap();

        let object_id = main.retain_object(object).unwrap();

        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();

        main.set_hierarchy_node_name(node_id, "n".to_owned())
            .unwrap();

        main.set_hierarchy_node_transform(node_id, TyTransformF64::default())
            .unwrap();

        main.set_hierarchy_node_children(node_id, Vec::new(), vec![object_id])
            .unwrap();

        main.set_root_hierarchy_node_ids(vec![node_id]).unwrap();

        main.move_object(object_id, 0).unwrap();

        main.set_object_name(object_id, "o".to_owned()).unwrap();

        main.set_object_origin(object_id, TyVector3I32::ONE)
            .unwrap();

        main.remap_object_voxels(object_id, TyVector3U32::splat(2), |p| p.as_ivec3())
            .unwrap();

        main.resample_object_voxels(object_id, TyVector3U32::splat(1), Some)
            .unwrap();

        let composite: &dyn VoxconvExt = main.ext().as_ref();

        let composite = composite.downcast_ref::<CompositeVoxExt>().unwrap();

        for ext in &composite.exts {
            assert_eq!(
                ext.downcast_ref::<Recorder>().unwrap().0,
                [
                    "node name",
                    "node transform",
                    "node children",
                    "roots",
                    "object move",
                    "object name",
                    "object origin",
                    "object voxels",
                    "object resample",
                ]
            );
        }
    }
}
