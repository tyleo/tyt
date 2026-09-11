use crate::BVoxVoxel;
use branded_id::U32Id;

/// The ext a [`VoxMain`](crate::VoxMain) carries: the hooks the state fires
/// when a listing moves.
///
/// A format ext aligns its entries with the scene by listing index. A retain
/// or move fires its hook after the mutation, and a release fires its hook
/// before it, once every check has passed. The `did` or `will` in a hook's
/// name says which. Each carries the index the entity had, and the ext drops
/// or inserts its entry in step. A batch release fires once with every index,
/// descending, so an ext can remove entries in place. Every hook defaults to
/// a no-op, so a type that follows nothing implements the trait with an empty
/// body. `()` is that ext for a bare state.
///
/// An ext keeps a list in step with plain inserts and removes. A hook whose
/// index does not reach the list panics on the list's bounds, because the
/// ext was out of step before the mutation.
///
/// How an ext persists, and how a boxed one downcasts or clones, is not part
/// of being an ext. That belongs to the crate that converts between formats.
pub trait VoxExt {
    /// A hierarchy node was retained at listing `index`.
    fn hierarchy_node_did_retain(&mut self, _index: usize) {}

    /// The hierarchy node at listing `index` is about to be released.
    fn hierarchy_node_will_release(&mut self, _index: usize) {}

    /// An object was retained at listing `index`.
    fn object_did_retain(&mut self, _index: usize) {}

    /// The object at listing `index` is about to be released.
    fn object_will_release(&mut self, _index: usize) {}

    /// The object at listing `from` moved to `to`.
    fn object_did_move(&mut self, _from: usize, _to: usize) {}

    /// A palette was retained at listing `index`.
    fn palette_did_retain(&mut self, _index: usize) {}

    /// The palette at listing `index` is about to be released.
    fn palette_will_release(&mut self, _index: usize) {}

    /// The palette at listing `from` moved to `to`.
    fn palette_did_move(&mut self, _from: usize, _to: usize) {}

    /// A material was retained at `index` in the palette at listing
    /// `palette`.
    fn material_did_retain(&mut self, _palette: usize, _index: usize) {}

    /// The materials at `indices`, descending, are about to be released from
    /// the palette at listing `palette`.
    fn materials_will_release(&mut self, _palette: usize, _indices: &[usize]) {}

    /// The samples of the palette at listing `palette` were repainted in one
    /// pass: each `(from, to)` pair moved the samples of material `from` onto
    /// material `to`. The listing itself did not move.
    fn materials_did_repaint(&mut self, _palette: usize, _remap: &[(usize, usize)]) {}

    /// Voxel `voxel` of the object at listing `object` was retained.
    fn voxel_did_retain(&mut self, _object: usize, _voxel: U32Id<BVoxVoxel>) {}

    /// Voxel `voxel` of the object at listing `object` is about to be
    /// released.
    fn voxel_will_release(&mut self, _object: usize, _voxel: U32Id<BVoxVoxel>) {}
}

/// No ext. Ignores every hook.
impl VoxExt for () {}

#[cfg(test)]
mod tests {
    use crate::{
        BVoxVoxel, VoxExt, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValuePool,
    };
    use branded_id::U32Id;
    use std::collections::{HashMap, HashSet};
    use ty_math::TyVector3U32;

    /// Records every hook the state fires.
    #[derive(Clone, Debug, Default, PartialEq)]
    struct Recorder(Vec<String>);

    impl VoxExt for Recorder {
        fn hierarchy_node_did_retain(&mut self, index: usize) {
            self.0.push(format!("node retained {index}"));
        }

        fn hierarchy_node_will_release(&mut self, index: usize) {
            self.0.push(format!("node released {index}"));
        }

        fn object_did_retain(&mut self, index: usize) {
            self.0.push(format!("object retained {index}"));
        }

        fn object_will_release(&mut self, index: usize) {
            self.0.push(format!("object released {index}"));
        }

        fn object_did_move(&mut self, from: usize, to: usize) {
            self.0.push(format!("object moved {from} {to}"));
        }

        fn palette_did_retain(&mut self, index: usize) {
            self.0.push(format!("palette retained {index}"));
        }

        fn palette_will_release(&mut self, index: usize) {
            self.0.push(format!("palette released {index}"));
        }

        fn palette_did_move(&mut self, from: usize, to: usize) {
            self.0.push(format!("palette moved {from} {to}"));
        }

        fn material_did_retain(&mut self, palette: usize, index: usize) {
            self.0.push(format!("material retained {palette} {index}"));
        }

        fn materials_will_release(&mut self, palette: usize, indices: &[usize]) {
            self.0
                .push(format!("materials released {palette} {indices:?}"));
        }

        fn materials_did_repaint(&mut self, palette: usize, remap: &[(usize, usize)]) {
            self.0
                .push(format!("materials repainted {palette} {remap:?}"));
        }

