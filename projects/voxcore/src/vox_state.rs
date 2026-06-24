use crate::{
    BVoxHierarchyNode, BVoxObject, BVoxPalette, BVoxPaletteCell, VoxError, VoxGcRemap,
    VoxHierarchyNode, VoxObject, VoxPalette, VoxResult, VoxValue,
};
use branded_id::{
    U32Id,
    soa::{IdField, IdStruct},
};
use std::collections::HashMap;

/// The in-memory state of a voxel model: its objects, shared palettes, scene
/// hierarchy, and roots.
///
/// Add entities with [`add_object`](Self::add_object),
/// [`add_palette`](Self::add_palette), and
/// [`add_hierarchy_node`](Self::add_hierarchy_node), and read them back by id or
/// through the `iter_*` methods. Ids are bare indices into this state, meaningful
/// only within it. [`validate`](Self::validate) checks the cross-references.
#[derive(Debug, Default)]
pub struct VoxState {
    /// Object id pool.
    object_ids: IdStruct<BVoxObject>,

    /// The objects.
    objects: IdField<BVoxObject, VoxObject>,

    /// Palette id pool.
    palette_ids: IdStruct<BVoxPalette>,

    /// The shared palettes.
    palettes: IdField<BVoxPalette, VoxPalette>,

    /// Hierarchy node id pool.
    hierarchy_node_ids: IdStruct<BVoxHierarchyNode>,

    /// The hierarchy nodes.
    hierarchy_nodes: IdField<BVoxHierarchyNode, VoxHierarchyNode>,

    /// The scene's roots: hierarchy node ids.
    root_hierarchy_nodes: Vec<U32Id<BVoxHierarchyNode>>,

    /// Optional user-extension namespace; the core format assigns it no meaning.
    ext: Option<VoxValue>,
}

impl VoxState {
    /// Adds an object, returning its id (its listing index).
    pub fn add_object(&mut self, object: VoxObject) -> U32Id<BVoxObject> {
        let id = self.object_ids.retain();
        self.objects.retain(id, object);
        id
    }

    /// Number of objects.
    pub fn object_count(&self) -> usize {
        self.object_ids.len()
    }

    /// The object `id`, or `None` if not one of this state's.
    pub fn object(&self, id: U32Id<BVoxObject>) -> Option<&VoxObject> {
        // Safety: retained ids have a value.
        self.object_ids
            .is_retained(id)
            .then(|| unsafe { self.objects.get(id) })
    }

