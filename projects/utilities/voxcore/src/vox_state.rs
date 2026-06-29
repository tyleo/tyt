use crate::{
    BVoxHierarchyNode, BVoxObject, BVoxPalette, BVoxPaletteCell, Error, Result, VoxEditObject,
    VoxEditState, VoxGcRemap, VoxHierarchyNode, VoxObject, VoxPalette, VoxRuntimeState, VoxValue,
};
use branded_id::{IdVec, U32Id, soa::IdRemap};
use std::collections::{HashMap, HashSet};

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
    /// The runtime scene: objects, shared palettes, hierarchy, and roots.
    runtime_state: VoxRuntimeState,

    /// The editor state: one edit grid per runtime object.
    edit_state: VoxEditState,

    /// Optional user-extension namespace; the core format assigns it no meaning.
    ext: Option<VoxValue>,
}

impl VoxState {
    /// Adds an object, returning its id (its listing index). Its edit grid is
    /// initialized to the object's runtime grid (a zero-margin edit grid);
    /// override it with [`set_edit_object`](Self::set_edit_object).
    pub fn add_object(&mut self, object: VoxObject) -> U32Id<BVoxObject> {
        let edit_object = VoxEditObject {
            bounds: object.bounds(),
            origin: object.origin(),
        };
        let id = self.runtime_state.object_ids.retain();
        self.runtime_state.objects.retain(id, object);
        self.edit_state.retain(id, edit_object);
        id
    }

    /// Number of objects.
    pub fn object_count(&self) -> usize {
        self.runtime_state.object_ids.len()
    }

    /// The object `id`, or `None` if not one of this state's.
    pub fn object(&self, id: U32Id<BVoxObject>) -> Option<&VoxObject> {
        // Safety: retained ids have a value.
        self.runtime_state
            .object_ids
            .is_retained(id)
            .then(|| unsafe { self.runtime_state.objects.get(id) })
    }

