use crate::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxPoolValue, BVoxProperty,
    BVoxValuePool, Error, Result, VoxBound, VoxGcRemap, VoxHierarchyNode, VoxObject, VoxPalette,
    VoxPoolValueRef, VoxRuntimeState, VoxValue, VoxValuePool, VoxValuePoolKind,
};
use branded_id::{IdVec, U32Id, soa::IdRemap};
use std::collections::{HashMap, HashSet};
use ty_math::{TyQuaternionExt, TyVector3F64};

/// The in-memory state of a voxel model: its objects, shared palettes, scene
/// hierarchy, and roots.
///
/// Ids are meaningful only within this state. [`validate`](Self::validate)
/// checks the cross-references.
#[derive(Debug, Default)]
pub struct VoxMain {
    /// The runtime scene: objects.
    runtime_state: VoxRuntimeState,

    /// Optional user-extension namespace; the core format assigns it no
    /// meaning.
    ext: Option<VoxValue>,
}

impl VoxMain {
    /// Adds an object at the end of the listing, returning its id.
    pub fn add_object(&mut self, object: VoxObject) -> U32Id<BVoxObject> {
        let id = self.runtime_state.object_ids.retain();
        self.runtime_state.objects.retain(id, object);
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

    /// Objects in listing order, as `(id, object)`.
    pub fn iter_objects(&self) -> impl Iterator<Item = (U32Id<BVoxObject>, &VoxObject)> + '_ {
        // Safety: retained ids have a value.
        self.runtime_state
            .object_ids
            .iter()
            .map(move |id| (id, unsafe { self.runtime_state.objects.get(id) }))
    }

    /// The object `id` for mutation, or `None` if not one of this state's.
    /// Pairs with [`object`](Self::object) for read-only access.
    pub fn object_mut(&mut self, id: U32Id<BVoxObject>) -> Option<&mut VoxObject> {
        if !self.runtime_state.object_ids.is_retained(id) {
            return None;
        }
        // Safety: the id is retained, so it has a value.
        Some(unsafe { self.runtime_state.objects.get_mut(id) })
    }

    /// Moves object `id` to position `index` in the listing, shifting the
    /// objects between its old and new positions one slot. Returns `None` and
    /// changes nothing if `id` is not one of this state's objects or `index` is
    /// at or past [`object_count`](Self::object_count).
    pub fn move_object(&mut self, id: U32Id<BVoxObject>, index: usize) -> Option<()> {
        if !self.runtime_state.object_ids.is_retained(id)
            || index >= self.runtime_state.object_ids.len()
        {
            return None;
        }
        self.runtime_state.object_ids.move_to(id, index);
        Some(())
    }

    /// The listing position of object `id`, or `None` if `id` is not one of
    /// this state's objects.
    pub fn object_index(&self, id: U32Id<BVoxObject>) -> Option<usize> {
        self.runtime_state.object_ids.index_of(id)
    }

    /// Adds a shared palette at the end of the listing, returning its id.
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