    /// Objects in id order, as `(id, object)`.
    pub fn iter_objects(&self) -> impl Iterator<Item = (U32Id<BVoxObject>, &VoxObject)> + '_ {
        // Safety: retained ids have a value.
        self.object_ids
            .iter()
            .map(move |id| (id, unsafe { self.objects.get(id) }))
    }

    /// Adds a shared palette, returning its id (its listing index).
    pub fn add_palette(&mut self, palette: VoxPalette) -> U32Id<BVoxPalette> {
        let id = self.palette_ids.retain();
        self.palettes.retain(id, palette);
        id
    }

    /// Number of shared palettes.
    pub fn palette_count(&self) -> usize {
        self.palette_ids.len()
    }

    /// The palette `id`, or `None` if not one of this state's.
    pub fn palette(&self, id: U32Id<BVoxPalette>) -> Option<&VoxPalette> {
        // Safety: retained ids have a value.
        self.palette_ids
            .is_retained(id)
            .then(|| unsafe { self.palettes.get(id) })
    }

    /// Palettes in id order, as `(id, palette)`.
    pub fn iter_palettes(&self) -> impl Iterator<Item = (U32Id<BVoxPalette>, &VoxPalette)> + '_ {
        // Safety: retained ids have a value.
        self.palette_ids
            .iter()
            .map(move |id| (id, unsafe { self.palettes.get(id) }))
    }

    /// Adds a hierarchy node, returning its id (its listing index). Its
    /// references are checked by [`validate`](Self::validate), not here.
    pub fn add_hierarchy_node(&mut self, node: VoxHierarchyNode) -> U32Id<BVoxHierarchyNode> {
        let id = self.hierarchy_node_ids.retain();
        self.hierarchy_nodes.retain(id, node);
        id
    }

    /// Number of hierarchy nodes.
    pub fn hierarchy_node_count(&self) -> usize {
        self.hierarchy_node_ids.len()
    }

    /// The hierarchy node `id`, or `None` if not one of this state's.
    pub fn hierarchy_node(&self, id: U32Id<BVoxHierarchyNode>) -> Option<&VoxHierarchyNode> {
        // Safety: retained ids have a value.
        self.hierarchy_node_ids
            .is_retained(id)
            .then(|| unsafe { self.hierarchy_nodes.get(id) })
    }

    /// Hierarchy nodes in id order, as `(id, node)`.
    pub fn iter_hierarchy_nodes(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxHierarchyNode>, &VoxHierarchyNode)> + '_ {
        // Safety: retained ids have a value.
        self.hierarchy_node_ids
            .iter()
            .map(move |id| (id, unsafe { self.hierarchy_nodes.get(id) }))
    }

    /// The scene's roots: hierarchy node ids.
    pub fn root_hierarchy_nodes(&self) -> &[U32Id<BVoxHierarchyNode>] {
        &self.root_hierarchy_nodes
    }

    /// Replaces the scene's roots. Checked by [`validate`](Self::validate), not
    /// here.
    pub fn set_root_hierarchy_nodes(&mut self, roots: Vec<U32Id<BVoxHierarchyNode>>) {
        self.root_hierarchy_nodes = roots;
    }

    /// Appends a root.
    pub fn push_root_hierarchy_node(&mut self, root: U32Id<BVoxHierarchyNode>) {
        self.root_hierarchy_nodes.push(root);
    }

    /// The user-extension value, or `None` if unset.
    pub fn ext(&self) -> Option<&VoxValue> {
        self.ext.as_ref()
    }

    /// Sets or clears the user-extension value.
    pub fn set_ext(&mut self, ext: Option<VoxValue>) {
        self.ext = ext;
    }

    /// Removes object `id`, detaching it from every node's `child_objects`.
    /// `None`, changing nothing, if `id` is not one of this state's objects.
    /// Leaves a hole until [`gc`](Self::gc) renumbers for a deterministic save.
    pub fn remove_object(&mut self, id: U32Id<BVoxObject>) -> Option<()> {
        if !self.object_ids.is_retained(id) {
            return None;
        }
        let node_ids: Vec<_> = self.hierarchy_node_ids.iter().collect();
        for node_id in node_ids {
            // Safety: retained node ids have a value.
            let node = unsafe { self.hierarchy_nodes.get_mut(node_id) };
            node.child_objects.retain(|&object| object != id);
        }
        // Safety: a retained object id has a value.
        unsafe { self.objects.release(id) };
        self.object_ids.release(id);
        Some(())
    }

    /// Removes palette `id`, detaching every object reference to it (along with
    /// that reference's per-voxel sample column). `None`, changing nothing, if
    /// `id` is not one of this state's palettes. Leaves a hole until
    /// [`gc`](Self::gc) renumbers.
    pub fn remove_palette(&mut self, id: U32Id<BVoxPalette>) -> Option<()> {
        if !self.palette_ids.is_retained(id) {
            return None;
        }
        let object_ids: Vec<_> = self.object_ids.iter().collect();
        for object_id in object_ids {
            // Safety: retained object ids have a value.
            let object = unsafe { self.objects.get_mut(object_id) };
            object.remove_palette_refs_to(id);
        }
        // Safety: a retained palette id has a value; its Drop frees its cells.
        unsafe { self.palettes.release(id) };
        self.palette_ids.release(id);
        Some(())
    }

    /// Removes hierarchy node `id`, detaching it from every `child_nodes` list and
    /// from the roots. Its own children keep any other parents (the hierarchy is a
    /// DAG). `None`, changing nothing, if `id` is not one of this state's nodes.
    /// Leaves a hole until [`gc`](Self::gc) renumbers.
    pub fn remove_hierarchy_node(&mut self, id: U32Id<BVoxHierarchyNode>) -> Option<()> {
        if !self.hierarchy_node_ids.is_retained(id) {
            return None;
        }
        let node_ids: Vec<_> = self.hierarchy_node_ids.iter().collect();
        for node_id in node_ids {
            // Safety: retained node ids have a value.
            let node = unsafe { self.hierarchy_nodes.get_mut(node_id) };
            node.child_nodes.retain(|&child| child != id);
        }
        self.root_hierarchy_nodes.retain(|&root| root != id);
        // Safety: a retained node id has a value.
        unsafe { self.hierarchy_nodes.release(id) };
        self.hierarchy_node_ids.release(id);
        Some(())
    }

    /// Removes `cell` from `palette`, first repainting every live voxel that
    /// samples it onto `replacement` so no voxel is left without a material.
    /// `None`, changing nothing, if `palette` is not one of this state's palettes,
    /// if `cell` or `replacement` is not one of that palette's cells, or if
    /// `replacement` is `cell` itself. Leaves a hole until [`gc`](Self::gc)
    /// renumbers.
    pub fn remove_cell(
        &mut self,
        palette: U32Id<BVoxPalette>,
        cell: U32Id<BVoxPaletteCell>,
        replacement: U32Id<BVoxPaletteCell>,
    ) -> Option<()> {
        if !self.palette_ids.is_retained(palette) || cell == replacement {
            return None;
        }
        // Safety: the palette id is retained.
        let palette_ref = unsafe { self.palettes.get(palette) };
        if !palette_ref.contains_cell(cell) || !palette_ref.contains_cell(replacement) {
            return None;
        }

        let object_ids: Vec<_> = self.object_ids.iter().collect();
        for object_id in object_ids {
            // Safety: retained object ids have a value.
            let object = unsafe { self.objects.get_mut(object_id) };
            object.repaint_cell(palette, cell, replacement);
        }

        // Safety: the palette id is retained; the cell is one of its cells.
        unsafe { self.palettes.get_mut(palette) }.remove_cell(cell);
        Some(())
    }

    /// Compacts every id pool back to a contiguous `0..len` and rewrites every
    /// cross-reference to match, so a state edited by removals numbers its
    /// entities the way a freshly loaded one does and voxsmith saves stay
    /// deterministic (id equals listing index). Call this after any removal and
    /// before saving.
    ///
    /// Requires a referentially valid state, which the `remove_*` methods preserve
    /// (each detaches what it removes) and [`validate`](Self::validate) checks.
    /// The voxel grids are dense and never compacted, so voxel ids keep equaling
    /// their raster index.
    ///
    /// Returns the [`VoxGcRemap`] recording where each id moved, so any ids held
    /// outside the state can be translated to their compacted values.
    pub fn gc(&mut self) -> VoxGcRemap {
        // Compact each palette's own attribute and cell pools first, keeping the
        // cell relabelings so object samples can be translated below.
        let palette_ids: Vec<_> = self.palette_ids.iter().collect();
        let mut cell_remaps = HashMap::with_capacity(palette_ids.len());
        for palette_id in palette_ids {
            // Safety: retained palette ids have a value.
            let cell_remap = unsafe { self.palettes.get_mut(palette_id) }.gc();
            cell_remaps.insert(palette_id, cell_remap);
        }

        // Compact the palette pool.
        let palette_remap = self.palette_ids.gc();
        // Safety: the palette column was in sync with the pre-gc palette pool, and
        // nothing has retained or released since.
        unsafe { self.palettes.gc(&palette_remap) };

        // Rewrite each object's palette references and sample cells, then compact
        // its own reference pool.
        let object_ids: Vec<_> = self.object_ids.iter().collect();
        for object_id in object_ids {
            // Safety: retained object ids have a value.
            unsafe { self.objects.get_mut(object_id) }.gc(&palette_remap, &cell_remaps);
        }

        // Compact the object pool.
        let object_remap = self.object_ids.gc();
        // Safety: the object column was in sync with the pre-gc object pool, and
        // nothing has retained or released since.
        unsafe { self.objects.gc(&object_remap) };

        // Compact the node pool, then translate child links and roots, which point
        // at the relabeled nodes and objects.
        let node_remap = self.hierarchy_node_ids.gc();
        // Safety: the node column was in sync with the pre-gc node pool, and
        // nothing has retained or released since.
        unsafe { self.hierarchy_nodes.gc(&node_remap) };

        let node_ids: Vec<_> = self.hierarchy_node_ids.iter().collect();
        for node_id in node_ids {
            // Safety: retained node ids have a value.
            let node = unsafe { self.hierarchy_nodes.get_mut(node_id) };
            for child in &mut node.child_nodes {
                *child = node_remap
                    .new_id(*child)
                    .expect("a child node is live in a valid state");
            }
            for object in &mut node.child_objects {
                *object = object_remap
                    .new_id(*object)
                    .expect("a child object is live in a valid state");
            }
        }

        for root in &mut self.root_hierarchy_nodes {
            *root = node_remap
                .new_id(*root)
                .expect("a root is live in a valid state");
        }

        VoxGcRemap {
            objects: object_remap,
            palettes: palette_remap,
            hierarchy_nodes: node_remap,
            cells: cell_remaps,
        }
    }

    /// Checks that every cross-reference resolves and the hierarchy is acyclic:
    /// object palette refs, live-voxel sample cells, node children, and roots,
    /// plus a cycle check over `child_nodes`. Shared and duplicated children are
    /// allowed (the hierarchy is a DAG); only dangling refs and cycles fail.
    pub fn validate(&self) -> VoxResult<()> {
        // Object palette refs (INV-13) and live-voxel sample cells (INV-14).
        // Checks are by id retention, not index range, so they hold whether or
        // not removals have left the pools with holes.
        for (object_id, object) in self.iter_objects() {
            let mut ref_palettes = Vec::with_capacity(object.palette_ref_count());
            for (palette_ref_id, palette_id) in object.iter_palette_refs() {
                let palette = self.palette(palette_id).ok_or(VoxError::PaletteRef {
                    object: object_id.to_u32(),
                    palette: palette_id.to_u32(),
                })?;
                ref_palettes.push((palette_ref_id, palette));
            }
            for voxel_id in object.iter_live() {
                for &(palette_ref_id, palette) in &ref_palettes {
                    let cell = object
                        .voxel_cell(voxel_id, palette_ref_id)
                        .expect("live voxel has a sample for every reference");
                    if !palette.contains_cell(cell) {
                        return Err(VoxError::SampleCell {
                            object: object_id.to_u32(),
                            voxel: voxel_id.to_u32(),
                            cell: cell.to_u32(),
                        });
                    }
                }
            }
        }

        // Node children (INV-18); retention-checked before the cycle pass.
        for (node_id, node) in self.iter_hierarchy_nodes() {
            for &child in &node.child_nodes {
                if self.hierarchy_node(child).is_none() {
                    return Err(VoxError::ChildNode {
                        node: node_id.to_u32(),
                        child: child.to_u32(),
                    });
                }
            }
            for &object in &node.child_objects {
                if self.object(object).is_none() {
                    return Err(VoxError::ChildObject {
                        node: node_id.to_u32(),
                        object: object.to_u32(),
                    });
                }
            }
        }

        // Roots (INV-17).
        for &root in &self.root_hierarchy_nodes {
            if self.hierarchy_node(root).is_none() {
                return Err(VoxError::Root {
                    root: root.to_u32(),
                });
            }
        }

        // Acyclicity (INV-19); every child is now known live.
        if let Some(node) = self.first_cycle_node() {
            return Err(VoxError::Cycle { node });
        }

        Ok(())
    }

    /// A node on a `child_nodes` cycle, or `None` if acyclic. Iterative
    /// three-colour DFS (so a deep chain can't overflow the stack): a back edge
    /// into an in-progress node is a cycle, revisiting a finished node is not.
    /// Call only after every child id is known live. Works over the retained node
    /// ids by position, so it holds whether or not the pool has holes.
    fn first_cycle_node(&self) -> Option<u32> {
        const WHITE: u8 = 0;
        const GREY: u8 = 1;
        const BLACK: u8 = 2;

        // Retained node ids and a lookup from id to its position here, so a holed
        // pool (ids not contiguous from zero) is handled the same as a packed one.
        let node_ids: Vec<_> = self.hierarchy_node_ids.iter().collect();
        let index_of: HashMap<u32, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(index, id)| (id.to_u32(), index))
            .collect();
        let count = node_ids.len();
        let mut colour = vec![WHITE; count];

        for start in 0..count {
            if colour[start] != WHITE {
                continue;
            }
            colour[start] = GREY;
            // Each frame is a node position plus how many children we have walked.
            let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
            while let Some(&(node, cursor)) = stack.last() {
                let node_id = node_ids[node];
                let next_child = {
                    // Safety: `node_id` is a retained node id.
                    let children = &unsafe { self.hierarchy_nodes.get(node_id) }.child_nodes;
                    (cursor < children.len()).then(|| children[cursor].to_u32())
                };
                match next_child {
                    Some(child) => {
                        stack.last_mut().unwrap().1 += 1;
                        // Children are retention-checked before this pass runs.
                        let child = index_of[&child];
                        match colour[child] {
                            WHITE => {
                                colour[child] = GREY;
                                stack.push((child, 0));
                            }
                            GREY => return Some(node_ids[child].to_u32()),
                            _ => {}
                        }
                    }
                    None => {
                        colour[node] = BLACK;
                        stack.pop();
                    }
                }
            }
        }

        None
    }

    /// Deep copy, rebuilding every column against fresh id pools since the SoA
    /// types can't derive `Clone`.
    pub fn clone_state(&self) -> Self {
        let mut objects = IdField::new();
        for id in self.object_ids.iter() {
            // Safety: retained ids have a value.
            objects.retain(id, unsafe { self.objects.get(id) }.clone_object());
        }

        let mut palettes = IdField::new();
        for id in self.palette_ids.iter() {
            // Safety: retained ids have a value.
            palettes.retain(id, unsafe { self.palettes.get(id) }.clone_palette());
        }

        let mut hierarchy_nodes = IdField::new();
        for id in self.hierarchy_node_ids.iter() {
            // Safety: retained ids have a value.
            hierarchy_nodes.retain(id, unsafe { self.hierarchy_nodes.get(id) }.clone());
        }

        Self {
            object_ids: self.object_ids.clone(),
            objects,
            palette_ids: self.palette_ids.clone(),
            palettes,
            hierarchy_node_ids: self.hierarchy_node_ids.clone(),
            hierarchy_nodes,
            root_hierarchy_nodes: self.root_hierarchy_nodes.clone(),
            ext: self.ext.clone(),
        }
    }
}