    /// Objects in id order, as `(id, object)`.
    pub fn iter_objects(&self) -> impl Iterator<Item = (U32Id<BVoxObject>, &VoxObject)> + '_ {
        // Safety: retained ids have a value.
        self.runtime_state
            .object_ids
            .iter()
            .map(move |id| (id, unsafe { self.runtime_state.objects.get(id) }))
    }

    /// Adds a shared palette, returning its id (its listing index).
    pub fn add_palette(&mut self, palette: VoxPalette) -> U32Id<BVoxPalette> {
        let id = self.runtime_state.palette_ids.retain();
        self.runtime_state.palettes.retain(id, palette);
        id
    }

    /// Number of shared palettes.
    pub fn palette_count(&self) -> usize {
        self.runtime_state.palette_ids.len()
    }

    /// The palette `id`, or `None` if not one of this state's.
    pub fn palette(&self, id: U32Id<BVoxPalette>) -> Option<&VoxPalette> {
        // Safety: retained ids have a value.
        self.runtime_state
            .palette_ids
            .is_retained(id)
            .then(|| unsafe { self.runtime_state.palettes.get(id) })
    }

    /// Palettes in id order, as `(id, palette)`.
    pub fn iter_palettes(&self) -> impl Iterator<Item = (U32Id<BVoxPalette>, &VoxPalette)> + '_ {
        // Safety: retained ids have a value.
        self.runtime_state
            .palette_ids
            .iter()
            .map(move |id| (id, unsafe { self.runtime_state.palettes.get(id) }))
    }

    /// Adds a hierarchy node, returning its id (its listing index). Its
    /// references are checked by [`validate`](Self::validate), not here.
    pub fn add_hierarchy_node(&mut self, node: VoxHierarchyNode) -> U32Id<BVoxHierarchyNode> {
        let id = self.runtime_state.hierarchy_node_ids.retain();
        self.runtime_state.hierarchy_nodes.retain(id, node);
        id
    }

    /// Number of hierarchy nodes.
    pub fn hierarchy_node_count(&self) -> usize {
        self.runtime_state.hierarchy_node_ids.len()
    }

    /// The hierarchy node `id`, or `None` if not one of this state's.
    pub fn hierarchy_node(&self, id: U32Id<BVoxHierarchyNode>) -> Option<&VoxHierarchyNode> {
        // Safety: retained ids have a value.
        self.runtime_state
            .hierarchy_node_ids
            .is_retained(id)
            .then(|| unsafe { self.runtime_state.hierarchy_nodes.get(id) })
    }

    /// Hierarchy nodes in id order, as `(id, node)`.
    pub fn iter_hierarchy_nodes(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxHierarchyNode>, &VoxHierarchyNode)> + '_ {
        // Safety: retained ids have a value.
        self.runtime_state
            .hierarchy_node_ids
            .iter()
            .map(move |id| (id, unsafe { self.runtime_state.hierarchy_nodes.get(id) }))
    }

    /// The scene's roots: hierarchy node ids.
    pub fn root_hierarchy_nodes(&self) -> &[U32Id<BVoxHierarchyNode>] {
        &self.runtime_state.root_hierarchy_nodes
    }

    /// Replaces the scene's roots. Checked by [`validate`](Self::validate), not
    /// here.
    pub fn set_root_hierarchy_nodes(&mut self, roots: Vec<U32Id<BVoxHierarchyNode>>) {
        self.runtime_state.root_hierarchy_nodes = roots;
    }

    /// Appends a root. Root uniqueness is checked by [`validate`](Self::validate),
    /// not here.
    pub fn push_root_hierarchy_node(&mut self, root: U32Id<BVoxHierarchyNode>) {
        self.runtime_state.root_hierarchy_nodes.push(root);
    }

    /// The user-extension value, or `None` if unset.
    pub fn ext(&self) -> Option<&VoxValue> {
        self.ext.as_ref()
    }

    /// Sets or clears the user-extension value.
    pub fn set_ext(&mut self, ext: Option<VoxValue>) {
        self.ext = ext;
    }

    /// The edit grid for object `id`, or `None` if `id` is not one of this
    /// state's objects.
    pub fn edit_object(&self, id: U32Id<BVoxObject>) -> Option<VoxEditObject> {
        // Safety: a retained object id always has an edit grid.
        self.runtime_state
            .object_ids
            .is_retained(id)
            .then(|| unsafe { self.edit_state.get(id) })
    }

    /// Sets the edit grid for object `id`. `None`, changing nothing, if `id` is
    /// not one of this state's objects. The edit grid is expected to contain the
    /// object's runtime grid; that is checked by [`validate`](Self::validate),
    /// not here.
    pub fn set_edit_object(
        &mut self,
        id: U32Id<BVoxObject>,
        edit_object: VoxEditObject,
    ) -> Option<()> {
        if !self.runtime_state.object_ids.is_retained(id) {
            return None;
        }
        // Safety: the id is retained, so its edit grid is live.
        unsafe { self.edit_state.set(id, edit_object) };
        Some(())
    }

    /// Removes object `id`, detaching it from every node's `child_objects`.
    /// `None`, changing nothing, if `id` is not one of this state's objects.
    /// Leaves a hole until [`gc`](Self::gc) renumbers for a deterministic save.
    pub fn remove_object(&mut self, id: U32Id<BVoxObject>) -> Option<()> {
        if !self.runtime_state.object_ids.is_retained(id) {
            return None;
        }
        let node_ids: Vec<_> = self.runtime_state.hierarchy_node_ids.iter().collect();
        for node_id in node_ids {
            // Safety: retained node ids have a value.
            let node = unsafe { self.runtime_state.hierarchy_nodes.get_mut(node_id) };
            node.child_objects.retain(|&object| object != id);
        }
        // Safety: a retained object id has a value; its edit grid is keyed by the
        // same id, so it releases alongside.
        unsafe { self.runtime_state.objects.release(id) };
        unsafe { self.edit_state.release(id) };
        self.runtime_state.object_ids.release(id);
        Some(())
    }

    /// Removes palette `id`, detaching every object reference to it (along with
    /// that reference's per-voxel sample column). `None`, changing nothing, if
    /// `id` is not one of this state's palettes. Leaves a hole until
    /// [`gc`](Self::gc) renumbers.
    pub fn remove_palette(&mut self, id: U32Id<BVoxPalette>) -> Option<()> {
        if !self.runtime_state.palette_ids.is_retained(id) {
            return None;
        }
        let object_ids: Vec<_> = self.runtime_state.object_ids.iter().collect();
        for object_id in object_ids {
            // Safety: retained object ids have a value.
            let object = unsafe { self.runtime_state.objects.get_mut(object_id) };
            object.remove_palette_refs_to(id);
        }
        // Safety: a retained palette id has a value; its Drop frees its cells.
        unsafe { self.runtime_state.palettes.release(id) };
        self.runtime_state.palette_ids.release(id);
        Some(())
    }

    /// Removes hierarchy node `id`, detaching it from every `child_nodes` list and
    /// from the roots. Its own children keep any other parents (the hierarchy is a
    /// DAG). `None`, changing nothing, if `id` is not one of this state's nodes.
    /// Leaves a hole until [`gc`](Self::gc) renumbers.
    pub fn remove_hierarchy_node(&mut self, id: U32Id<BVoxHierarchyNode>) -> Option<()> {
        if !self.runtime_state.hierarchy_node_ids.is_retained(id) {
            return None;
        }
        let node_ids: Vec<_> = self.runtime_state.hierarchy_node_ids.iter().collect();
        for node_id in node_ids {
            // Safety: retained node ids have a value.
            let node = unsafe { self.runtime_state.hierarchy_nodes.get_mut(node_id) };
            node.child_nodes.retain(|&child| child != id);
        }
        self.runtime_state
            .root_hierarchy_nodes
            .retain(|&root| root != id);
        // Safety: a retained node id has a value.
        unsafe { self.runtime_state.hierarchy_nodes.release(id) };
        self.runtime_state.hierarchy_node_ids.release(id);
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
        if !self.runtime_state.palette_ids.is_retained(palette) || cell == replacement {
            return None;
        }
        // Safety: the palette id is retained.
        let palette_ref = unsafe { self.runtime_state.palettes.get(palette) };
        if !palette_ref.contains_cell(cell) || !palette_ref.contains_cell(replacement) {
            return None;
        }

        let object_ids: Vec<_> = self.runtime_state.object_ids.iter().collect();
        for object_id in object_ids {
            // Safety: retained object ids have a value.
            let object = unsafe { self.runtime_state.objects.get_mut(object_id) };
            object.repaint_cell(palette, cell, replacement);
        }

        // Safety: the palette id is retained; the cell is one of its cells.
        unsafe { self.runtime_state.palettes.get_mut(palette) }.remove_cell(cell);
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
        // Compact each palette's own pools first, so the cell relabelings are
        // ready when object samples are translated below. They are indexed by old
        // palette id, so the column covers the palette pool's whole id space.
        let palette_id_space = self.runtime_state.palette_ids.peek_next_fresh().to_u32() as usize;
        let mut cell_remaps: IdVec<BVoxPalette, IdRemap<BVoxPaletteCell, u32>> =
            IdVec::from_vec((0..palette_id_space).map(|_| IdRemap::default()).collect());
        for palette_id in self.runtime_state.palette_ids.iter().collect::<Vec<_>>() {
            // Safety: retained palette ids have a value.
            let cell_remap = unsafe { self.runtime_state.palettes.get_mut(palette_id) }.gc();
            cell_remaps[palette_id.to_usize_id()] = cell_remap;
        }

        // Compact the palette pool.
        let palette_remap = self.runtime_state.palette_ids.gc();
        // Safety: the palette column was in sync with the pre-gc palette pool, and
        // nothing has retained or released since.
        unsafe { self.runtime_state.palettes.gc(&palette_remap) };

        // Rewrite each object's palette references and sample cells, then compact
        // its own reference pool.
        let object_ids: Vec<_> = self.runtime_state.object_ids.iter().collect();
        for object_id in object_ids {
            // Safety: retained object ids have a value.
            unsafe { self.runtime_state.objects.get_mut(object_id) }
                .gc(&palette_remap, &cell_remaps);
        }

        // Compact the object pool.
        let object_remap = self.runtime_state.object_ids.gc();
        // Safety: the object column was in sync with the pre-gc object pool, and
        // nothing has retained or released since.
        unsafe { self.runtime_state.objects.gc(&object_remap) };
        // The edit column is keyed by object id, so it compacts with the same
        // remap.
        unsafe { self.edit_state.gc(&object_remap) };

        // Compact the node pool, then translate child links and roots, which point
        // at the relabeled nodes and objects.
        let node_remap = self.runtime_state.hierarchy_node_ids.gc();
        // Safety: the node column was in sync with the pre-gc node pool, and
        // nothing has retained or released since.
        unsafe { self.runtime_state.hierarchy_nodes.gc(&node_remap) };

        let node_ids: Vec<_> = self.runtime_state.hierarchy_node_ids.iter().collect();
        for node_id in node_ids {
            // Safety: retained node ids have a value.
            let node = unsafe { self.runtime_state.hierarchy_nodes.get_mut(node_id) };
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

        for root in &mut self.runtime_state.root_hierarchy_nodes {
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

    /// Checks the cross-references and per-entity rules:
    ///
    /// 1. every object palette ref resolves, and no object references the same
    ///    palette twice;
    /// 2. every live-voxel sample cell is within its palette's cells;
    /// 3. every object's grid is exactly tight around its live voxels: an empty
    ///    object is `[0, 0, 0]`, else each axis spans the grid from `0` to its
    ///    bound minus one;
    /// 4. every object's edit grid contains its runtime grid on each axis;
    /// 5. every node child node and child object resolves, and no node lists the
    ///    same one twice;
    /// 6. every root resolves, and no root repeats;
    /// 7. no palette declares the same attribute key twice;
    /// 8. every node transform has a non-zero scale on each axis and a unit-length
    ///    rotation quaternion within `1e-6`;
    /// 9. the `child_nodes` graph is acyclic.
    ///
    /// A node may have several parents, since the hierarchy is a DAG; that sharing
    /// is not a cycle.
    pub fn validate(&self) -> Result<()> {
        // How far a rotation quaternion's length-squared may stray from 1 and
        // still count as a unit quaternion.
        const ROTATION_TOLERANCE: f64 = 1e-6;

        // Palette attribute keys are unique within a palette.
        for (palette_id, palette) in self.iter_palettes() {
            let mut seen_attributes = HashSet::with_capacity(palette.attribute_count());
            for (attribute_id, name) in palette.iter_attributes() {
                if !seen_attributes.insert(name) {
                    return Err(Error::DuplicateAttribute {
                        palette: palette_id.to_u32(),
                        attribute: attribute_id.to_u32(),
                    });
                }
            }
        }

        // Object palette refs and live-voxel sample cells.
        // Checks are by id retention, not index range, so they hold whether or
        // not removals have left the pools with holes.
        for (object_id, object) in self.iter_objects() {
            let mut ref_palettes = Vec::with_capacity(object.palette_ref_count());
            let mut seen_palettes = HashSet::with_capacity(object.palette_ref_count());
            for (palette_ref_id, palette_id) in object.iter_palette_refs() {
                let palette = self.palette(palette_id).ok_or(Error::PaletteRef {
                    object: object_id.to_u32(),
                    palette: palette_id.to_u32(),
                })?;
                if !seen_palettes.insert(palette_id.to_u32()) {
                    return Err(Error::DuplicatePaletteRef {
                        object: object_id.to_u32(),
                        palette: palette_id.to_u32(),
                    });
                }
                ref_palettes.push((palette_ref_id, palette));
            }
            // Live-voxel sample cells, and the tight extent of the live voxels.
            let mut min = [u32::MAX; 3];
            let mut max = [0u32; 3];
            let mut live = false;
            for voxel_id in object.iter_live() {
                live = true;
                let position = object
                    .voxel_position(voxel_id)
                    .expect("a live voxel is within the grid");
                for (axis, coord) in [position.x, position.y, position.z].into_iter().enumerate() {
                    min[axis] = min[axis].min(coord);
                    max[axis] = max[axis].max(coord);
                }
                for &(palette_ref_id, palette) in &ref_palettes {
                    let cell = object
                        .voxel_cell(voxel_id, palette_ref_id)
                        .expect("live voxel has a sample for every reference");
                    if !palette.contains_cell(cell) {
                        return Err(Error::SampleCell {
                            object: object_id.to_u32(),
                            voxel: voxel_id.to_u32(),
                            cell: cell.to_u32(),
                        });
                    }
                }
            }

            // The runtime grid is exactly tight around the live voxels: an empty
            // object is [0, 0, 0]; otherwise each axis spans the grid fully, from 0
            // to its bound minus one.
            let bounds = object.bounds();
            let grid = [bounds.x, bounds.y, bounds.z];
            let tight = if live {
                (0..3).all(|axis| min[axis] == 0 && grid[axis] == max[axis] + 1)
            } else {
                grid == [0, 0, 0]
            };
            if !tight {
                return Err(Error::UntightBounds {
                    object: object_id.to_u32(),
                });
            }

            // The edit grid contains the runtime grid on every axis.
            let edit = self
                .edit_object(object_id)
                .expect("a retained object has an edit grid");
            let origin = object.origin();
            let edit_origin = [edit.origin.x, edit.origin.y, edit.origin.z];
            let edit_bounds = [edit.bounds.x, edit.bounds.y, edit.bounds.z];
            let run_origin = [origin.x, origin.y, origin.z];
            let contains = (0..3).all(|axis| {
                edit_origin[axis] <= run_origin[axis]
                    && i64::from(edit_origin[axis]) + i64::from(edit_bounds[axis])
                        >= i64::from(run_origin[axis]) + i64::from(grid[axis])
            });
            if !contains {
                return Err(Error::EditGridContainment {
                    object: object_id.to_u32(),
                });
            }
        }

        // Node children; retention-checked before the cycle pass.
        for (node_id, node) in self.iter_hierarchy_nodes() {
            let mut seen_child_nodes = HashSet::with_capacity(node.child_nodes.len());
            for &child in &node.child_nodes {
                if self.hierarchy_node(child).is_none() {
                    return Err(Error::ChildNode {
                        node: node_id.to_u32(),
                        child: child.to_u32(),
                    });
                }
                if !seen_child_nodes.insert(child.to_u32()) {
                    return Err(Error::DuplicateChildNode {
                        node: node_id.to_u32(),
                        child: child.to_u32(),
                    });
                }
            }
            let mut seen_child_objects = HashSet::with_capacity(node.child_objects.len());
            for &object in &node.child_objects {
                if self.object(object).is_none() {
                    return Err(Error::ChildObject {
                        node: node_id.to_u32(),
                        object: object.to_u32(),
                    });
                }
                if !seen_child_objects.insert(object.to_u32()) {
                    return Err(Error::DuplicateChildObject {
                        node: node_id.to_u32(),
                        object: object.to_u32(),
                    });
                }
            }

            // The node transform must be non-degenerate.
            let scale = node.transform.scale;
            if scale.x == 0.0 || scale.y == 0.0 || scale.z == 0.0 {
                return Err(Error::ZeroScale {
                    node: node_id.to_u32(),
                });
            }
            let rotation = node.transform.rotation;
            let length_squared = rotation.x * rotation.x
                + rotation.y * rotation.y
                + rotation.z * rotation.z
                + rotation.w * rotation.w;
            if (length_squared - 1.0).abs() > ROTATION_TOLERANCE {
                return Err(Error::NonUnitRotation {
                    node: node_id.to_u32(),
                });
            }
        }

        // Roots.
        let mut seen_roots = HashSet::with_capacity(self.runtime_state.root_hierarchy_nodes.len());
        for &root in &self.runtime_state.root_hierarchy_nodes {
            if self.hierarchy_node(root).is_none() {
                return Err(Error::Root {
                    root: root.to_u32(),
                });
            }
            if !seen_roots.insert(root.to_u32()) {
                return Err(Error::DuplicateRoot {
                    root: root.to_u32(),
                });
            }
        }

        // Acyclicity; every child is now known live.
        if let Some(node) = self.first_cycle_node() {
            return Err(Error::Cycle { node });
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
        let node_ids: Vec<_> = self.runtime_state.hierarchy_node_ids.iter().collect();
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
                    let children =
                        &unsafe { self.runtime_state.hierarchy_nodes.get(node_id) }.child_nodes;
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

    /// Deep copy. The runtime scene rebuilds its columns against fresh id pools;
    /// the edit column is `Copy`, so it clones directly.
    pub fn clone_state(&self) -> Self {
        Self {
            runtime_state: self.runtime_state.clone_runtime_state(),
            edit_state: self.edit_state.clone(),
            ext: self.ext.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BVoxAttribute, BVoxHierarchyNode, BVoxObject, BVoxPalette, BVoxPaletteCell, BVoxPaletteRef,
        Error, VoxEditObject, VoxHierarchyNode, VoxObject, VoxPalette, VoxState, VoxValue,
    };
    use branded_id::U32Id;
    use ty_math::{TyQuaternion, TyVector3, TyVector3I32, TyVector3U32};

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

    /// A 1x1x1 object with its single voxel live, so its grid is exactly tight.
    fn unit_object(name: &str) -> VoxObject {
        let mut object = VoxObject::new(name.to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        let voxel = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel, &[]).unwrap();
        object
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
    fn validate_accepts_a_shared_child_dag() {
        let mut state = VoxState::default();
        let leaf = state.add_hierarchy_node(VoxHierarchyNode::default());
        // Sharing a child across parents is legal in a DAG; each parent lists it
        // once.
        let a = state.add_hierarchy_node(node_with_children(vec![leaf]));
        let b = state.add_hierarchy_node(node_with_children(vec![leaf]));
        state.set_root_hierarchy_nodes(vec![a, b]);

        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn validate_rejects_a_duplicate_child_node() {
        let mut state = VoxState::default();
        let leaf = state.add_hierarchy_node(VoxHierarchyNode::default());
        let parent = state.add_hierarchy_node(node_with_children(vec![leaf, leaf]));
        assert_eq!(
            state.validate(),
            Err(Error::DuplicateChildNode {
                node: parent.to_u32(),
                child: leaf.to_u32(),
            })
        );
    }

    #[test]
    fn validate_rejects_a_duplicate_child_object() {
        let mut state = VoxState::default();
        let object = state.add_object(unit_object("o"));
        let node = state.add_hierarchy_node(node_with_objects(vec![object, object]));
        assert_eq!(
            state.validate(),
            Err(Error::DuplicateChildObject {
                node: node.to_u32(),
                object: object.to_u32(),
            })
        );
    }

    #[test]
    fn validate_rejects_a_duplicate_root() {
        let mut state = VoxState::default();
        let node = state.add_hierarchy_node(VoxHierarchyNode::default());
        state.set_root_hierarchy_nodes(vec![node, node]);
        assert_eq!(
            state.validate(),
            Err(Error::DuplicateRoot {
                root: node.to_u32(),
            })
        );
    }

    #[test]
    fn validate_rejects_a_duplicate_palette_ref() {
        let mut state = VoxState::default();
        let palette = state.add_palette(one_cell_palette(0.0));
        let mut object = unit_object("o");
        // Two references naming the same palette.
        object.add_palette_ref(palette, cell_id(0));
        object.add_palette_ref(palette, cell_id(0));
        let object = state.add_object(object);
        assert_eq!(
            state.validate(),
            Err(Error::DuplicatePaletteRef {
                object: object.to_u32(),
                palette: palette.to_u32(),
            })
        );
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
            Err(Error::PaletteRef {
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
            Err(Error::ChildNode { child: 9, .. })
        ));
    }

    #[test]
    fn validate_rejects_dangling_root() {
        let mut state = VoxState::default();
        state.add_hierarchy_node(VoxHierarchyNode::default());
        state.set_root_hierarchy_nodes(vec![node_id(7)]);
        assert_eq!(state.validate(), Err(Error::Root { root: 7 }));
    }

    #[test]
    fn validate_rejects_a_cycle() {
        let mut state = VoxState::default();
        // node 0 -> child 1, node 1 -> child 0.
        state.add_hierarchy_node(node_with_children(vec![node_id(1)]));
        state.add_hierarchy_node(node_with_children(vec![node_id(0)]));
        assert!(matches!(state.validate(), Err(Error::Cycle { .. })));
    }

    #[test]
    fn validate_rejects_a_duplicate_attribute_key() {
        let mut state = VoxState::default();
        let mut palette = VoxPalette::default();
        palette.add_attribute("rgba".to_owned());
        palette.add_attribute("rgba".to_owned());
        state.add_palette(palette);
        assert_eq!(
            state.validate(),
            Err(Error::DuplicateAttribute {
                palette: 0,
                attribute: 1,
            })
        );
    }

    #[test]
    fn validate_rejects_a_zero_scale() {
        let mut state = VoxState::default();
        let mut node = VoxHierarchyNode::default();
        node.transform.scale = TyVector3::new(1.0, 0.0, 1.0);
        let id = state.add_hierarchy_node(node);
        assert_eq!(
            state.validate(),
            Err(Error::ZeroScale { node: id.to_u32() })
        );
    }

    #[test]
    fn validate_rejects_a_non_unit_rotation() {
        let mut state = VoxState::default();
        let mut node = VoxHierarchyNode::default();
        // Length squared 4, well outside the unit tolerance.
        node.transform.rotation = TyQuaternion::new(0.0, 0.0, 0.0, 2.0);
        let id = state.add_hierarchy_node(node);
        assert_eq!(
            state.validate(),
            Err(Error::NonUnitRotation { node: id.to_u32() })
        );
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
        assert!(remap.cells[palette_a.to_usize_id()].is_empty());
        assert_eq!(
            remap.cells[palette_b.to_usize_id()].new_id(cell_id(0)),
            Some(cell_id(0))
        );
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

    #[test]
    fn edit_objects_default_to_the_runtime_grid_and_survive_gc() {
        let mut state = VoxState::default();
        let a =
            state.add_object(VoxObject::new("a".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap());
        let b =
            state.add_object(VoxObject::new("b".to_owned(), TyVector3U32::new(3, 3, 3)).unwrap());

        // A new object's edit grid is its runtime grid (zero margin).
        assert_eq!(
            state.edit_object(a),
            Some(VoxEditObject {
                bounds: TyVector3U32::new(2, 1, 1),
                origin: TyVector3I32::default(),
            })
        );

        // Give `b` a distinct edit grid carrying margin.
        let b_edit = VoxEditObject {
            bounds: TyVector3U32::new(5, 5, 5),
            origin: TyVector3I32::new(-1, -1, -1),
        };
        assert_eq!(state.set_edit_object(b, b_edit), Some(()));

        // Remove `a` and gc: `b` renumbers to 0 and its edit grid moves with it.
        assert_eq!(state.remove_object(a), Some(()));
        assert_eq!(state.edit_object(a), None);
        state.gc();

        let b0 = U32Id::<BVoxObject>::from_u32(0);
        assert_eq!(state.object(b0).unwrap().name(), "b");
        assert_eq!(state.edit_object(b0), Some(b_edit));

        // An unknown id has no edit grid, and setting one is rejected.
        let unknown = U32Id::<BVoxObject>::from_u32(9);
        assert_eq!(state.edit_object(unknown), None);
        assert_eq!(state.set_edit_object(unknown, b_edit), None);
    }
}