    /// Palettes in listing order, as `(id, palette)`.
    pub fn iter_palettes(&self) -> impl Iterator<Item = (U32Id<BVoxPalette>, &VoxPalette)> + '_ {
        // Safety: retained ids have a value.
        self.runtime_state
            .palette_ids
            .iter()
            .map(move |id| (id, unsafe { self.runtime_state.palettes.get(id) }))
    }

    /// Moves palette `id` to position `index` in the listing, shifting the
    /// palettes between its old and new positions one slot. Returns `None` and
    /// changes nothing if `id` is not one of this state's palettes or `index`
    /// is at or past [`palette_count`](Self::palette_count).
    pub fn move_palette(&mut self, id: U32Id<BVoxPalette>, index: usize) -> Option<()> {
        if !self.runtime_state.palette_ids.is_retained(id)
            || index >= self.runtime_state.palette_ids.len()
        {
            return None;
        }
        self.runtime_state.palette_ids.move_to(id, index);
        Some(())
    }

    /// The listing position of palette `id`, or `None` if `id` is not one of
    /// this state's palettes.
    pub fn palette_index(&self, id: U32Id<BVoxPalette>) -> Option<usize> {
        self.runtime_state.palette_ids.index_of(id)
    }

    /// Adds a shared value pool at the end of the listing, returning its id.
    pub fn add_value_pool(&mut self, pool: VoxValuePool) -> U32Id<BVoxValuePool> {
        let id = self.runtime_state.value_pool_ids.retain();
        self.runtime_state.value_pools.retain(id, pool);
        id
    }

    /// Number of shared value pools.
    pub fn value_pool_count(&self) -> usize {
        self.runtime_state.value_pool_ids.len()
    }

    /// The value pool `id`, or `None` if not one of this state's.
    pub fn value_pool(&self, id: U32Id<BVoxValuePool>) -> Option<&VoxValuePool> {
        // Safety: retained ids have a value.
        self.runtime_state
            .value_pool_ids
            .is_retained(id)
            .then(|| unsafe { self.runtime_state.value_pools.get(id) })
    }

    /// Value pools in listing order, as `(id, pool)`.
    pub fn iter_value_pools(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxValuePool>, &VoxValuePool)> + '_ {
        // Safety: retained ids have a value.
        self.runtime_state
            .value_pool_ids
            .iter()
            .map(move |id| (id, unsafe { self.runtime_state.value_pools.get(id) }))
    }

    /// Moves value pool `id` to position `index` in the listing, shifting the
    /// pools between its old and new positions one slot. Returns `None` and
    /// changes nothing if `id` is not one of this state's pools or `index` is
    /// at or past [`value_pool_count`](Self::value_pool_count).
    pub fn move_value_pool(&mut self, id: U32Id<BVoxValuePool>, index: usize) -> Option<()> {
        if !self.runtime_state.value_pool_ids.is_retained(id)
            || index >= self.runtime_state.value_pool_ids.len()
        {
            return None;
        }
        self.runtime_state.value_pool_ids.move_to(id, index);
        Some(())
    }

    /// The listing position of value pool `id`, or `None` if `id` is not one
    /// of this state's pools.
    pub fn value_pool_index(&self, id: U32Id<BVoxValuePool>) -> Option<usize> {
        self.runtime_state.value_pool_ids.index_of(id)
    }

    /// Resolves what `material` in `palette` draws for `property`: the
    /// value pool the property draws from and the value id in that pool.
    /// `None` if any id is not this state's, `property` is not
    /// `palette`'s, or the property names a pool this state does not hold.
    /// Read the value at that id out of the returned pool by the pool's
    /// kind.
    pub fn material_value(
        &self,
        palette: U32Id<BVoxPalette>,
        material: U32Id<BVoxMaterial>,
        property: U32Id<BVoxProperty>,
    ) -> Option<(&VoxValuePool, U32Id<BVoxPoolValue>)> {
        let palette = self.palette(palette)?;
        let value_id = palette.value_id(material, property)?;
        let pool = self.value_pool(palette.property(property)?.pool)?;
        Some((pool, value_id))
    }

    /// Adds a hierarchy node at the end of the listing, returning its id. Its
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

    /// Hierarchy nodes in listing order, as `(id, node)`.
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

    /// Appends a root. Root uniqueness is checked by
    /// [`validate`](Self::validate), not here.
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

    /// Removes object `id`, detaching it from every node's `child_objects`.
    /// Returns `None` and changes nothing if `id` is not one of this state's
    /// objects. Leaves a hole until [`gc`](Self::gc) renumbers for a
    /// deterministic save.
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
        // Safety: a retained object id has a value.
        unsafe { self.runtime_state.objects.release(id) };
        self.runtime_state.object_ids.release_stable(id);
        Some(())
    }

    /// Removes palette `id`, detaching every object reference to it (along with
    /// that reference's per-voxel sample column). Returns `None` and changes
    /// nothing if `id` is not one of this state's palettes. Leaves a hole until
    /// [`gc`](Self::gc) renumbers.
    pub fn remove_palette(&mut self, id: U32Id<BVoxPalette>) -> Option<()> {
        if !self.runtime_state.palette_ids.is_retained(id) {
            return None;
        }
        let object_ids: Vec<_> = self.runtime_state.object_ids.iter().collect();
        for object_id in object_ids {
            // Safety: retained object ids have a value.
            let object = unsafe { self.runtime_state.objects.get_mut(object_id) };
            object.remove_layers_to(id);
        }
        // Safety: a retained palette id has a value; its Drop frees its cells.
        unsafe { self.runtime_state.palettes.release(id) };
        self.runtime_state.palette_ids.release_stable(id);
        Some(())
    }

    /// Removes hierarchy node `id`, detaching it from every `child_nodes` list
    /// and from the roots. Returns `None` and changes nothing if `id` is not
    /// one of this state's nodes. Leaves a hole until [`gc`](Self::gc)
    /// renumbers.
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
        self.runtime_state.hierarchy_node_ids.release_stable(id);
        Some(())
    }

    /// Removes `material` from `palette`, first repainting every live voxel
    /// that samples it onto `replacement` so no voxel is left without a
    /// material. Returns `None` and changes nothing if `palette` is not one of
    /// this state's palettes, if `material` or `replacement` is not one of that
    /// palette's materials, or if `replacement` is `material` itself. Leaves a
    /// hole until [`gc`](Self::gc) renumbers.
    pub fn remove_material(
        &mut self,
        palette: U32Id<BVoxPalette>,
        material: U32Id<BVoxMaterial>,
        replacement: U32Id<BVoxMaterial>,
    ) -> Option<()> {
        if !self.runtime_state.palette_ids.is_retained(palette) || material == replacement {
            return None;
        }
        // Safety: the palette id is retained.
        let palette_ref = unsafe { self.runtime_state.palettes.get(palette) };
        if !palette_ref.contains_material(material) || !palette_ref.contains_material(replacement) {
            return None;
        }

        let object_ids: Vec<_> = self.runtime_state.object_ids.iter().collect();
        for object_id in object_ids {
            // Safety: retained object ids have a value.
            let object = unsafe { self.runtime_state.objects.get_mut(object_id) };
            object.repaint_material(palette, material, replacement);
        }

        // Safety: the palette id is retained; the material is one of its
        // materials.
        unsafe { self.runtime_state.palettes.get_mut(palette) }.remove_material(material);
        Some(())
    }

    /// Removes `value` from `pool`, first repointing every palette cell that
    /// draws it onto `replacement` so no material is left without a value.
    /// Returns `None` and changes nothing if `pool` is not one of this state's
    /// pools, if `value` or `replacement` is not one of that pool's values, or
    /// if `replacement` is `value` itself. Leaves a hole until [`gc`](Self::gc)
    /// renumbers.
    pub fn remove_pool_value(
        &mut self,
        pool: U32Id<BVoxValuePool>,
        value: U32Id<BVoxPoolValue>,
        replacement: U32Id<BVoxPoolValue>,
    ) -> Option<()> {
        if !self.runtime_state.value_pool_ids.is_retained(pool) || value == replacement {
            return None;
        }
        // Safety: the pool id is retained.
        let pool_ref = unsafe { self.runtime_state.value_pools.get(pool) };
        if !pool_ref.contains_value(value) || !pool_ref.contains_value(replacement) {
            return None;
        }

        for palette_id in self.runtime_state.palette_ids.iter() {
            // Safety: retained palette ids have a value.
            let palette = unsafe { self.runtime_state.palettes.get_mut(palette_id) };
            palette.repoint_pool_value(pool, value, replacement);
        }

        // Safety: the pool id is retained and the value is one of its values.
        unsafe { self.runtime_state.value_pools.get_mut(pool) }.release_value_stable(value);
        Some(())
    }

    /// Compacts every id pool back to a contiguous `0..len` in listing order
    /// and rewrites every cross-reference to match, so a state edited by
    /// removals and moves numbers its entities the way a freshly loaded one
    /// does and saves stay deterministic. Call it once before saving, not after
    /// each removal or move.
    ///
    /// Requires a referentially valid state. The `remove_*` methods preserve it
    /// by detaching what they remove, and [`validate`](Self::validate) checks
    /// it. The voxel grids are dense and never compacted, so voxel ids keep
    /// equaling their raster index.
    ///
    /// Returns the [`VoxGcRemap`] recording where each id moved, so any ids
    /// held outside the state can be translated to their compacted values.
    pub fn gc(&mut self) -> VoxGcRemap {
        // Compact each pool's values first, recording the value relabelings
        // by the pool's pre-gc id so the palette pass below can translate its
        // cells before pool ids move.
        let pool_id_space = self.runtime_state.value_pool_ids.peek_next_fresh().to_u32() as usize;
        let mut pool_value_remaps: IdVec<BVoxValuePool, IdRemap<BVoxPoolValue, u32>> =
            IdVec::from_vec((0..pool_id_space).map(|_| IdRemap::default()).collect());
        for pool_id in self.runtime_state.value_pool_ids.iter() {
            // Safety: retained pool ids have a value.
            let pool = unsafe { self.runtime_state.value_pools.get_mut(pool_id) };
            pool_value_remaps[pool_id.to_usize_id()] = pool.gc_values();
        }

        // Compact the shared value-pool store, then relabel every palette
        // property's pool, so the pool ids are settled before palettes are
        // compacted. Pool ids follow the listing, so a pool moved before gc is
        // renumbered here and every property's pool id is rewritten to match.
        let pool_remap = self.runtime_state.value_pool_ids.gc();
        // Safety: the value-pool column was in sync with the pre-gc pool, and
        // nothing has retained or released since.
        unsafe { self.runtime_state.value_pools.gc(&pool_remap) };

        // Compact each palette's own pools, so the material relabelings are
        // ready when object samples are translated below. They are indexed by
        // old palette id, so the column covers the palette pool's whole id
        // space. Cells translate through the value relabelings first, while
        // each property still names its pool's pre-gc id.
        let palette_id_space = self.runtime_state.palette_ids.peek_next_fresh().to_u32() as usize;
        let mut material_remaps: IdVec<BVoxPalette, IdRemap<BVoxMaterial, u32>> =
            IdVec::from_vec((0..palette_id_space).map(|_| IdRemap::default()).collect());
        for palette_id in self.runtime_state.palette_ids.iter().collect::<Vec<_>>() {
            // Safety: retained palette ids have a value.
            let palette = unsafe { self.runtime_state.palettes.get_mut(palette_id) };
            palette.relabel_pool_values(&pool_value_remaps);
            palette.relabel_value_pools(&pool_remap);
            material_remaps[palette_id.to_usize_id()] = palette.gc();
        }

        // Compact the palette pool.
        let palette_remap = self.runtime_state.palette_ids.gc();
        // Safety: the palette column was in sync with the pre-gc palette pool,
        // and nothing has retained or released since.
        unsafe { self.runtime_state.palettes.gc(&palette_remap) };

        // Rewrite each object's palette references and sample cells, then
        // compact its own reference pool.
        let object_ids: Vec<_> = self.runtime_state.object_ids.iter().collect();
        for object_id in object_ids {
            // Safety: retained object ids have a value.
            unsafe { self.runtime_state.objects.get_mut(object_id) }
                .gc(&palette_remap, &material_remaps);
        }

        // Compact the object pool.
        let object_remap = self.runtime_state.object_ids.gc();
        // Safety: the object column was in sync with the pre-gc object pool,
        // and nothing has retained or released since.
        unsafe { self.runtime_state.objects.gc(&object_remap) };

        // Compact the node pool, then translate child links and roots, which
        // point at the relabeled nodes and objects.
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
            value_pools: pool_remap,
            pool_values: pool_value_remaps,
            objects: object_remap,
            palettes: palette_remap,
            hierarchy_nodes: node_remap,
            materials: material_remaps,
        }
    }

    /// Releases value-pool entries no material references, keeping the
    /// survivors' listing order and their ids. The pool-value counterpart to
    /// the entity `remove_*` methods. [`gc`](Self::gc) renumbers. Requires a
    /// referentially valid state, which [`validate`](Self::validate) checks.
    ///
    /// 1. references union across palettes, so a shared entry survives while
    ///    any one material uses it
    /// 2. a pool nothing references is left whole, since
    ///    [`validate`](Self::validate) requires every pool non-empty
    /// 3. the state stays referentially valid
    pub fn prune_value_pools(&mut self) {
        // The value ids each pool still has a material referencing.
        let pool_ids: Vec<_> = self.runtime_state.value_pool_ids.iter().collect();
        let mut referenced: HashMap<U32Id<BVoxValuePool>, HashSet<U32Id<BVoxPoolValue>>> =
            pool_ids.iter().map(|&id| (id, HashSet::new())).collect();

        for palette_id in self.runtime_state.palette_ids.iter() {
            // Safety: retained palette ids have a value.
            let palette = unsafe { self.runtime_state.palettes.get(palette_id) };
            for (property_id, property) in palette.iter_properties() {
                let uses = referenced
                    .get_mut(&property.pool)
                    .expect("a property names a live value pool in a valid state");
                for material in palette.iter_materials() {
                    let value_id = palette
                        .value_id(material, property_id)
                        .expect("a retained material has a value id for every property");
                    uses.insert(value_id);
                }
            }
        }

        // Release each pool's unreferenced entries. A pool nothing references
        // is left whole.
        for &pool_id in &pool_ids {
            let keep = &referenced[&pool_id];
            if keep.is_empty() {
                continue;
            }
            // Safety: retained pool ids have a value.
            let pool = unsafe { self.runtime_state.value_pools.get_mut(pool_id) };
            let doomed: Vec<_> = pool
                .iter_values()
                .map(|(value_id, _)| value_id)
                .filter(|value_id| !keep.contains(value_id))
                .collect();
            for value_id in doomed {
                pool.release_value_stable(value_id);
            }
        }
    }

    /// Reorders `pool`'s values to `new_order`, which lists the pool's value
    /// ids in their new listing order. Value ids are stable, so what every
    /// material resolves to is unchanged. Returns `None` and changes nothing if
    /// `pool` is not one of this state's or `new_order` does not list each of
    /// the pool's value ids exactly once.
    pub fn reorder_value_pool(
        &mut self,
        pool: U32Id<BVoxValuePool>,
        new_order: &[U32Id<BVoxPoolValue>],
    ) -> Option<()> {
        if !self.runtime_state.value_pool_ids.is_retained(pool) {
            return None;
        }
        // Safety: the id is retained, so it has a value.
        let values = unsafe { self.runtime_state.value_pools.get_mut(pool) };
        if !is_permutation(values, new_order) {
            return None;
        }
        values.set_value_order(new_order);
        Some(())
    }

    /// Checks the value pools, palettes, cross-references, and per-entity rules:
    ///
    /// 1. every value pool is non-empty, its values well-formed for its kind,
    ///    and its `min`/`max` finite, integer-valued for an `int` pool, and
    ///    ordered
    /// 2. per palette:
    ///    1. every property names a live value pool
    ///    2. no property name repeats
    ///    3. every material value id is within its property's pool
    ///    4. there is at least one material
    /// 3. every object layer references a live palette (two layers may share
    ///    one), and every live-voxel sample material is within its layer's
    ///    palette
    /// 4. every node child node and child object resolves, and no node lists
    ///    the same one twice
    /// 5. every root resolves, and no root repeats
    /// 6. every node transform has finite position and scale components, a
    ///    non-zero scale on each axis, and a unit-length rotation quaternion
    ///    within `1e-6`
    /// 7. the `child_nodes` graph is acyclic
    ///
    /// A node may have several parents, since the hierarchy is a DAG; that
    /// sharing is not a cycle.
    pub fn validate(&self) -> Result<()> {
        // How far a rotation quaternion's length-squared may stray from 1 and
        // still count as a unit quaternion.
        const ROTATION_TOLERANCE: f64 = 1e-6;

        // Value pools are non-empty and their values and bounds well-formed for
        // their kind. This runs first, so a palette that reads a malformed pool
        // is reported after the pool it reads.
        for (pool_id, pool) in self.iter_value_pools() {
            check_value_pool(pool_id, pool)?;
        }

        // Palette property rules: pools resolve and every value id is within
        // its pool. Names are unique by construction, which add_property
        // enforces.
        for (palette_id, palette) in self.iter_palettes() {
            for (property_id, property) in palette.iter_properties() {
                let pool = self.value_pool(property.pool).ok_or(Error::PropertyPool {
                    palette: palette_id,
                    property: property_id,
                    pool: property.pool,
                })?;
                for material_id in palette.iter_materials() {
                    let value_id = palette
                        .value_id(material_id, property_id)
                        .expect("a material has a value id for every property");
                    if !pool.contains_value(value_id) {
                        return Err(Error::MaterialValue {
                            palette: palette_id,
                            property: property_id,
                            material: material_id,
                        });
                    }
                }
            }

            // Every palette is sampled, so it needs a material to sample.
            if palette.material_count() == 0 {
                return Err(Error::PaletteWithoutMaterials {
                    palette: palette_id,
                });
            }
        }

        // Object layer palette refs and live-voxel sample materials. Checks are
        // by id retention, not index range, so they hold whether or not
        // removals have left the pools with holes. Two layers may reference the
        // same palette, so there is no duplicate-layer rule.
        for (object_id, object) in self.iter_objects() {
            let mut layer_palettes = Vec::with_capacity(object.layer_count());
            for (layer_id, palette_id) in object.iter_layers() {
                let palette = self.palette(palette_id).ok_or(Error::PaletteRef {
                    object: object_id,
                    palette: palette_id,
                })?;
                layer_palettes.push((layer_id, palette));
            }
            // Every live voxel samples a material within each layer's
            // palette. Layer-major so each layer's sample column is read
            // once.
            for &(layer_id, palette) in &layer_palettes {
                let samples = object
                    .iter_live_samples(layer_id)
                    .expect("an iterated layer is one of the object's layers");
                for (voxel_id, material) in samples {
                    if !palette.contains_material(material) {
                        return Err(Error::SampleMaterial {
                            object: object_id,
                            voxel: voxel_id,
                            material,
                        });
                    }
                }
            }
        }

        // Node children; retention-checked before the cycle pass.
        for (node_id, node) in self.iter_hierarchy_nodes() {
            let mut seen_child_nodes = HashSet::with_capacity(node.child_nodes.len());
            for &child in &node.child_nodes {
                if self.hierarchy_node(child).is_none() {
                    return Err(Error::ChildNode {
                        node: node_id,
                        child,
                    });
                }
                if !seen_child_nodes.insert(child) {
                    return Err(Error::DuplicateChildNode {
                        node: node_id,
                        child,
                    });
                }
            }
            let mut seen_child_objects = HashSet::with_capacity(node.child_objects.len());
            for &object in &node.child_objects {
                if self.object(object).is_none() {
                    return Err(Error::ChildObject {
                        node: node_id,
                        object,
                    });
                }
                if !seen_child_objects.insert(object) {
                    return Err(Error::DuplicateChildObject {
                        node: node_id,
                        object,
                    });
                }
            }

            // The node transform must be finite and non-degenerate. The
            // rotation needs no finiteness guard of its own: a non-finite
            // component fails the unit-length check below.
            let position = node.transform.position;
            let scale = node.transform.scale;
            if !vector_is_finite(position) || !vector_is_finite(scale) {
                return Err(Error::NonFiniteTransform { node: node_id });
            }
            if scale.x == 0.0 || scale.y == 0.0 || scale.z == 0.0 {
                return Err(Error::ZeroScale { node: node_id });
            }
            let rotation = node.transform.rotation;
            if !rotation.is_normalized_within(ROTATION_TOLERANCE) {
                return Err(Error::NonUnitRotation { node: node_id });
            }
        }

        // Roots.
        let mut seen_roots = HashSet::with_capacity(self.runtime_state.root_hierarchy_nodes.len());
        for &root in &self.runtime_state.root_hierarchy_nodes {
            if self.hierarchy_node(root).is_none() {
                return Err(Error::Root { root });
            }
            if !seen_roots.insert(root) {
                return Err(Error::DuplicateRoot { root });
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
    /// Call only after every child id is known live. Works over the retained
    /// node ids by position, so it holds whether or not the pool has holes.
    fn first_cycle_node(&self) -> Option<U32Id<BVoxHierarchyNode>> {
        const WHITE: u8 = 0;
        const GREY: u8 = 1;
        const BLACK: u8 = 2;

        // Retained node ids and a lookup from id to its position here, so a
        // holed pool (ids not contiguous from zero) is handled the same as a
        // packed one.
        let node_ids: Vec<_> = self.runtime_state.hierarchy_node_ids.iter().collect();
        let index_of: HashMap<U32Id<BVoxHierarchyNode>, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(index, &id)| (id, index))
            .collect();
        let count = node_ids.len();
        let mut colour = vec![WHITE; count];

        for start in 0..count {
            if colour[start] != WHITE {
                continue;
            }
            colour[start] = GREY;
            // Each frame is a node position plus how many children we have
            // walked.
            let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
            while let Some(&(node, cursor)) = stack.last() {
                let node_id = node_ids[node];
                let next_child = {
                    // Safety: `node_id` is a retained node id.
                    let children =
                        &unsafe { self.runtime_state.hierarchy_nodes.get(node_id) }.child_nodes;
                    (cursor < children.len()).then(|| children[cursor])
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
                            GREY => return Some(node_ids[child]),
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

    /// Deep copy. The runtime scene rebuilds its columns against fresh id
    /// pools.
    pub fn clone_state(&self) -> Self {
        Self {
            runtime_state: self.runtime_state.clone_runtime_state(),
            ext: self.ext.clone(),
        }
    }
}

/// Checks a value pool is non-empty and every value and bound is well-formed
/// for its kind:
///
/// 1. int/float bounds finite, integer-valued for `int`, and ordered
/// 2. int/float values finite and within bounds
/// 3. color components within their space's range
///
/// `pool_id` is the pool's id, for the error.
fn check_value_pool(pool_id: U32Id<BVoxValuePool>, pool: &VoxValuePool) -> Result<()> {
    if pool.values_len() == 0 {
        return Err(Error::EmptyPool { pool: pool_id });
    }
    match pool.kind() {
        VoxValuePoolKind::Float { min, max, .. } => {
            check_numeric_bounds(pool_id, min, max, false)?;
            for (value_id, value) in pool.iter_values() {
                let VoxPoolValueRef::Float(number) = value else {
                    unreachable!("a float pool yields float values");
                };
                if !number.is_finite() || !value_in_bounds(min, max, number) {
                    return Err(Error::PoolValue {
                        pool: pool_id,
                        value: value_id,
                    });
                }
            }
        }
        VoxValuePoolKind::Int { min, max, .. } => {
            check_numeric_bounds(pool_id, min, max, true)?;
            for (value_id, value) in pool.iter_values() {
                let VoxPoolValueRef::Int(number) = value else {
                    unreachable!("an int pool yields int values");
                };
                if !int_value_in_bounds(min, max, number) {
                    return Err(Error::PoolValue {
                        pool: pool_id,
                        value: value_id,
                    });
                }
            }
        }
        VoxValuePoolKind::Srgb { .. } | VoxValuePoolKind::Srgba { .. } => {
            check_color_components(pool_id, pool, false)?
        }
        VoxValuePoolKind::LinearRgb { .. } | VoxValuePoolKind::LinearRgba { .. } => {
            check_color_components(pool_id, pool, true)?
        }
        VoxValuePoolKind::Json { .. }
        | VoxValuePoolKind::Bool { .. }
        | VoxValuePoolKind::String { .. } => {}
    }
    Ok(())
}

/// Checks a bounded pool's `min`/`max`: each numeric bound is finite (and
/// integer-valued when `integer`), and `min <= max` when both are finite.
fn check_numeric_bounds(
    pool_id: U32Id<BVoxValuePool>,
    min: &VoxBound,
    max: &VoxBound,
    integer: bool,
) -> Result<()> {
    let min = bound_number(pool_id, min, integer)?;
    let max = bound_number(pool_id, max, integer)?;
    if let (Some(low), Some(high)) = (min, max)
        && low > high
    {
        return Err(Error::PoolBound { pool: pool_id });
    }
    Ok(())
}

/// A bound's finite numeric value, or `None` if unbounded. Rejects a non-finite
/// bound, or a non-integer bound on an `int` pool.
fn bound_number(
    pool_id: U32Id<BVoxValuePool>,
    bound: &VoxBound,
    integer: bool,
) -> Result<Option<f64>> {
    match bound {
        VoxBound::None => Ok(None),
        VoxBound::Number(number) => {
            if !number.is_finite() || (integer && number.fract() != 0.0) {
                Err(Error::PoolBound { pool: pool_id })
            } else {
                Ok(Some(*number))
            }
        }
    }
}

/// Whether `value` lies within `min`/`max`, each side unbounded when `None`.
fn value_in_bounds(min: &VoxBound, max: &VoxBound, value: f64) -> bool {
    let low_ok = match min {
        VoxBound::Number(low) => value >= *low,
        VoxBound::None => true,
    };
    let high_ok = match max {
        VoxBound::Number(high) => value <= *high,
        VoxBound::None => true,
    };
    low_ok && high_ok
}

/// Whether integer `value` lies within `min`/`max`, each side unbounded when
/// `None`. The integer sibling of [`value_in_bounds`]: it compares in the
/// integer domain, since casting an `i64` past 2^53 to `f64` rounds and could
/// carry it across a bound. Each numeric bound is finite and integer-valued,
/// which [`check_numeric_bounds`] establishes first.
fn int_value_in_bounds(min: &VoxBound, max: &VoxBound, value: i64) -> bool {
    let low_ok = match min {
        VoxBound::Number(low) => int_at_least(value, *low),
        VoxBound::None => true,
    };
    let high_ok = match max {
        VoxBound::Number(high) => int_at_most(value, *high),
        VoxBound::None => true,
    };
    low_ok && high_ok
}

/// Exact `value >= bound` for a finite integer-valued `bound`. `i64::MAX as
/// f64` rounds up to 2^63 and `i64::MIN as f64` is exactly -2^63, so the two
/// range tests filter every bound outside the `i64` range; the remainder
/// converts to `i64` exactly.
fn int_at_least(value: i64, bound: f64) -> bool {
    if bound >= i64::MAX as f64 {
        false
    } else if bound < i64::MIN as f64 {
        true
    } else {
        value >= bound as i64
    }
}

/// Exact `value <= bound` for a finite integer-valued `bound`. The mirror of
/// [`int_at_least`].
fn int_at_most(value: i64, bound: f64) -> bool {
    if bound >= i64::MAX as f64 {
        true
    } else if bound < i64::MIN as f64 {
        false
    } else {
        value <= bound as i64
    }
}

/// Checks each color's components lie in its space's range: sRGB in `[0, 1]`,
/// linear finite and `>= 0`. The sRGB range test rejects any non-finite
/// component on its own; the linear side is only lower-bounded, so it guards
/// finiteness explicitly to reject `+Infinity`, which would otherwise pass
/// `>= 0`.
fn check_color_components(
    pool_id: U32Id<BVoxValuePool>,
    pool: &VoxValuePool,
    linear: bool,
) -> Result<()> {
    for (value_id, value) in pool.iter_values() {
        let components: &[f64] = match value {
            VoxPoolValueRef::Srgb(color) => color,
            VoxPoolValueRef::Srgba(color) => color,
            VoxPoolValueRef::LinearRgb(color) => color,
            VoxPoolValueRef::LinearRgba(color) => color,
            _ => unreachable!("a color pool yields color values"),
        };
        for &component in components {
            let in_range = if linear {
                component.is_finite() && component >= 0.0
            } else {
                (0.0..=1.0).contains(&component)
            };
            if !in_range {
                return Err(Error::PoolValue {
                    pool: pool_id,
                    value: value_id,
                });
            }
        }
    }
    Ok(())
}

/// Whether every component of `vector` is finite.
fn vector_is_finite(vector: TyVector3F64) -> bool {
    vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()
}

/// Whether `order` lists each of `pool`'s value ids exactly once.
fn is_permutation(pool: &VoxValuePool, order: &[U32Id<BVoxPoolValue>]) -> bool {
    if order.len() != pool.values_len() {
        return false;
    }
    let mut seen = vec![false; order.len()];
    for &id in order {
        match pool.value_index(id) {
            Some(position) if !seen[position] => seen[position] = true,
            _ => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::{
        BVoxHierarchyNode, BVoxLayer, BVoxMaterial, BVoxObject, BVoxPalette, BVoxPoolValue,
        BVoxProperty, BVoxValuePool, BVoxVoxel, Error, VoxBound, VoxHierarchyNode, VoxMain,
        VoxObject, VoxPalette, VoxPoolValueRef, VoxValuePool, VoxValuePoolKind,
    };
    use branded_id::U32Id;
    use ty_math::{TyQuaternion, TyVector3, TyVector3U32};

    fn node_id(index: u32) -> U32Id<BVoxHierarchyNode> {
        U32Id::from_u32(index)
    }

    fn material(index: u32) -> U32Id<BVoxMaterial> {
        U32Id::from_u32(index)
    }

    fn value(index: u32) -> U32Id<BVoxPoolValue> {
        U32Id::from_u32(index)
    }

    fn pool(index: u32) -> U32Id<BVoxValuePool> {
        U32Id::from_u32(index)
    }

    fn palette(index: u32) -> U32Id<BVoxPalette> {
        U32Id::from_u32(index)
    }

    fn voxel(index: u32) -> U32Id<BVoxVoxel> {
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

    /// Adds an unbounded `int` value pool holding `values` and returns its id.
    fn int_pool(state: &mut VoxMain, values: Vec<i64>) -> U32Id<BVoxValuePool> {
        state.add_value_pool(VoxValuePool::int(VoxBound::None, VoxBound::None, values))
    }

    /// A palette with one property "v" on `pool` and two materials, drawing
    /// value ids 0 and 1.
    fn two_material_palette(pool: U32Id<BVoxValuePool>) -> VoxPalette {
        let mut palette = VoxPalette::default();
        palette.add_property("v".to_owned(), pool).unwrap();
        palette.add_material(vec![value(0)]).unwrap();
        palette.add_material(vec![value(1)]).unwrap();
        palette
    }

    /// A palette with one property "v" on `pool` and one material
    /// drawing value id `index`.
    fn one_material_palette(pool: U32Id<BVoxValuePool>, index: u32) -> VoxPalette {
        let mut palette = VoxPalette::default();
        palette.add_property("v".to_owned(), pool).unwrap();
        palette.add_material(vec![value(index)]).unwrap();
        palette
    }

    #[test]
    fn add_and_read_back_in_listing_order() {
        let mut state = VoxMain::default();
        let a = state.add_object(unit_object("a"));
        let b = state.add_object(unit_object("b"));

        assert_eq!(state.object_count(), 2);
        assert_eq!(state.object(a).unwrap().name(), "a");
        let names: Vec<&str> = state.iter_objects().map(|(_, o)| o.name()).collect();
        assert_eq!(names, ["a", "b"]);
        assert_eq!(b.to_u32(), 1);
    }

    #[test]
    fn add_and_read_back_value_pools_in_listing_order() {
        let mut state = VoxMain::default();
        let colors = state.add_value_pool(VoxValuePool::srgba(vec![[1.0, 0.0, 0.0, 1.0]]));
        let metallic = state.add_value_pool(VoxValuePool::float(
            VoxBound::Number(0.0),
            VoxBound::Number(1.0),
            vec![0.0, 1.0],
        ));

        assert_eq!(state.value_pool_count(), 2);
        assert_eq!(colors, U32Id::<BVoxValuePool>::from_u32(0));
        assert_eq!(metallic.to_u32(), 1);
        assert!(matches!(
            state.value_pool(colors).map(VoxValuePool::kind),
            Some(VoxValuePoolKind::Srgba { .. })
        ));
        assert_eq!(
            state.value_pool(metallic).map(VoxValuePool::values_len),
            Some(2)
        );
        // An id past the pool is not one of this state's.
        assert_eq!(state.value_pool(U32Id::<BVoxValuePool>::from_u32(2)), None);

        let mut pools = state.iter_value_pools();
        assert!(matches!(
            pools.next().map(|(_, pool)| pool.kind()),
            Some(VoxValuePoolKind::Srgba { .. })
        ));
        assert!(matches!(
            pools.next().map(|(_, pool)| pool.kind()),
            Some(VoxValuePoolKind::Float { .. })
        ));
        assert!(pools.next().is_none());
    }

    #[test]
    fn clone_state_deep_copies_value_pools() {
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::int(VoxBound::None, VoxBound::None, vec![7]));

        let copy = state.clone_state();
        assert_eq!(copy.value_pool_count(), 1);
        assert_eq!(
            copy.value_pool(U32Id::<BVoxValuePool>::from_u32(0)),
            Some(&VoxValuePool::int(VoxBound::None, VoxBound::None, vec![7]))
        );

        // Mutating the original must not touch the copy.
        state.add_value_pool(VoxValuePool::boolean(vec![true]));
        assert_eq!(state.value_pool_count(), 2);
        assert_eq!(copy.value_pool_count(), 1);
    }

    #[test]
    fn prune_value_pools_releases_unreferenced_entries_keeping_ids() {
        let mut state = VoxMain::default();
        // Four colors; the palette references only the middle two.
        let colors = state.add_value_pool(VoxValuePool::srgba(vec![
            [1.0, 0.0, 0.0, 1.0], // 0 red, unused
            [0.0, 1.0, 0.0, 1.0], // 1 green, used
            [0.0, 0.0, 1.0, 1.0], // 2 blue, unused
            [1.0, 1.0, 1.0, 1.0], // 3 white, used
        ]));
        let mut palette = VoxPalette::default();
        let property = palette
            .add_property("baseColorFactor".to_owned(), colors)
            .unwrap();
        let green = palette.add_material(vec![value(1)]).unwrap();
        let white = palette.add_material(vec![value(3)]).unwrap();
        let palette_id = state.add_palette(palette);
        state.validate().unwrap();

        state.prune_value_pools();

        // The pool keeps green then white in listing order, and the material
        // cells keep their ids. gc owns the renumbering.
        assert_eq!(
            state.value_pool(colors),
            Some(&VoxValuePool::srgba(vec![
                [0.0, 1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0, 1.0]
            ]))
        );
        let palette = state.palette(palette_id).unwrap();
        assert_eq!(palette.value_id(green, property), Some(value(1)));
        assert_eq!(palette.value_id(white, property), Some(value(3)));
        state.validate().unwrap();

        // gc renumbers the survivors to listing order: green to 0, white to 1.
        state.gc();
        let palette = state.palette(palette_id).unwrap();
        assert_eq!(palette.value_id(green, property), Some(value(0)));
        assert_eq!(palette.value_id(white, property), Some(value(1)));
        state.validate().unwrap();
    }

    #[test]
    fn prune_value_pools_keeps_entries_any_palette_still_uses() {
        let mut state = VoxMain::default();
        let ints = int_pool(&mut state, vec![10, 20, 30]);
        // Palette a draws id 0, palette b draws id 2, and id 1 is unused.
        let mut a = VoxPalette::default();
        let a_property = a.add_property("v".to_owned(), ints).unwrap();
        let a_material = a.add_material(vec![value(0)]).unwrap();
        let a_id = state.add_palette(a);
        let mut b = VoxPalette::default();
        let b_property = b.add_property("v".to_owned(), ints).unwrap();
        let b_material = b.add_material(vec![value(2)]).unwrap();
        let b_id = state.add_palette(b);
        state.validate().unwrap();

        state.prune_value_pools();

        // 10 and 30 survive (ids 0 and 2 used). 20 (id 1) is dropped, and the
        // survivors keep their ids until gc.
        assert_eq!(
            state.value_pool(ints),
            Some(&VoxValuePool::int(
                VoxBound::None,
                VoxBound::None,
                vec![10, 30]
            ))
        );
        assert_eq!(
            state
                .palette(a_id)
                .unwrap()
                .value_id(a_material, a_property),
            Some(value(0))
        );
        assert_eq!(
            state
                .palette(b_id)
                .unwrap()
                .value_id(b_material, b_property),
            Some(value(2))
        );
        state.validate().unwrap();
    }

    #[test]
    fn reorder_value_pool_permutes_the_listing_leaving_resolutions() {
        let mut state = VoxMain::default();
        // Three colors. Two palettes bind the pool, each with materials
        // drawing scattered ids.
        let colors = state.add_value_pool(VoxValuePool::srgba(vec![
            [1.0, 0.0, 0.0, 1.0], // 0 red
            [0.0, 1.0, 0.0, 1.0], // 1 green
            [0.0, 0.0, 1.0, 1.0], // 2 blue
        ]));
        let mut a = VoxPalette::default();
        let a_property = a
            .add_property("baseColorFactor".to_owned(), colors)
            .unwrap();
        let a_blue = a.add_material(vec![value(2)]).unwrap();
        let a_red = a.add_material(vec![value(0)]).unwrap();
        let a_id = state.add_palette(a);
        let mut b = VoxPalette::default();
        let b_property = b
            .add_property("baseColorFactor".to_owned(), colors)
            .unwrap();
        let b_green = b.add_material(vec![value(1)]).unwrap();
        let b_id = state.add_palette(b);
        state.validate().unwrap();

        // List blue first, then red, then green.
        assert_eq!(
            state.reorder_value_pool(colors, &[value(2), value(0), value(1)]),
            Some(())
        );

        // The pool follows the new order.
        assert_eq!(
            state.value_pool(colors),
            Some(&VoxValuePool::srgba(vec![
                [0.0, 0.0, 1.0, 1.0],
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, 1.0]
            ]))
        );
        // No cell is rewritten: value ids are stable, so every material keeps
        // its id and resolves to its original color.
        let a = state.palette(a_id).unwrap();
        assert_eq!(a.value_id(a_blue, a_property), Some(value(2)));
        assert_eq!(a.value_id(a_red, a_property), Some(value(0)));
        let pool = state.value_pool(colors).unwrap();
        assert_eq!(
            pool.value(value(2)),
            Some(VoxPoolValueRef::Srgba(&[0.0, 0.0, 1.0, 1.0]))
        );
        assert_eq!(
            state.palette(b_id).unwrap().value_id(b_green, b_property),
            Some(value(1))
        );
        state.validate().unwrap();
    }

    #[test]
    fn reorder_value_pool_rejects_a_non_permutation_without_changing_state() {
        let mut state = VoxMain::default();
        let ints = int_pool(&mut state, vec![10, 20, 30]);

        // A repeated id, a wrong length, an id not the pool's, and an unknown
        // pool all reject.
        assert_eq!(
            state.reorder_value_pool(ints, &[value(0), value(0), value(1)]),
            None
        );
        assert_eq!(state.reorder_value_pool(ints, &[value(0), value(1)]), None);
        assert_eq!(
            state.reorder_value_pool(ints, &[value(0), value(1), value(3)]),
            None
        );
        assert_eq!(
            state.reorder_value_pool(
                U32Id::<BVoxValuePool>::from_u32(9),
                &[value(0), value(1), value(2)]
            ),
            None
        );
        assert_eq!(
            state.value_pool(ints),
            Some(&VoxValuePool::int(
                VoxBound::None,
                VoxBound::None,
                vec![10, 20, 30]
            ))
        );
    }

    #[test]
    fn prune_value_pools_leaves_a_fully_referenced_pool() {
        let mut state = VoxMain::default();
        let ints = int_pool(&mut state, vec![1, 2]);
        let mut palette = VoxPalette::default();
        palette.add_property("v".to_owned(), ints).unwrap();
        palette.add_material(vec![value(0)]).unwrap();
        palette.add_material(vec![value(1)]).unwrap();
        state.add_palette(palette);

        state.prune_value_pools();

        assert_eq!(
            state.value_pool(ints).map(VoxValuePool::values_len),
            Some(2)
        );
    }

    #[test]
    fn validate_accepts_a_shared_child_dag() {
        let mut state = VoxMain::default();
        let leaf = state.add_hierarchy_node(VoxHierarchyNode::default());
        // Sharing a child across parents is legal in a DAG; each parent lists
        // it once.
        let a = state.add_hierarchy_node(node_with_children(vec![leaf]));
        let b = state.add_hierarchy_node(node_with_children(vec![leaf]));
        state.set_root_hierarchy_nodes(vec![a, b]);

        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn validate_rejects_a_duplicate_child_node() {
        let mut state = VoxMain::default();
        let leaf = state.add_hierarchy_node(VoxHierarchyNode::default());
        let parent = state.add_hierarchy_node(node_with_children(vec![leaf, leaf]));
        assert_eq!(
            state.validate(),
            Err(Error::DuplicateChildNode {
                node: parent,
                child: leaf,
            })
        );
    }

    #[test]
    fn validate_rejects_a_duplicate_child_object() {
        let mut state = VoxMain::default();
        let object = state.add_object(unit_object("o"));
        let node = state.add_hierarchy_node(node_with_objects(vec![object, object]));
        assert_eq!(
            state.validate(),
            Err(Error::DuplicateChildObject { node, object })
        );
    }

    #[test]
    fn validate_rejects_a_duplicate_root() {
        let mut state = VoxMain::default();
        let node = state.add_hierarchy_node(VoxHierarchyNode::default());
        state.set_root_hierarchy_nodes(vec![node, node]);
        assert_eq!(state.validate(), Err(Error::DuplicateRoot { root: node }));
    }

    #[test]
    fn validate_accepts_a_palette_with_no_properties() {
        let mut state = VoxMain::default();
        // A palette with no properties still carries materials; each
        // row is empty and every property resolves to its default. Voxels
        // sample them like any other material.
        let mut palette = VoxPalette::default();
        palette.add_material(vec![]).unwrap();
        let second = palette.add_material(vec![]).unwrap();
        let palette = state.add_palette(palette);

        let mut object = unit_object("o");
        object.add_layer(palette, second);
        state.add_object(object);
        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn validate_rejects_a_palette_without_materials() {
        let mut state = VoxMain::default();
        // Every palette is sampled, so even a property-less palette needs a
        // material.
        let id = state.add_palette(VoxPalette::default());
        assert_eq!(
            state.validate(),
            Err(Error::PaletteWithoutMaterials { palette: id })
        );
    }

    #[test]
    fn validate_rejects_a_dangling_layer_palette() {
        let mut state = VoxMain::default();
        let mut object = unit_object("o");
        // Reference palette id 0, but the state has no palettes.
        object.add_layer(palette(0), material(0));
        let id = state.add_object(object);

        assert_eq!(
            state.validate(),
            Err(Error::PaletteRef {
                object: id,
                palette: palette(0),
            })
        );
    }

    #[test]
    fn validate_rejects_a_bad_sample_material() {
        let mut state = VoxMain::default();
        let pool = int_pool(&mut state, vec![7]);
        let palette = state.add_palette(one_material_palette(pool, 0));

        // The layer back-fills the live voxel with material 9, beyond the
        // palette's one material.
        let mut object = unit_object("o");
        object.add_layer(palette, material(9));
        let id = state.add_object(object);

        assert_eq!(
            state.validate(),
            Err(Error::SampleMaterial {
                object: id,
                voxel: voxel(0),
                material: material(9),
            })
        );
    }

    #[test]
    fn validate_rejects_dangling_child() {
        let mut state = VoxMain::default();
        state.add_hierarchy_node(node_with_children(vec![node_id(9)]));
        assert!(matches!(
            state.validate(),
            Err(Error::ChildNode { child, .. }) if child == node_id(9)
        ));
    }

    #[test]
    fn validate_rejects_dangling_root() {
        let mut state = VoxMain::default();
        state.add_hierarchy_node(VoxHierarchyNode::default());
        state.set_root_hierarchy_nodes(vec![node_id(7)]);
        assert_eq!(state.validate(), Err(Error::Root { root: node_id(7) }));
    }

    #[test]
    fn validate_rejects_a_cycle() {
        let mut state = VoxMain::default();
        // node 0 -> child 1, node 1 -> child 0.
        state.add_hierarchy_node(node_with_children(vec![node_id(1)]));
        state.add_hierarchy_node(node_with_children(vec![node_id(0)]));
        assert!(matches!(state.validate(), Err(Error::Cycle { .. })));
    }

    #[test]
    fn validate_rejects_a_zero_scale() {
        let mut state = VoxMain::default();
        let mut node = VoxHierarchyNode::default();
        node.transform.scale = TyVector3::new(1.0, 0.0, 1.0);
        let id = state.add_hierarchy_node(node);
        assert_eq!(state.validate(), Err(Error::ZeroScale { node: id }));
    }

    #[test]
    fn validate_rejects_a_non_finite_scale() {
        let mut state = VoxMain::default();
        let mut node = VoxHierarchyNode::default();
        // NaN slips past the zero-scale check (NaN == 0.0 is false), so the
        // finiteness check must catch it first.
        node.transform.scale = TyVector3::new(1.0, f64::NAN, 1.0);
        let id = state.add_hierarchy_node(node);
        assert_eq!(
            state.validate(),
            Err(Error::NonFiniteTransform { node: id })
        );
    }

    #[test]
    fn validate_rejects_a_non_finite_position() {
        let mut state = VoxMain::default();
        let mut node = VoxHierarchyNode::default();
        node.transform.position = TyVector3::new(0.0, 0.0, f64::INFINITY);
        let id = state.add_hierarchy_node(node);
        assert_eq!(
            state.validate(),
            Err(Error::NonFiniteTransform { node: id })
        );
    }

    #[test]
    fn validate_rejects_a_non_unit_rotation() {
        let mut state = VoxMain::default();
        let mut node = VoxHierarchyNode::default();
        // Length squared 4, well outside the unit tolerance.
        node.transform.rotation = TyQuaternion::from_xyzw(0.0, 0.0, 0.0, 2.0);
        let id = state.add_hierarchy_node(node);
        assert_eq!(state.validate(), Err(Error::NonUnitRotation { node: id }));
    }

    #[test]
    fn clone_state_is_an_independent_deep_copy() {
        let mut state = VoxMain::default();
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
        let mut state = VoxMain::default();
        let pool_a = int_pool(&mut state, vec![10]);
        let pool_b = int_pool(&mut state, vec![20]);
        let palette_a = state.add_palette(one_material_palette(pool_a, 0));
        let palette_b = state.add_palette(one_material_palette(pool_b, 0));

        let mut a = unit_object("a");
        a.add_layer(palette_a, material(0));
        let object_a = state.add_object(a);

        let mut b = unit_object("b");
        b.add_layer(palette_b, material(0));
        let voxel = b.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        b.retain_voxel(voxel, &[material(0)]).unwrap();
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
        let property = U32Id::<BVoxProperty>::from_u32(0);
        assert_eq!(state.object(object).unwrap().name(), "b");
        // Material 0 resolves through property 0 to pool B's value 20.
        let (pool, index) = state
            .material_value(palette, material(0), property)
            .unwrap();
        assert_eq!(pool.value(index), Some(VoxPoolValueRef::Int(20)));
        assert_eq!(
            state
                .object(object)
                .unwrap()
                .iter_layers()
                .collect::<Vec<_>>(),
            [(U32Id::<BVoxLayer>::from_u32(0), palette)]
        );
        assert_eq!(
            state
                .object(object)
                .unwrap()
                .voxel_material(voxel, U32Id::<BVoxLayer>::from_u32(0)),
            Some(material(0))
        );

        // The inner node dropped `a` and renumbered `b` to 0; the roots are
        // intact.
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
        // removed entities map to None, survivors to their compacted ids. Value
        // pools are never removed, so both map to themselves.
        assert_eq!(remap.objects.new_id(object_a), None);
        assert_eq!(remap.objects.new_id(object_b), Some(object));
        assert_eq!(remap.palettes.new_id(palette_a), None);
        assert_eq!(remap.palettes.new_id(palette_b), Some(palette));
        assert!(remap.materials[palette_a.to_usize_id()].is_empty());
        assert_eq!(
            remap.materials[palette_b.to_usize_id()].new_id(material(0)),
            Some(material(0))
        );
        assert_eq!(remap.value_pools.new_id(pool_a), Some(pool_a));
        assert_eq!(remap.value_pools.new_id(pool_b), Some(pool_b));
    }

    #[test]
    fn remove_hierarchy_node_detaches_children_and_roots() {
        let mut state = VoxMain::default();
        let leaf = state.add_hierarchy_node(VoxHierarchyNode::default());
        let mid = state.add_hierarchy_node(node_with_children(vec![leaf]));
        let top = state.add_hierarchy_node(node_with_children(vec![mid, leaf]));
        state.set_root_hierarchy_nodes(vec![top, mid]);

        assert_eq!(state.remove_hierarchy_node(mid), Some(()));
        assert_eq!(state.remove_hierarchy_node(mid), None); // already gone

        // `mid` is detached from `top` and the roots; the shared `leaf`
        // survives.
        assert_eq!(state.hierarchy_node(top).unwrap().child_nodes, [leaf]);
        assert_eq!(state.root_hierarchy_nodes(), [top]);
        assert!(state.hierarchy_node(mid).is_none());
        assert!(state.hierarchy_node(leaf).is_some());
        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn remove_material_repaints_live_voxels_onto_the_replacement() {
        let mut state = VoxMain::default();
        let pool = int_pool(&mut state, vec![0, 1]);
        let mut palette = VoxPalette::default();
        palette.add_property("v".to_owned(), pool).unwrap();
        let keep = palette.add_material(vec![value(0)]).unwrap();
        let drop = palette.add_material(vec![value(1)]).unwrap();
        let palette = state.add_palette(palette);

        let mut object = unit_object("o");
        let layer = object.add_layer(palette, keep);
        let voxel = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel, &[drop]).unwrap();
        let object = state.add_object(object);

        // Removing `drop` repaints the voxel that used it onto `keep`.
        assert_eq!(state.remove_material(palette, drop, keep), Some(()));
        assert_eq!(state.validate(), Ok(()));
        assert_eq!(
            state.object(object).unwrap().voxel_material(voxel, layer),
            Some(keep)
        );
        assert!(!state.palette(palette).unwrap().contains_material(drop));

        // A no-op replacement and unknown ids are rejected.
        assert_eq!(state.remove_material(palette, keep, keep), None);
        assert_eq!(state.remove_material(palette, drop, keep), None); // drop gone

        state.gc();
        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn validate_and_gc_handle_a_high_id_sample_after_a_material_hole() {
        let mut state = VoxMain::default();
        let pool = int_pool(&mut state, vec![0, 1, 2]);
        let mut palette = VoxPalette::default();
        palette.add_property("v".to_owned(), pool).unwrap();
        let first = palette.add_material(vec![value(0)]).unwrap();
        let second = palette.add_material(vec![value(1)]).unwrap();
        let third = palette.add_material(vec![value(2)]).unwrap();
        let palette = state.add_palette(palette);

        let mut object = unit_object("o");
        let layer = object.add_layer(palette, first);
        let voxel = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel, &[third]).unwrap(); // samples the highest id
        let object = state.add_object(object);

        // Remove `first`; no live voxel used it, so the repaint is a no-op. The
        // palette is now holed: the voxel still samples `third`, whose id
        // exceeds the live material count. A range check would wrongly reject
        // this; the retention check accepts it.
        assert_eq!(state.remove_material(palette, first, second), Some(()));
        assert_eq!(state.validate(), Ok(()));

        state.gc();
        assert_eq!(state.validate(), Ok(()));
        // gc preserves which material the voxel samples: still the value-2
        // material, just renumbered.
        let sampled = state
            .object(object)
            .unwrap()
            .voxel_material(voxel, layer)
            .unwrap();
        let property = U32Id::<BVoxProperty>::from_u32(0);
        let (pool, index) = state.material_value(palette, sampled, property).unwrap();
        assert_eq!(pool.value(index), Some(VoxPoolValueRef::Int(2)));
        assert_eq!(state.palette(palette).unwrap().material_count(), 2);
    }

    #[test]
    fn remove_object_rejects_an_unknown_id() {
        let mut state = VoxMain::default();
        let object = state.add_object(unit_object("o"));
        assert_eq!(state.remove_object(object), Some(()));
        assert_eq!(state.remove_object(object), None);
        assert_eq!(
            state.remove_palette(U32Id::<BVoxPalette>::from_u32(0)),
            None
        );
    }

    #[test]
    fn objects_with_build_volume_margin_validate_and_survive_gc() {
        let mut state = VoxMain::default();
        let a =
            state.add_object(VoxObject::new("a".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap());
        // `b` carries margin: a 5x5x5 build volume with one live voxel off the
        // origin, which the bounds rule allows.
        let mut object_b = VoxObject::new("b".to_owned(), TyVector3U32::new(5, 5, 5)).unwrap();
        let voxel = object_b.voxel_id(TyVector3U32::new(2, 3, 1)).unwrap();
        object_b.retain_voxel(voxel, &[]).unwrap();
        let b = state.add_object(object_b);
        assert_eq!(b.to_u32(), 1);
        assert_eq!(state.validate(), Ok(()));

        // Remove `a` and gc: `b` renumbers to 0, keeping its margin grid and
        // voxel.
        assert_eq!(state.remove_object(a), Some(()));
        state.gc();
        let b0 = U32Id::<BVoxObject>::from_u32(0);
        let object = state.object(b0).unwrap();
        assert_eq!(object.name(), "b");
        assert_eq!(object.bounds(), TyVector3U32::new(5, 5, 5));
        assert_eq!(
            object.live_extent(),
            Some((TyVector3U32::new(2, 3, 1), TyVector3U32::new(1, 1, 1)))
        );
    }

    /// Two layers referencing the same palette is legal and the layers do not
    /// merge, so `validate` accepts the shape and each layer resolves its own
    /// samples through it.
    #[test]
    fn two_layer_object_sharing_a_palette_gcs_and_resolves() {
        // The Phase 3 gate: build a two-layer object that shares one palette,
        // validate it, gc it, and read a material's resolved values back.
        let mut state = VoxMain::default();
        let colors = state.add_value_pool(VoxValuePool::srgba(vec![
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
        ]));
        let metallic = state.add_value_pool(VoxValuePool::float(
            VoxBound::Number(0.0),
            VoxBound::Number(1.0),
            vec![0.0, 1.0],
        ));
        let mut palette = VoxPalette::default();
        let color = palette
            .add_property("baseColorFactor".to_owned(), colors)
            .unwrap();
        let metal = palette
            .add_property("metallicFactor".to_owned(), metallic)
            .unwrap();
        let matte_red = palette.add_material(vec![value(0), value(0)]).unwrap();
        let shiny_green = palette.add_material(vec![value(1), value(1)]).unwrap();
        let palette = state.add_palette(palette);

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        // Two layers on the same palette; each voxel samples one material per
        // layer.
        let base = object.add_layer(palette, matte_red);
        let overlay = object.add_layer(palette, matte_red);
        let v0 = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        let v1 = object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap();
        object.retain_voxel(v0, &[matte_red, shiny_green]).unwrap();
        object.retain_voxel(v1, &[shiny_green, matte_red]).unwrap();
        state.add_object(object);

        assert_eq!(state.validate(), Ok(()));
        state.gc();
        assert_eq!(state.validate(), Ok(()));

        // Resolve the base-layer material at v0: matte_red draws color id 0
        // (red) and metallic id 0 (0.0).
        let object = state.object(U32Id::<BVoxObject>::from_u32(0)).unwrap();
        let sampled = object.voxel_material(v0, base).unwrap();
        let (pool, index) = state.material_value(palette, sampled, color).unwrap();
        assert_eq!(
            pool.value(index),
            Some(VoxPoolValueRef::Srgba(&[1.0, 0.0, 0.0, 1.0]))
        );
        let (pool, index) = state.material_value(palette, sampled, metal).unwrap();
        assert_eq!(pool.value(index), Some(VoxPoolValueRef::Float(0.0)));

        // The overlay layer at v0 samples shiny_green, drawing color id 1
        // (green), proving the two layers resolve independently.
        let overlay_sampled = object.voxel_material(v0, overlay).unwrap();
        let (pool, index) = state
            .material_value(palette, overlay_sampled, color)
            .unwrap();
        assert_eq!(
            pool.value(index),
            Some(VoxPoolValueRef::Srgba(&[0.0, 1.0, 0.0, 1.0]))
        );
    }

    #[test]
    fn validate_rejects_an_empty_pool() {
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::boolean(vec![]));
        assert_eq!(state.validate(), Err(Error::EmptyPool { pool: pool(0) }));
    }

    #[test]
    fn validate_rejects_unordered_pool_bounds() {
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::float(
            VoxBound::Number(1.0),
            VoxBound::Number(0.0),
            vec![0.5],
        ));
        assert_eq!(state.validate(), Err(Error::PoolBound { pool: pool(0) }));
    }

    #[test]
    fn validate_rejects_a_non_integer_int_bound() {
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::int(
            VoxBound::Number(0.5),
            VoxBound::None,
            vec![1],
        ));
        assert_eq!(state.validate(), Err(Error::PoolBound { pool: pool(0) }));
    }

    #[test]
    fn validate_rejects_a_value_out_of_bounds() {
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::float(
            VoxBound::Number(0.0),
            VoxBound::Number(1.0),
            vec![0.0, 2.0],
        ));
        assert_eq!(
            state.validate(),
            Err(Error::PoolValue {
                pool: pool(0),
                value: value(1),
            })
        );
    }

    #[test]
    fn validate_compares_int_values_against_bounds_exactly() {
        // 2^53 + 1 rounds down to 2^53 as f64, so a float comparison would
        // wrongly accept it against a max of 2^53; the integer-domain
        // comparison rejects it.
        const MAX: i64 = 1 << 53;
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::int(
            VoxBound::None,
            VoxBound::Number(MAX as f64),
            vec![MAX, MAX + 1],
        ));
        assert_eq!(
            state.validate(),
            Err(Error::PoolValue {
                pool: pool(0),
                value: value(1),
            })
        );
    }

    #[test]
    fn validate_rejects_an_srgb_component_out_of_range() {
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::srgb(vec![[0.0, 0.0, 0.0], [0.0, 1.5, 0.0]]));
        assert_eq!(
            state.validate(),
            Err(Error::PoolValue {
                pool: pool(0),
                value: value(1),
            })
        );
    }

    #[test]
    fn validate_accepts_hdr_linear_components() {
        let mut state = VoxMain::default();
        // Linear colors allow components above 1 for HDR.
        state.add_value_pool(VoxValuePool::linear_rgb(vec![[0.0, 4.0, 12.0]]));
        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn validate_rejects_a_negative_linear_component() {
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::linear_rgba(vec![[0.0, -0.1, 0.0, 1.0]]));
        assert_eq!(
            state.validate(),
            Err(Error::PoolValue {
                pool: pool(0),
                value: value(0),
            })
        );
    }

    #[test]
    fn validate_rejects_a_non_finite_linear_component() {
        let mut state = VoxMain::default();
        // A linear pool is only lower-bounded, so `+Infinity` would pass a bare
        // `>= 0` test; the finiteness guard must reject it. The wire has no
        // Infinity, so such a value could never round-trip.
        state.add_value_pool(VoxValuePool::linear_rgb(vec![[0.0, f64::INFINITY, 0.0]]));
        assert_eq!(
            state.validate(),
            Err(Error::PoolValue {
                pool: pool(0),
                value: value(0),
            })
        );
    }

    #[test]
    fn validate_rejects_a_dangling_property_pool() {
        let mut state = VoxMain::default();
        let mut palette = VoxPalette::default();
        // The property references pool id 0, but the state holds no pools.
        let property = palette
            .add_property("baseColorFactor".to_owned(), pool(0))
            .unwrap();
        let palette = state.add_palette(palette);
        assert_eq!(
            state.validate(),
            Err(Error::PropertyPool {
                palette,
                property,
                pool: pool(0),
            })
        );
    }

    #[test]
    fn validate_rejects_a_material_value_id_not_in_the_pool() {
        let mut state = VoxMain::default();
        let pool = int_pool(&mut state, vec![0, 1]);
        let mut palette = VoxPalette::default();
        let property = palette.add_property("v".to_owned(), pool).unwrap();
        // Two pool values, but this material draws value id 2.
        let material = palette.add_material(vec![value(2)]).unwrap();
        let palette = state.add_palette(palette);
        assert_eq!(
            state.validate(),
            Err(Error::MaterialValue {
                palette,
                property,
                material,
            })
        );
    }

    #[test]
    fn validate_reports_a_stale_value_id_after_a_missed_rewrite() {
        let mut state = VoxMain::default();
        let pool = int_pool(&mut state, vec![10, 20]);
        let mut palette = VoxPalette::default();
        let property = palette.add_property("v".to_owned(), pool).unwrap();
        let material = palette.add_material(vec![value(1)]).unwrap();
        let palette = state.add_palette(palette);
        state.validate().unwrap();

        // Release the drawn value directly, skipping the cell rewrite
        // remove_pool_value performs, so the material's cell holds a stale id.
        // Safety: the pool id is retained.
        let pool_ref = unsafe { state.runtime_state.value_pools.get_mut(pool) };
        pool_ref.release_value_stable(value(1));

        assert_eq!(
            state.validate(),
            Err(Error::MaterialValue {
                palette,
                property,
                material,
            })
        );
    }

    #[test]
    fn remove_object_preserves_the_survivors_order() {
        let mut state = VoxMain::default();
        let a = state.add_object(unit_object("a"));
        let b = state.add_object(unit_object("b"));
        let c = state.add_object(unit_object("c"));

        // Removing the first of three is the smallest case a swap-remove would
        // get wrong, listing "c" before "b".
        assert_eq!(state.remove_object(a), Some(()));
        let names: Vec<&str> = state.iter_objects().map(|(_, o)| o.name()).collect();
        assert_eq!(names, ["b", "c"]);

        // An object added after the removal recycles the freed id but appends
        // at the end of the order.
        let d = state.add_object(unit_object("d"));
        assert_eq!(d, a);
        let names: Vec<&str> = state.iter_objects().map(|(_, o)| o.name()).collect();
        assert_eq!(names, ["b", "c", "d"]);
        assert_eq!(state.object_index(b), Some(0));
        assert_eq!(state.object_index(c), Some(1));
        assert_eq!(state.object_index(d), Some(2));
    }

    #[test]
    fn remove_palette_preserves_the_survivors_order() {
        let mut state = VoxMain::default();
        let a = state.add_palette(VoxPalette::default());
        let b = state.add_palette(VoxPalette::default());
        let c = state.add_palette(VoxPalette::default());

        // Removing the first of three is the smallest case a swap-remove would
        // get wrong, listing `c` before `b`.
        assert_eq!(state.remove_palette(a), Some(()));
        assert_eq!(
            state.iter_palettes().map(|(id, _)| id).collect::<Vec<_>>(),
            [b, c]
        );
    }

    #[test]
    fn remove_palette_detaches_every_layer_drawing_it() {
        let mut state = VoxMain::default();
        let pool = int_pool(&mut state, vec![10, 20]);
        let a = state.add_palette(two_material_palette(pool));
        let b = state.add_palette(two_material_palette(pool));
        let c = state.add_palette(two_material_palette(pool));

        // Two of the four layers draw `a`, so the detach has to remove both.
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        object.add_layer(a, material(0));
        let on_b = object.add_layer(b, material(0));
        object.add_layer(a, material(0));
        let on_c = object.add_layer(c, material(0));

        // Each layer samples a different material per voxel, so a detach that
        // drops the wrong sample column shows up below.
        let first = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        let second = object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap();
        object
            .retain_voxel(first, &[material(0), material(1), material(0), material(0)])
            .unwrap();
        object
            .retain_voxel(
                second,
                &[material(1), material(0), material(1), material(1)],
            )
            .unwrap();
        let object_id = state.add_object(object);
        state.validate().unwrap();

        assert_eq!(state.remove_palette(a), Some(()));
        state.validate().unwrap();

        // Both layers on `a` are gone and the survivors keep their order.
        assert_eq!(
            state.iter_palettes().map(|(id, _)| id).collect::<Vec<_>>(),
            [b, c]
        );
        let object_ref = state.object(object_id).unwrap();
        assert_eq!(
            object_ref.iter_layers().collect::<Vec<_>>(),
            [(on_b, b), (on_c, c)]
        );

        // Each surviving layer kept its own samples.
        assert_eq!(object_ref.voxel_material(first, on_b), Some(material(1)));
        assert_eq!(object_ref.voxel_material(second, on_b), Some(material(0)));
        assert_eq!(object_ref.voxel_material(first, on_c), Some(material(0)));
        assert_eq!(object_ref.voxel_material(second, on_c), Some(material(1)));

        state.gc();
        state.validate().unwrap();
    }

    #[test]
    fn remove_hierarchy_node_preserves_the_survivors_order() {
        let mut state = VoxMain::default();
        let a = state.add_hierarchy_node(VoxHierarchyNode::default());
        let b = state.add_hierarchy_node(VoxHierarchyNode::default());
        let c = state.add_hierarchy_node(VoxHierarchyNode::default());

        // Removing the first of three is the smallest case a swap-remove would
        // get wrong, listing `c` before `b`.
        assert_eq!(state.remove_hierarchy_node(a), Some(()));
        assert_eq!(
            state
                .iter_hierarchy_nodes()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            [b, c]
        );
    }

    #[test]
    fn move_object_reorders_the_listing_and_validates() {
        let mut state = VoxMain::default();
        let a = state.add_object(unit_object("a"));
        let b = state.add_object(unit_object("b"));
        let c = state.add_object(unit_object("c"));

        assert_eq!(state.move_object(a, 2), Some(()));
        let names: Vec<&str> = state.iter_objects().map(|(_, o)| o.name()).collect();
        assert_eq!(names, ["b", "c", "a"]);
        assert_eq!(state.object_index(a), Some(2));

        // An out-of-range index and an unknown id are rejected.
        assert_eq!(state.move_object(a, 3), None);
        state.remove_object(b).unwrap();
        assert_eq!(state.move_object(b, 0), None);
        assert_eq!(state.object_index(b), None);
        assert_eq!(state.object_index(c), Some(0));
        let names: Vec<&str> = state.iter_objects().map(|(_, o)| o.name()).collect();
        assert_eq!(names, ["c", "a"]);
    }

    #[test]
    fn move_palette_reorders_the_listing_and_validates() {
        let mut state = VoxMain::default();
        let a = state.add_palette(VoxPalette::default());
        let b = state.add_palette(VoxPalette::default());

        assert_eq!(state.move_palette(b, 0), Some(()));
        assert_eq!(
            state.iter_palettes().map(|(id, _)| id).collect::<Vec<_>>(),
            [b, a]
        );
        assert_eq!(state.palette_index(b), Some(0));

        // An out-of-range index and an unknown id are rejected.
        assert_eq!(state.move_palette(b, 2), None);
        assert_eq!(state.move_palette(U32Id::from_u32(9), 0), None);
        assert_eq!(state.palette_index(U32Id::from_u32(9)), None);
    }

    #[test]
    fn move_value_pool_reorders_the_listing_and_validates() {
        let mut state = VoxMain::default();
        let a = int_pool(&mut state, vec![1]);
        let b = int_pool(&mut state, vec![2]);

        assert_eq!(state.move_value_pool(b, 0), Some(()));
        assert_eq!(
            state
                .iter_value_pools()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            [b, a]
        );
        assert_eq!(state.value_pool_index(b), Some(0));

        // An out-of-range index and an unknown id are rejected.
        assert_eq!(state.move_value_pool(b, 2), None);
        assert_eq!(state.move_value_pool(U32Id::from_u32(9), 0), None);
        assert_eq!(state.value_pool_index(U32Id::from_u32(9)), None);
    }

    #[test]
    fn remove_pool_value_repoints_cells_preserves_order_and_validates() {
        let mut state = VoxMain::default();
        let ints = int_pool(&mut state, vec![10, 20, 30]);
        // Two palettes draw the doomed value, so both must be repointed.
        let a = one_material_palette(ints, 0);
        let a_id = state.add_palette(a);
        let mut b = VoxPalette::default();
        let b_property = b.add_property("v".to_owned(), ints).unwrap();
        let b_doomed = b.add_material(vec![value(0)]).unwrap();
        let b_last = b.add_material(vec![value(2)]).unwrap();
        let b_id = state.add_palette(b);
        state.validate().unwrap();

        // Removing the first of three is the smallest case a swap-remove would
        // get wrong, listing 30 before 20.
        assert_eq!(state.remove_pool_value(ints, value(0), value(2)), Some(()));

        // Every cell that drew 10 now draws 30, and the survivors keep their
        // order and ids.
        let a_property = U32Id::<BVoxProperty>::from_u32(0);
        let a_material = U32Id::<BVoxMaterial>::from_u32(0);
        assert_eq!(
            state
                .palette(a_id)
                .unwrap()
                .value_id(a_material, a_property),
            Some(value(2))
        );
        let b_ref = state.palette(b_id).unwrap();
        assert_eq!(b_ref.value_id(b_doomed, b_property), Some(value(2)));
        assert_eq!(b_ref.value_id(b_last, b_property), Some(value(2)));
        assert_eq!(
            state.value_pool(ints),
            Some(&VoxValuePool::int(
                VoxBound::None,
                VoxBound::None,
                vec![20, 30]
            ))
        );
        state.validate().unwrap();

        // A repeated id, an id not the pool's, a released id, and an unknown
        // pool all reject.
        assert_eq!(state.remove_pool_value(ints, value(1), value(1)), None);
        assert_eq!(state.remove_pool_value(ints, value(9), value(1)), None);
        assert_eq!(state.remove_pool_value(ints, value(1), value(0)), None);
        assert_eq!(
            state.remove_pool_value(U32Id::from_u32(9), value(1), value(2)),
            None
        );
        state.validate().unwrap();
    }

    #[test]
    fn gc_after_moves_renumbers_to_listing_order() {
        let mut state = VoxMain::default();
        let ints = int_pool(&mut state, vec![1, 2]);
        let mut palette = VoxPalette::default();
        let property = palette.add_property("v".to_owned(), ints).unwrap();
        let one = palette.add_material(vec![value(0)]).unwrap();
        let two = palette.add_material(vec![value(1)]).unwrap();
        let palette_id = state.add_palette(palette);
        let object_a = state.add_object(unit_object("a"));
        let object_b = state.add_object(unit_object("b"));
        state.validate().unwrap();

        // List the value holding 2 first and object b first.
        state
            .reorder_value_pool(ints, &[value(1), value(0)])
            .unwrap();
        state.move_object(object_b, 0).unwrap();
        state.validate().unwrap();

        let remap = state.gc();
        state.validate().unwrap();

        // Everything renumbers to listing order: the value holding 2 is now id
        // 0, object b is id 0, and every id equals its listing index.
        let names: Vec<&str> = state.iter_objects().map(|(_, o)| o.name()).collect();
        assert_eq!(names, ["b", "a"]);
        assert_eq!(remap.objects.new_id(object_b), Some(U32Id::from_u32(0)));
        assert_eq!(remap.objects.new_id(object_a), Some(U32Id::from_u32(1)));
        assert_eq!(
            state
                .iter_objects()
                .map(|(id, _)| id.to_u32())
                .collect::<Vec<_>>(),
            [0, 1]
        );

        // The value remap is indexed by the pool's old id. The value holding 2
        // moved from id 1 to id 0.
        assert_eq!(
            remap.pool_values[ints.to_usize_id()].new_id(value(1)),
            Some(value(0))
        );
        assert_eq!(
            remap.pool_values[ints.to_usize_id()].new_id(value(0)),
            Some(value(1))
        );
        let pool = state.value_pool(ints).unwrap();
        assert_eq!(pool.value(value(0)), Some(VoxPoolValueRef::Int(2)));
        assert_eq!(pool.value(value(1)), Some(VoxPoolValueRef::Int(1)));
        assert_eq!(
            pool.iter_values()
                .map(|(id, _)| id.to_u32())
                .collect::<Vec<_>>(),
            [0, 1]
        );

        // The material cells followed the value renumbering, so each still
        // resolves to its number.
        let palette_ref = state.palette(palette_id).unwrap();
        assert_eq!(palette_ref.value_id(one, property), Some(value(1)));
        assert_eq!(palette_ref.value_id(two, property), Some(value(0)));
    }

    #[test]
    fn gc_after_a_pool_move_relabels_pools_and_each_pool_s_values() {
        let mut state = VoxMain::default();
        let first_pool = int_pool(&mut state, vec![10, 20]);
        let second_pool = int_pool(&mut state, vec![30, 40, 50]);
        let mut palette = VoxPalette::default();
        // Both properties come before the material, so neither is back-filled.
        let first = palette
            .add_property("first".to_owned(), first_pool)
            .unwrap();
        let second = palette
            .add_property("second".to_owned(), second_pool)
            .unwrap();
        let material = palette.add_material(vec![value(1), value(0)]).unwrap();
        let palette_id = state.add_palette(palette);
        state.validate().unwrap();

        // Move the second pool ahead of the first, so the pool relabel is not
        // the identity, and give the two pools different value permutations, so
        // a cell relabeled through the wrong pool's remap lands on the wrong
        // value.
        state.move_value_pool(second_pool, 0).unwrap();
        state
            .reorder_value_pool(first_pool, &[value(1), value(0)])
            .unwrap();
        state
            .reorder_value_pool(second_pool, &[value(2), value(0), value(1)])
            .unwrap();
        state.validate().unwrap();

        let remap = state.gc();
        state.validate().unwrap();

        // The pools renumber to listing order, so the moved pool is now id 0.
        assert_eq!(remap.value_pools.new_id(second_pool), Some(pool(0)));
        assert_eq!(remap.value_pools.new_id(first_pool), Some(pool(1)));
        assert_eq!(
            state
                .iter_value_pools()
                .map(|(id, _)| id.to_u32())
                .collect::<Vec<_>>(),
            [0, 1]
        );

        // Each pool's value remap is keyed by that pool's pre-gc id, which is
        // the id the moved pool held before the relabel above.
        assert_eq!(
            remap.pool_values[first_pool.to_usize_id()].new_id(value(1)),
            Some(value(0))
        );
        assert_eq!(
            remap.pool_values[second_pool.to_usize_id()].new_id(value(2)),
            Some(value(0))
        );

        // Every property followed the pool renumbering.
        let palette_ref = state.palette(palette_id).unwrap();
        assert_eq!(palette_ref.property(first).unwrap().pool, pool(1));
        assert_eq!(palette_ref.property(second).unwrap().pool, pool(0));

        // The material still reads the same two numbers, through the relabeled
        // pool ids and the relabeled cells.
        let (pool_ref, value_id) = state.material_value(palette_id, material, first).unwrap();
        assert_eq!(pool_ref.value(value_id), Some(VoxPoolValueRef::Int(20)));
        let (pool_ref, value_id) = state.material_value(palette_id, material, second).unwrap();
        assert_eq!(pool_ref.value(value_id), Some(VoxPoolValueRef::Int(30)));
    }
}
