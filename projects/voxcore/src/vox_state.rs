use crate::{
    BVoxHierarchyNode, BVoxObject, BVoxPalette, VoxError, VoxHierarchyNode, VoxObject, VoxPalette,
    VoxResult, VoxValue,
};
use branded_id::{
    U32Id,
    soa::{IdField, IdStruct},
};

/// The in-memory state of a voxel model: its objects, shared palettes, scene
/// hierarchy, and roots.
///
/// Add entities with [`add_object`](Self::add_object),
/// [`add_palette`](Self::add_palette), and
/// [`add_hierarchy_node`](Self::add_hierarchy_node), and read them back by id or
/// through the `iter_*` methods. Ids are bare indices into this state, meaningful
/// only within it. [`validate`](Self::validate) checks the cross-references.
///
/// Fields are private because the columns must stay in lockstep with their id
/// pools.
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

    /// Checks that every cross-reference resolves and the hierarchy is acyclic:
    /// object palette refs, live-voxel sample cells, node children, and roots,
    /// plus a cycle check over `child_nodes`. Shared and duplicated children are
    /// allowed (the hierarchy is a DAG); only dangling refs and cycles fail.
    pub fn validate(&self) -> VoxResult<()> {
        let object_count = self.object_ids.len();
        let node_count = self.hierarchy_node_ids.len();

        // Object palette refs (INV-13) and live-voxel sample cells (INV-14).
        for (object_id, object) in self.iter_objects() {
            let mut ref_cell_counts = Vec::with_capacity(object.palette_ref_count());
            for (palette_ref_id, palette_id) in object.iter_palette_refs() {
                let palette = self.palette(palette_id).ok_or(VoxError::PaletteRef {
                    object: object_id.to_u32(),
                    palette: palette_id.to_u32(),
                })?;
                ref_cell_counts.push((palette_ref_id, palette.cell_count()));
            }
            for voxel_id in object.iter_live() {
                for &(palette_ref_id, cell_count) in &ref_cell_counts {
                    let cell = object
                        .voxel_cell(voxel_id, palette_ref_id)
                        .expect("live voxel has a sample for every reference");
                    if (cell.to_u32() as usize) >= cell_count {
                        return Err(VoxError::SampleCell {
                            object: object_id.to_u32(),
                            voxel: voxel_id.to_u32(),
                            cell: cell.to_u32(),
                        });
                    }
                }
            }
        }

        // Node children (INV-18); range-check before the cycle pass indexes nodes.
        for (node_id, node) in self.iter_hierarchy_nodes() {
            for &child in &node.child_nodes {
                if (child.to_u32() as usize) >= node_count {
                    return Err(VoxError::ChildNode {
                        node: node_id.to_u32(),
                        child: child.to_u32(),
                    });
                }
            }
            for &object in &node.child_objects {
                if (object.to_u32() as usize) >= object_count {
                    return Err(VoxError::ChildObject {
                        node: node_id.to_u32(),
                        object: object.to_u32(),
                    });
                }
            }
        }

        // Roots (INV-17).
        for &root in &self.root_hierarchy_nodes {
            if (root.to_u32() as usize) >= node_count {
                return Err(VoxError::Root {
                    root: root.to_u32(),
                });
            }
        }

        // Acyclicity (INV-19); every child is now known in range.
        if let Some(node) = self.first_cycle_node() {
            return Err(VoxError::Cycle { node });
        }

        Ok(())
    }

    /// A node on a `child_nodes` cycle, or `None` if acyclic. Iterative
    /// three-colour DFS (so a deep chain can't overflow the stack): a back edge
    /// into an in-progress node is a cycle, revisiting a finished node is not.
    /// Call only after every child id is known in range.
    fn first_cycle_node(&self) -> Option<u32> {
        const WHITE: u8 = 0;
        const GREY: u8 = 1;
        const BLACK: u8 = 2;

        let count = self.hierarchy_node_ids.len();
        let mut colour = vec![WHITE; count];

        for start in 0..count {
            if colour[start] != WHITE {
                continue;
            }
            colour[start] = GREY;
            // Each frame is a node plus how many of its children we have walked.
            let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
            while let Some(&(node, cursor)) = stack.last() {
                let node_id = U32Id::<BVoxHierarchyNode>::from_u32(node as u32);
                let next_child = {
                    // Safety: `node` is a retained node id (in `0..count`).
                    let children = &unsafe { self.hierarchy_nodes.get(node_id) }.child_nodes;
                    (cursor < children.len()).then(|| children[cursor].to_u32() as usize)
                };
                match next_child {
                    Some(child) => {
                        stack.last_mut().unwrap().1 += 1;
                        match colour[child] {
                            WHITE => {
                                colour[child] = GREY;
                                stack.push((child, 0));
                            }
                            GREY => return Some(child as u32),
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
        BVoxHierarchyNode, BVoxPalette, BVoxPaletteCell, VoxError, VoxHierarchyNode, VoxObject,
        VoxPalette, VoxState,
    };
    use branded_id::U32Id;
    use ty_math::TyVector3U32;

    fn node_id(index: u32) -> U32Id<BVoxHierarchyNode> {
        U32Id::from_u32(index)
    }

    /// A node referencing the given child nodes (and no objects).
    fn node_with_children(child_nodes: Vec<U32Id<BVoxHierarchyNode>>) -> VoxHierarchyNode {
        VoxHierarchyNode {
            child_nodes,
            ..VoxHierarchyNode::default()
        }
    }

    fn unit_object(name: &str) -> VoxObject {
        VoxObject::new(name.to_owned(), TyVector3U32::new(1, 1, 1)).unwrap()
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
}
