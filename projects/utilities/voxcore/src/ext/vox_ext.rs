use crate::{BVoxVoxel, VoxMap, ext::Result};
use branded_id::U32Id;
use std::{any::Any, fmt::Debug};

/// The ext a [`VoxMain`](crate::VoxMain) carries: the whole ext block a
/// document persists, plus the hooks the state fires when a listing moves.
///
/// A format ext aligns its entries with the scene by listing index. A retain
/// or move fires its hook after the mutation, and a release fires its hook
/// before it, once every check has passed. The `did` or `will` in a hook's
/// name says which. Each carries the index the entity had, and the ext drops
/// or inserts its entry in step. A batch release fires once with every index,
/// descending, so an ext can remove entries in place. Every hook defaults to
/// a no-op.
///
/// [`to_vox_ext`](Self::to_vox_ext) returns the block. An empty map means no
/// block, and a writer omits it. The trait is object-safe, so a state can
/// carry `Box<dyn VoxExt>` when the format is chosen at run time.
///
/// Impls: `()` encodes no block and ignores every hook. [`VoxMap`] encodes
/// itself verbatim and cannot follow a hook. `Option<E>` forwards to `Some`
/// and encodes no block for `None`. `Box<dyn VoxExt>` forwards.
pub trait VoxExt: Any + Debug {
    /// Encodes the ext as a document's ext block. Empty means no block.
    fn to_vox_ext(&self) -> Result<VoxMap>;

    /// The ext as `Any`, so a boxed ext can downcast to its format's type.
    fn as_any(&self) -> &dyn Any;

    /// Clones the ext into a box.
    fn clone_box(&self) -> Box<dyn VoxExt>;

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

/// No ext. Encodes no block and ignores every hook.
impl VoxExt for () {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        Ok(VoxMap::default())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn VoxExt> {
        Box::new(())
    }
}

/// An optional ext. `Some` forwards everything to the ext. `None` encodes no
/// block and ignores every hook.
impl<E: VoxExt + Clone> VoxExt for Option<E> {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        match self {
            Some(ext) => ext.to_vox_ext(),
            None => Ok(VoxMap::default()),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn VoxExt> {
        Box::new(self.clone())
    }

    fn hierarchy_node_did_retain(&mut self, index: usize) {
        if let Some(ext) = self {
            ext.hierarchy_node_did_retain(index);
        }
    }

    fn hierarchy_node_will_release(&mut self, index: usize) {
        if let Some(ext) = self {
            ext.hierarchy_node_will_release(index);
        }
    }

    fn object_did_retain(&mut self, index: usize) {
        if let Some(ext) = self {
            ext.object_did_retain(index);
        }
    }

    fn object_will_release(&mut self, index: usize) {
        if let Some(ext) = self {
            ext.object_will_release(index);
        }
    }

    fn object_did_move(&mut self, from: usize, to: usize) {
        if let Some(ext) = self {
            ext.object_did_move(from, to);
        }
    }

    fn palette_did_retain(&mut self, index: usize) {
        if let Some(ext) = self {
            ext.palette_did_retain(index);
        }
    }

    fn palette_will_release(&mut self, index: usize) {
        if let Some(ext) = self {
            ext.palette_will_release(index);
        }
    }

    fn palette_did_move(&mut self, from: usize, to: usize) {
        if let Some(ext) = self {
            ext.palette_did_move(from, to);
        }
    }

    fn material_did_retain(&mut self, palette: usize, index: usize) {
        if let Some(ext) = self {
            ext.material_did_retain(palette, index);
        }
    }

    fn materials_will_release(&mut self, palette: usize, indices: &[usize]) {
        if let Some(ext) = self {
            ext.materials_will_release(palette, indices);
        }
    }

    fn materials_did_repaint(&mut self, palette: usize, remap: &[(usize, usize)]) {
        if let Some(ext) = self {
            ext.materials_did_repaint(palette, remap);
        }
    }

    fn voxel_did_retain(&mut self, object: usize, voxel: U32Id<BVoxVoxel>) {
        if let Some(ext) = self {
            ext.voxel_did_retain(object, voxel);
        }
    }

    fn voxel_will_release(&mut self, object: usize, voxel: U32Id<BVoxVoxel>) {
        if let Some(ext) = self {
            ext.voxel_will_release(object, voxel);
        }
    }
}

/// A boxed ext. Everything forwards to the ext inside, so a downcast through
/// [`as_any`](VoxExt::as_any) reaches the format's type.
impl VoxExt for Box<dyn VoxExt> {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        (**self).to_vox_ext()
    }