impl Drop for VoxState {
    fn drop(&mut self) {
        // Safety: each column holds a value for every id in its pool; the fields
        // free their own storage on drop.
        unsafe {
            self.objects.release_all(&self.object_ids);
            self.palettes.release_all(&self.palette_ids);
            self.hierarchy_nodes.release_all(&self.hierarchy_node_ids);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BVoxAttribute, BVoxHierarchyNode, BVoxObject, BVoxPalette, BVoxPaletteCell, BVoxPaletteRef,
        VoxError, VoxHierarchyNode, VoxObject, VoxPalette, VoxState, VoxValue,
    };
    use branded_id::U32Id;
    use ty_math::TyVector3U32;

    fn node_id(index: u32) -> U32Id<BVoxHierarchyNode> {
        U32Id::from_u32(index)
    }

    fn cell_id(index: u32) -> U32Id<BVoxPaletteCell> {
        U32Id::from_u32(index)
    }

    /// A node referencing the given child nodes (and no objects).
    fn node_with_children(child_nodes: Vec<U32Id<BVoxHierarchyNode>>) -> VoxHierarchyNode {
        VoxHierarchyNode {
            child_nodes,
            ..VoxHierarchyNode::default()
        }
    }

    /// A node placing the given child objects (and no child nodes).
    fn node_with_objects(child_objects: Vec<U32Id<BVoxObject>>) -> VoxHierarchyNode {
        VoxHierarchyNode {
            child_objects,
            ..VoxHierarchyNode::default()
        }
    }

    fn unit_object(name: &str) -> VoxObject {
        VoxObject::new(name.to_owned(), TyVector3U32::new(1, 1, 1)).unwrap()
    }

    /// A palette with one attribute and one cell whose value is `value`.
    fn one_cell_palette(value: f64) -> VoxPalette {
        let mut palette = VoxPalette::default();
        palette.add_attribute("v".to_owned());
        palette.add_cell(vec![VoxValue::Number(value)]).unwrap();
        palette
    }

    #[test]
    fn add_and_read_back_in_id_order() {
        let mut state = VoxState::default();
        let a = state.add_object(unit_object("a"));
        let b = state.add_object(unit_object("b"));

        assert_eq!(state.object_count(), 2);
        assert_eq!(state.object(a).unwrap().name(), "a");
        let names: Vec<&str> = state.iter_objects().map(|(_, o)| o.name()).collect();
        assert_eq!(names, ["a", "b"]);
        assert_eq!(b.to_u32(), 1);
    }

    #[test]
    fn validate_accepts_a_shared_and_duplicated_child_dag() {
        let mut state = VoxState::default();
        let leaf = state.add_hierarchy_node(VoxHierarchyNode::default());
        // Sharing and duplicate children are legal in a DAG.
        let a = state.add_hierarchy_node(node_with_children(vec![leaf, leaf]));
        let b = state.add_hierarchy_node(node_with_children(vec![leaf]));
        state.set_root_hierarchy_nodes(vec![a, b]);

        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn validate_rejects_dangling_palette_ref() {
        let mut state = VoxState::default();
        let mut object = unit_object("o");
        // Reference palette id 0, but the state has no palettes.
        object.add_palette_ref(
            U32Id::<BVoxPalette>::from_u32(0),
            U32Id::<BVoxPaletteCell>::from_u32(0),
        );
        let id = state.add_object(object);

        assert_eq!(
            state.validate(),
            Err(VoxError::PaletteRef {
                object: id.to_u32(),
                palette: 0,
            })
        );
    }

    #[test]
    fn validate_rejects_dangling_child() {
        let mut state = VoxState::default();
        state.add_hierarchy_node(node_with_children(vec![node_id(9)]));
        assert!(matches!(
            state.validate(),
            Err(VoxError::ChildNode { child: 9, .. })
        ));
    }

    #[test]
    fn validate_rejects_dangling_root() {
        let mut state = VoxState::default();
        state.add_hierarchy_node(VoxHierarchyNode::default());
        state.set_root_hierarchy_nodes(vec![node_id(7)]);
        assert_eq!(state.validate(), Err(VoxError::Root { root: 7 }));
    }

    #[test]
    fn validate_rejects_a_cycle() {
        let mut state = VoxState::default();
        // node 0 -> child 1, node 1 -> child 0.
        state.add_hierarchy_node(node_with_children(vec![node_id(1)]));
        state.add_hierarchy_node(node_with_children(vec![node_id(0)]));
        assert!(matches!(state.validate(), Err(VoxError::Cycle { .. })));
    }

    #[test]
    fn clone_state_is_an_independent_deep_copy() {
        let mut state = VoxState::default();
        state.add_palette(VoxPalette::default());
        state.add_object(unit_object("o"));

        let copy = state.clone_state();
        assert_eq!(copy.palette_count(), 1);
        assert_eq!(copy.object_count(), 1);

        state.add_object(unit_object("p"));
        assert_eq!(state.object_count(), 2);
        assert_eq!(copy.object_count(), 1);
    }

    #[test]
    fn remove_object_and_palette_then_gc_renumbers_and_resolves() {
        let mut state = VoxState::default();
        let palette_a = state.add_palette(one_cell_palette(0.0));
        let palette_b = state.add_palette(one_cell_palette(1.0));

        let mut a = unit_object("a");
        a.add_palette_ref(palette_a, cell_id(0));
        let object_a = state.add_object(a);

        let mut b = unit_object("b");
        b.add_palette_ref(palette_b, cell_id(0));
        let voxel = b.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        b.retain_voxel(voxel, &[cell_id(0)]).unwrap();
        let object_b = state.add_object(b);

        let inner = state.add_hierarchy_node(node_with_objects(vec![object_a, object_b]));
        let outer = state.add_hierarchy_node(node_with_children(vec![inner]));
        state.set_root_hierarchy_nodes(vec![outer]);
        assert_eq!(state.validate(), Ok(()));

        // Remove object `a` and palette A; the state stays clean (no dangling
        // refs) even before gc, just with holes.
        assert_eq!(state.remove_object(object_a), Some(()));
        assert_eq!(state.remove_palette(palette_a), Some(()));
        assert_eq!(state.validate(), Ok(()));
        assert_eq!(state.object_count(), 1);
        assert_eq!(state.palette_count(), 1);

        let remap = state.gc();
        assert_eq!(state.validate(), Ok(()));

        // The survivors renumber to 0 and their cross-references follow.
        let object = U32Id::<BVoxObject>::from_u32(0);
        let palette = U32Id::<BVoxPalette>::from_u32(0);
        assert_eq!(state.object(object).unwrap().name(), "b");
        assert_eq!(
            state
                .palette(palette)
                .unwrap()
                .cell_value(cell_id(0), U32Id::<BVoxAttribute>::from_u32(0)),
            Some(&VoxValue::Number(1.0))
        );
        assert_eq!(
            state
                .object(object)
                .unwrap()
                .iter_palette_refs()
                .collect::<Vec<_>>(),
            [(U32Id::<BVoxPaletteRef>::from_u32(0), palette)]
        );
        assert_eq!(
            state
                .object(object)
                .unwrap()
                .voxel_cell(voxel, U32Id::<BVoxPaletteRef>::from_u32(0)),
            Some(cell_id(0))
        );

        // The inner node dropped `a` and renumbered `b` to 0; the roots are intact.
        let inner = U32Id::<BVoxHierarchyNode>::from_u32(0);
        assert_eq!(
            state.hierarchy_node(inner).unwrap().child_objects,
            [U32Id::<BVoxObject>::from_u32(0)]
        );
        assert_eq!(
            state.root_hierarchy_nodes(),
            [U32Id::<BVoxHierarchyNode>::from_u32(1)]
        );

        // The returned remap translates the same renumbering for held ids:
        // removed entities map to None, survivors to their compacted ids.
        assert_eq!(remap.objects.new_id(object_a), None);
        assert_eq!(remap.objects.new_id(object_b), Some(object));
        assert_eq!(remap.palettes.new_id(palette_a), None);
        assert_eq!(remap.palettes.new_id(palette_b), Some(palette));
        assert!(!remap.cells.contains_key(&palette_a));
        assert_eq!(remap.cells[&palette_b].new_id(cell_id(0)), Some(cell_id(0)));
    }

    #[test]
    fn remove_hierarchy_node_detaches_children_and_roots() {
        let mut state = VoxState::default();
        let leaf = state.add_hierarchy_node(VoxHierarchyNode::default());
        let mid = state.add_hierarchy_node(node_with_children(vec![leaf]));
        let top = state.add_hierarchy_node(node_with_children(vec![mid, leaf]));
        state.set_root_hierarchy_nodes(vec![top, mid]);

        assert_eq!(state.remove_hierarchy_node(mid), Some(()));
        assert_eq!(state.remove_hierarchy_node(mid), None); // already gone

        // `mid` is detached from `top` and the roots; the shared `leaf` survives.
        assert_eq!(state.hierarchy_node(top).unwrap().child_nodes, [leaf]);
        assert_eq!(state.root_hierarchy_nodes(), [top]);
        assert!(state.hierarchy_node(mid).is_none());
        assert!(state.hierarchy_node(leaf).is_some());
        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn remove_cell_repaints_live_voxels_onto_the_replacement() {
        let mut state = VoxState::default();
        let mut palette = VoxPalette::default();
        palette.add_attribute("v".to_owned());
        let keep = palette.add_cell(vec![VoxValue::Number(0.0)]).unwrap();
        let drop = palette.add_cell(vec![VoxValue::Number(1.0)]).unwrap();
        let palette = state.add_palette(palette);

        let mut object = unit_object("o");
        let reference = object.add_palette_ref(palette, keep);
        let voxel = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel, &[drop]).unwrap();
        let object = state.add_object(object);

        // Removing `drop` repaints the voxel that used it onto `keep`.
        assert_eq!(state.remove_cell(palette, drop, keep), Some(()));
        assert_eq!(state.validate(), Ok(()));
        assert_eq!(
            state.object(object).unwrap().voxel_cell(voxel, reference),
            Some(keep)
        );
        assert!(!state.palette(palette).unwrap().contains_cell(drop));

        // A no-op replacement and unknown ids are rejected.
        assert_eq!(state.remove_cell(palette, keep, keep), None);
        assert_eq!(state.remove_cell(palette, drop, keep), None); // drop is gone

        state.gc();
        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn validate_and_gc_handle_a_high_id_sample_after_a_cell_hole() {
        let mut state = VoxState::default();
        let mut palette = VoxPalette::default();
        palette.add_attribute("v".to_owned());
        let first = palette.add_cell(vec![VoxValue::Number(0.0)]).unwrap();
        let second = palette.add_cell(vec![VoxValue::Number(1.0)]).unwrap();
        let third = palette.add_cell(vec![VoxValue::Number(2.0)]).unwrap();
        let palette = state.add_palette(palette);

        let mut object = unit_object("o");
        let reference = object.add_palette_ref(palette, first);
        let voxel = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel, &[third]).unwrap(); // samples the highest id
        let object = state.add_object(object);

        // Remove `first`; no live voxel used it, so the repaint is a no-op. The
        // palette is now holed: the voxel still samples `third`, whose id exceeds
        // the live cell count. A range check would wrongly reject this; the
        // retention check accepts it.
        assert_eq!(state.remove_cell(palette, first, second), Some(()));
        assert_eq!(state.validate(), Ok(()));

        state.gc();
        assert_eq!(state.validate(), Ok(()));
        // gc preserves which cell the voxel samples: still the value-2.0 cell,
        // just renumbered.
        let sampled = state
            .object(object)
            .unwrap()
            .voxel_cell(voxel, reference)
            .unwrap();
        assert_eq!(
            state
                .palette(palette)
                .unwrap()
                .cell_value(sampled, U32Id::<BVoxAttribute>::from_u32(0)),
            Some(&VoxValue::Number(2.0))
        );
        assert_eq!(state.palette(palette).unwrap().cell_count(), 2);
    }

    #[test]
    fn remove_object_rejects_an_unknown_id() {
        let mut state = VoxState::default();
        let object = state.add_object(unit_object("o"));
        assert_eq!(state.remove_object(object), Some(()));
        assert_eq!(state.remove_object(object), None);
        assert_eq!(
            state.remove_palette(U32Id::<BVoxPalette>::from_u32(0)),
            None
        );
    }
}