        fn voxel_did_retain(&mut self, object: usize, voxel: U32Id<BVoxVoxel>) {
            self.0
                .push(format!("voxel retained {object} {}", voxel.to_u32()));
        }

        fn voxel_will_release(&mut self, object: usize, voxel: U32Id<BVoxVoxel>) {
            self.0
                .push(format!("voxel released {object} {}", voxel.to_u32()));
        }
    }

    fn events(state: &VoxMain<Recorder>) -> Vec<&str> {
        state.ext().0.iter().map(String::as_str).collect()
    }

    fn unit_object(name: &str) -> VoxObject {
        VoxObject::new(name.to_owned(), TyVector3U32::new(1, 1, 1)).unwrap()
    }

    #[test]
    fn objects_fire_with_their_listing_index() {
        let mut state: VoxMain<Recorder> = VoxMain::default();

        state.retain_object(unit_object("a")).unwrap();

        let b_id = state.retain_object(unit_object("b")).unwrap();

        state.retain_object(unit_object("c")).unwrap();

        state.release_object(b_id).unwrap();

        // The reused id lands at the end of the listing.
        let d_id = state.retain_object(unit_object("d")).unwrap();

        state.move_object(d_id, 0).unwrap();

        assert_eq!(
            events(&state),
            [
                "object retained 0",
                "object retained 1",
                "object retained 2",
                "object released 1",
                "object retained 2",
                "object moved 2 0",
            ]
        );
    }

    #[test]
    fn palettes_fire_with_their_listing_index() {
        let mut state: VoxMain<Recorder> = VoxMain::default();

        let a_id = state.retain_palette(VoxPalette::default()).unwrap();

        let b_id = state.retain_palette(VoxPalette::default()).unwrap();

        state.release_palette(a_id).unwrap();

        state.retain_palette(VoxPalette::default()).unwrap();

        state.move_palette(b_id, 1).unwrap();

        assert_eq!(
            events(&state),
            [
                "palette retained 0",
                "palette retained 1",
                "palette released 0",
                "palette retained 1",
                "palette moved 0 1",
            ]
        );
    }

    #[test]
    fn hierarchy_nodes_fire_with_their_listing_index() {
        let mut state: VoxMain<Recorder> = VoxMain::default();

        state
            .retain_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();

        let b_id = state
            .retain_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();

        state
            .retain_hierarchy_nodes(vec![
                VoxHierarchyNode::default(),
                VoxHierarchyNode::default(),
            ])
            .unwrap();

        state.release_hierarchy_node(b_id).unwrap();

        assert_eq!(
            events(&state),
            [
                "node retained 0",
                "node retained 1",
                "node retained 2",
                "node retained 3",
                "node released 1",
            ]
        );
    }

    #[test]
    fn materials_fire_within_their_palette() {
        let mut state: VoxMain<Recorder> = VoxMain::default();

        state.retain_palette(VoxPalette::default()).unwrap();

        let value_pool_id = state.retain_value_pool(VoxValuePool::int(vec![1, 2, 3]).unwrap());

        let mut palette = VoxPalette::default();

        palette
            .retain_property("v".to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();

        let m0_id = palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();

        let m1_id = palette.retain_material(vec![U32Id::from_u32(1)]).unwrap();

        let m2_id = palette.retain_material(vec![U32Id::from_u32(2)]).unwrap();

        let palette_id = state.retain_palette(palette).unwrap();

        state
            .retain_material(palette_id, vec![U32Id::from_u32(2)])
            .unwrap();

        state
            .repaint_materials(palette_id, &HashMap::from([(m0_id, m1_id)]))
            .unwrap();

        state
            .release_materials(palette_id, &HashSet::from([m0_id, m2_id]))
            .unwrap();

        assert_eq!(
            events(&state),
            [
                "palette retained 0",
                "palette retained 1",
                "material retained 1 3",
                "materials repainted 1 [(0, 1)]",
                "materials released 1 [2, 0]",
            ]
        );
    }

    #[test]
    fn voxels_fire_with_the_object_index() {
        let mut state: VoxMain<Recorder> = VoxMain::default();

        state.retain_object(unit_object("a")).unwrap();

        let object_id = state.retain_object(unit_object("b")).unwrap();

        let voxel_id = state
            .object(object_id)
            .unwrap()
            .voxel_id(TyVector3U32::new(0, 0, 0))
            .unwrap();

        state.retain_voxel(object_id, voxel_id, &[]).unwrap();

        state.release_voxel(object_id, voxel_id).unwrap();

        assert_eq!(
            events(&state)[2..],
            ["voxel retained 1 0", "voxel released 1 0"]
        );
    }

    #[test]
    fn a_rejected_mutation_fires_nothing() {
        let mut state: VoxMain<Recorder> = VoxMain::default();

        assert!(state.release_object(U32Id::from_u32(9)).is_err());

        assert!(state.move_palette(U32Id::from_u32(0), 0).is_err());

        assert!(events(&state).is_empty());
    }
}
