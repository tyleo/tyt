use crate::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxVoxel, Result, VoxGcRemap,
    VoxState,
};
use branded_id::U32Id;
use std::collections::HashMap;

/// The ext a [`VoxMain`](crate::VoxMain) carries: the hooks the state fires
/// when an entity comes, goes, or is renumbered.
///
/// A format ext keeps an entry per entity, keyed by the entity's id. A retain
/// fires its hook after the mutation, a release fires its hook before it once
/// every check has passed, and [`gc`](crate::VoxMain::gc) fires
/// [`did_gc`](Self::did_gc) with the remap after renumbering. The `did` or
/// `will` in a hook's name says which. Each hook sees the scene and can
/// build a complete entry for the entity on the spot.
///
/// A hook can refuse. A `will` hook's error leaves the state unchanged. A
/// `did` hook's error reports an ext that could not follow a mutation that
/// already happened: the scene stands and the ext is out of step. Every hook
/// defaults to `Ok(())`. A type that follows nothing implements the trait
/// with an empty body, and `()` is that ext for a bare main.
///
/// How an ext persists, and how a boxed one downcasts or clones, is not part
/// of being an ext. That belongs to the crate that converts between formats.
pub trait VoxExt {
    /// Hierarchy node `node_id` was retained.
    fn hierarchy_node_did_retain(
        &mut self,
        _state: &VoxState,
        _node_id: U32Id<BVoxHierarchyNode>,
    ) -> Result<()> {
        Ok(())
    }

    /// Hierarchy node `node_id` is about to be released.
    fn hierarchy_node_will_release(
        &mut self,
        _state: &VoxState,
        _node_id: U32Id<BVoxHierarchyNode>,
    ) -> Result<()> {
        Ok(())
    }

    /// Object `object_id` was retained.
    fn object_did_retain(
        &mut self,
        _state: &VoxState,
        _object_id: U32Id<BVoxObject>,
    ) -> Result<()> {
        Ok(())
    }

    /// Object `object_id` is about to be released.
    fn object_will_release(
        &mut self,
        _state: &VoxState,
        _object_id: U32Id<BVoxObject>,
    ) -> Result<()> {
        Ok(())
    }

    /// Palette `palette_id` was retained.
    fn palette_did_retain(
        &mut self,
        _state: &VoxState,
        _palette_id: U32Id<BVoxPalette>,
    ) -> Result<()> {
        Ok(())
    }

    /// Palette `palette_id` is about to be released.
    fn palette_will_release(
        &mut self,
        _state: &VoxState,
        _palette_id: U32Id<BVoxPalette>,
    ) -> Result<()> {
        Ok(())
    }

    /// Material `material_id` was retained to palette `palette_id`.
    fn material_did_retain(
        &mut self,
        _state: &VoxState,
        _palette_id: U32Id<BVoxPalette>,
        _material_id: U32Id<BVoxMaterial>,
    ) -> Result<()> {
        Ok(())
    }

    /// The materials `material_ids`, in listing order, are about to be
    /// released from palette `palette_id`.
    fn materials_will_release(
        &mut self,
        _state: &VoxState,
        _palette_id: U32Id<BVoxPalette>,
        _material_ids: &[U32Id<BVoxMaterial>],
    ) -> Result<()> {
        Ok(())
    }

    /// The samples of palette `palette_id` were repainted in one pass: each
    /// `replacement_ids` entry moved the samples of its key onto its value.
    /// The materials themselves did not move.
    fn materials_did_repaint(
        &mut self,
        _state: &VoxState,
        _palette_id: U32Id<BVoxPalette>,
        _replacement_ids: &HashMap<U32Id<BVoxMaterial>, U32Id<BVoxMaterial>>,
    ) -> Result<()> {
        Ok(())
    }