    fn as_any(&self) -> &dyn Any {
        (**self).as_any()
    }

    fn clone_box(&self) -> Box<dyn VoxExt> {
        (**self).clone_box()
    }

    fn hierarchy_node_did_retain(&mut self, index: usize) {
        (**self).hierarchy_node_did_retain(index);
    }

    fn hierarchy_node_will_release(&mut self, index: usize) {
        (**self).hierarchy_node_will_release(index);
    }

    fn object_did_retain(&mut self, index: usize) {
        (**self).object_did_retain(index);
    }

    fn object_will_release(&mut self, index: usize) {
        (**self).object_will_release(index);
    }

    fn object_did_move(&mut self, from: usize, to: usize) {
        (**self).object_did_move(from, to);
    }

    fn palette_did_retain(&mut self, index: usize) {
        (**self).palette_did_retain(index);
    }

    fn palette_will_release(&mut self, index: usize) {
        (**self).palette_will_release(index);
    }

    fn palette_did_move(&mut self, from: usize, to: usize) {
        (**self).palette_did_move(from, to);
    }

    fn material_did_retain(&mut self, palette: usize, index: usize) {
        (**self).material_did_retain(palette, index);
    }

    fn materials_will_release(&mut self, palette: usize, indices: &[usize]) {
        (**self).materials_will_release(palette, indices);
    }

    fn materials_did_repaint(&mut self, palette: usize, remap: &[(usize, usize)]) {
        (**self).materials_did_repaint(palette, remap);
    }

    fn voxel_did_retain(&mut self, object: usize, voxel: U32Id<BVoxVoxel>) {
        (**self).voxel_did_retain(object, voxel);
    }

    fn voxel_will_release(&mut self, object: usize, voxel: U32Id<BVoxVoxel>) {
        (**self).voxel_will_release(object, voxel);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BVoxVoxel, VoxHierarchyNode, VoxMain, VoxMap, VoxObject, VoxPalette, VoxValue,
        VoxValuePool,
        ext::{Result, VoxExt},
    };
    use branded_id::U32Id;
    use std::{
        any::Any,
        collections::{HashMap, HashSet},
    };
    use ty_math::TyVector3U32;

    /// Records every hook the state fires.
    #[derive(Clone, Debug, Default, PartialEq)]
    struct Recorder(Vec<String>);

    impl VoxExt for Recorder {
        fn to_vox_ext(&self) -> Result<VoxMap> {
            Ok(VoxMap(vec![(
                "events".to_owned(),
                VoxValue::Number(self.0.len() as f64),
            )]))
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn clone_box(&self) -> Box<dyn VoxExt> {
            Box::new(self.clone())
        }

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

    #[test]
    fn an_optional_ext_forwards_to_some_and_none_ignores() {
        let mut state: VoxMain<Option<Recorder>> = VoxMain::default();

        state.retain_object(unit_object("a")).unwrap();

        assert_eq!(state.ext(), &None);

        assert_eq!(state.ext().to_vox_ext().unwrap(), VoxMap::default());

        state.set_ext(Some(Recorder::default()));

        state.retain_object(unit_object("b")).unwrap();

        assert_eq!(state.ext().as_ref().unwrap().0, ["object retained 1"]);
    }

    #[test]
    fn a_boxed_ext_forwards_and_downcasts() {
        let mut state: VoxMain<Box<dyn VoxExt>> =
            VoxMain::default().map_ext(|()| Box::new(Recorder::default()) as Box<dyn VoxExt>);

        state.retain_object(unit_object("a")).unwrap();

        let recorder = state
            .ext()
            .as_any()
            .downcast_ref::<Recorder>()
            .expect("the box holds the recorder");

        assert_eq!(recorder.0, ["object retained 0"]);

        assert_eq!(
            state.ext().to_vox_ext().unwrap(),
            VoxMap(vec![("events".to_owned(), VoxValue::Number(1.0))])
        );

        let cloned = state.ext().clone_box();

        assert_eq!(cloned.as_any().downcast_ref::<Recorder>(), Some(recorder));
    }

    #[test]
    fn the_unit_ext_encodes_no_block_and_a_map_encodes_itself() {
        assert_eq!(().to_vox_ext().unwrap(), VoxMap::default());

        let block = VoxMap(vec![("any".to_owned(), VoxValue::Null)]);

        assert_eq!(block.to_vox_ext().unwrap(), block);

        assert_eq!(Some(block.clone()).to_vox_ext().unwrap(), block);

        assert_eq!(None::<VoxMap>.to_vox_ext().unwrap(), VoxMap::default());
    }
}