    /// Voxel `voxel_id` of object `object_id` was retained. A repaint of a
    /// live voxel fires this too.
    fn voxel_did_retain(
        &mut self,
        _state: &VoxState,
        _object_id: U32Id<BVoxObject>,
        _voxel_id: U32Id<BVoxVoxel>,
    ) -> Result<()> {
        Ok(())
    }

    /// Voxel `voxel_id` of object `object_id` is about to be released.
    fn voxel_will_release(
        &mut self,
        _state: &VoxState,
        _object_id: U32Id<BVoxObject>,
        _voxel_id: U32Id<BVoxVoxel>,
    ) -> Result<()> {
        Ok(())
    }

    /// Every id pool was renumbered by `remap`. An ext keyed by id rekeys its
    /// entries here.
    fn did_gc(&mut self, _state: &VoxState, _remap: &VoxGcRemap) -> Result<()> {
        Ok(())
    }
}

/// No ext. Ignores every hook.
impl VoxExt for () {}

#[cfg(test)]
mod tests {
    use crate::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxVoxel, Error, Result, VoxExt,
        VoxGcRemap, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxState, VoxValuePool,
    };
    use branded_id::U32Id;
    use std::collections::{HashMap, HashSet};
    use ty_math::TyVector3U32;

    /// Records every hook the main fires, with what it could read.
    #[derive(Clone, Debug, Default, PartialEq)]
    struct Recorder(Vec<String>);

    impl VoxExt for Recorder {
        fn hierarchy_node_did_retain(
            &mut self,
            state: &VoxState,
            node_id: U32Id<BVoxHierarchyNode>,
        ) -> Result<()> {
            let count = state.hierarchy_node_count();
            self.0
                .push(format!("node retained {} of {count}", node_id.to_u32()));
            Ok(())
        }

        fn hierarchy_node_will_release(
            &mut self,
            state: &VoxState,
            node_id: U32Id<BVoxHierarchyNode>,
        ) -> Result<()> {
            let count = state.hierarchy_node_count();
            self.0
                .push(format!("node released {} of {count}", node_id.to_u32()));
            Ok(())
        }

        fn object_did_retain(
            &mut self,
            state: &VoxState,
            object_id: U32Id<BVoxObject>,
        ) -> Result<()> {
            let name = state.object(object_id).unwrap().name();
            self.0
                .push(format!("object retained {} {name}", object_id.to_u32()));
            Ok(())
        }

        fn object_will_release(
            &mut self,
            state: &VoxState,
            object_id: U32Id<BVoxObject>,
        ) -> Result<()> {
            let name = state.object(object_id).unwrap().name();
            self.0
                .push(format!("object released {} {name}", object_id.to_u32()));
            Ok(())
        }

        fn palette_did_retain(
            &mut self,
            _state: &VoxState,
            palette_id: U32Id<BVoxPalette>,
        ) -> Result<()> {
            self.0
                .push(format!("palette retained {}", palette_id.to_u32()));
            Ok(())
        }

        fn palette_will_release(
            &mut self,
            _state: &VoxState,
            palette_id: U32Id<BVoxPalette>,
        ) -> Result<()> {
            self.0
                .push(format!("palette released {}", palette_id.to_u32()));
            Ok(())
        }

        fn material_did_retain(
            &mut self,
            _state: &VoxState,
            palette_id: U32Id<BVoxPalette>,
            material_id: U32Id<BVoxMaterial>,
        ) -> Result<()> {
            self.0.push(format!(
                "material retained {} {}",
                palette_id.to_u32(),
                material_id.to_u32()
            ));
            Ok(())
        }

        fn materials_will_release(
            &mut self,
            _state: &VoxState,
            palette_id: U32Id<BVoxPalette>,
            material_ids: &[U32Id<BVoxMaterial>],
        ) -> Result<()> {
            let ids: Vec<u32> = material_ids.iter().map(|id| id.to_u32()).collect();
            self.0.push(format!(
                "materials released {} {ids:?}",
                palette_id.to_u32()
            ));
            Ok(())
        }

        fn materials_did_repaint(
            &mut self,
            _state: &VoxState,
            palette_id: U32Id<BVoxPalette>,
            replacement_ids: &HashMap<U32Id<BVoxMaterial>, U32Id<BVoxMaterial>>,
        ) -> Result<()> {
            let mut pairs: Vec<(u32, u32)> = replacement_ids
                .iter()
                .map(|(from, to)| (from.to_u32(), to.to_u32()))
                .collect();
            pairs.sort_unstable();
            self.0.push(format!(
                "materials repainted {} {pairs:?}",
                palette_id.to_u32()
            ));
            Ok(())
        }

        fn voxel_did_retain(
            &mut self,
            _state: &VoxState,
            object_id: U32Id<BVoxObject>,
            voxel_id: U32Id<BVoxVoxel>,
        ) -> Result<()> {
            self.0.push(format!(
                "voxel retained {} {}",
                object_id.to_u32(),
                voxel_id.to_u32()
            ));
            Ok(())
        }

        fn voxel_will_release(
            &mut self,
            _state: &VoxState,
            object_id: U32Id<BVoxObject>,
            voxel_id: U32Id<BVoxVoxel>,
        ) -> Result<()> {
            self.0.push(format!(
                "voxel released {} {}",
                object_id.to_u32(),
                voxel_id.to_u32()
            ));
            Ok(())
        }

        fn did_gc(&mut self, _state: &VoxState, remap: &VoxGcRemap) -> Result<()> {
            let objects: Vec<Option<u32>> = (0..remap.objects.old_len() as u32)
                .map(|old| {
                    remap
                        .objects
                        .new_id(U32Id::from_u32(old))
                        .map(|id| id.to_u32())
                })
                .collect();
            self.0.push(format!("gc objects {objects:?}"));
            Ok(())
        }
    }

    /// Refuses every release.
    #[derive(Debug, Default)]
    struct Refuser;

    impl VoxExt for Refuser {
        fn object_will_release(
            &mut self,
            _state: &VoxState,
            object_id: U32Id<BVoxObject>,
        ) -> Result<()> {
            Err(Error::Ext {
                reason: format!("object {} is pinned", object_id.to_u32()),
            })
        }
    }

    fn events(main: &VoxMain<Recorder>) -> Vec<&str> {
        main.ext().0.iter().map(String::as_str).collect()
    }

    fn unit_object(name: &str) -> VoxObject {
        VoxObject::new(name.to_owned(), TyVector3U32::new(1, 1, 1)).unwrap()
    }

    #[test]
    fn objects_fire_with_their_id_and_the_scene() {
        let mut main: VoxMain<Recorder> = VoxMain::default();

        main.retain_object(unit_object("a")).unwrap();

        let b_id = main.retain_object(unit_object("b")).unwrap();

        main.retain_object(unit_object("c")).unwrap();

        main.release_object(b_id).unwrap();

        // The reused id lands at the end of the listing. A move fires nothing
        // because ids do not change.
        let d_id = main.retain_object(unit_object("d")).unwrap();

        main.move_object(d_id, 0).unwrap();

        assert_eq!(
            events(&main),
            [
                "object retained 0 a",
                "object retained 1 b",
                "object retained 2 c",
                "object released 1 b",
                "object retained 1 d",
            ]
        );
    }

    #[test]
    fn palettes_fire_with_their_id() {
        let mut main: VoxMain<Recorder> = VoxMain::default();

        let a_id = main.retain_palette(VoxPalette::default()).unwrap();

        let b_id = main.retain_palette(VoxPalette::default()).unwrap();

        main.release_palette(a_id).unwrap();

        main.retain_palette(VoxPalette::default()).unwrap();

        main.move_palette(b_id, 1).unwrap();

        assert_eq!(
            events(&main),
            [
                "palette retained 0",
                "palette retained 1",
                "palette released 0",
                "palette retained 0",
            ]
        );
    }

    #[test]
    fn hierarchy_nodes_fire_with_their_id_after_the_retain() {
        let mut main: VoxMain<Recorder> = VoxMain::default();

        main.retain_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();

        let b_id = main
            .retain_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();

        main.retain_hierarchy_nodes(vec![
            VoxHierarchyNode::default(),
            VoxHierarchyNode::default(),
        ])
        .unwrap();

        main.release_hierarchy_node(b_id).unwrap();

        // A batch fires after the whole batch is in, and a release fires
        // while the node is still there.
        assert_eq!(
            events(&main),
            [
                "node retained 0 of 1",
                "node retained 1 of 2",
                "node retained 2 of 4",
                "node retained 3 of 4",
                "node released 1 of 4",
            ]
        );
    }

    #[test]
    fn materials_fire_within_their_palette() {
        let mut main: VoxMain<Recorder> = VoxMain::default();

        main.retain_palette(VoxPalette::default()).unwrap();

        let value_pool_id = main.retain_value_pool(VoxValuePool::int(vec![1, 2, 3]).unwrap());

        let mut palette = VoxPalette::default();

        palette
            .retain_property("v".to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();

        let m0_id = palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();

        let m1_id = palette.retain_material(vec![U32Id::from_u32(1)]).unwrap();

        let m2_id = palette.retain_material(vec![U32Id::from_u32(2)]).unwrap();

        let palette_id = main.retain_palette(palette).unwrap();

        main.retain_material(palette_id, vec![U32Id::from_u32(2)])
            .unwrap();

        main.repaint_materials(palette_id, &HashMap::from([(m0_id, m1_id)]))
            .unwrap();

        main.release_materials(palette_id, &HashSet::from([m0_id, m2_id]))
            .unwrap();

        assert_eq!(
            events(&main),
            [
                "palette retained 0",
                "palette retained 1",
                "material retained 1 3",
                "materials repainted 1 [(0, 1)]",
                "materials released 1 [0, 2]",
            ]
        );
    }

    #[test]
    fn voxels_fire_with_the_object_id() {
        let mut main: VoxMain<Recorder> = VoxMain::default();

        main.retain_object(unit_object("a")).unwrap();

        let object_id = main.retain_object(unit_object("b")).unwrap();

        let voxel_id = main
            .object(object_id)
            .unwrap()
            .voxel_id(TyVector3U32::new(0, 0, 0))
            .unwrap();

        main.retain_voxel(object_id, voxel_id, &[]).unwrap();

        main.release_voxel(object_id, voxel_id).unwrap();

        assert_eq!(
            events(&main)[2..],
            ["voxel retained 1 0", "voxel released 1 0"]
        );
    }

    #[test]
    fn gc_fires_with_the_remap() {
        let mut main: VoxMain<Recorder> = VoxMain::default();

        let a_id = main.retain_object(unit_object("a")).unwrap();

        main.retain_object(unit_object("b")).unwrap();

        main.release_object(a_id).unwrap();

        main.gc().unwrap();

        assert_eq!(events(&main)[3..], ["gc objects [None, Some(0)]"]);
    }

    #[test]
    fn a_rejected_mutation_fires_nothing() {
        let mut main: VoxMain<Recorder> = VoxMain::default();

        assert!(main.release_object(U32Id::from_u32(9)).is_err());

        assert!(main.move_palette(U32Id::from_u32(0), 0).is_err());

        assert!(events(&main).is_empty());
    }

    #[test]
    fn a_refused_release_changes_nothing() {
        let mut main: VoxMain<Refuser> = VoxMain::default();

        let object_id = main.retain_object(unit_object("a")).unwrap();

        assert_eq!(
            main.release_object(object_id),
            Err(Error::Ext {
                reason: "object 0 is pinned".to_owned()
            })
        );

        assert!(main.object(object_id).is_some());
    }
}
