use crate::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxPaletteBinding, BVoxValuePool,
    Error, Result, VoxBound, VoxGcRemap, VoxHierarchyNode, VoxObject, VoxPalette, VoxRuntimeState,
    VoxValue, VoxValuePool,
};
use branded_id::{IdVec, U32Id, soa::IdRemap};
use std::collections::{BTreeSet, HashMap, HashSet};

/// The in-memory state of a voxel model: its objects, shared palettes, scene
/// hierarchy, and roots.
///
/// Add entities with [`add_object`](Self::add_object),
/// [`add_palette`](Self::add_palette), and
/// [`add_hierarchy_node`](Self::add_hierarchy_node), and read them back by id
/// or through the `iter_*` methods. Ids are bare indices into this state,
/// meaningful only within it. [`validate`](Self::validate) checks the
/// cross-references.
#[derive(Debug, Default)]
pub struct VoxMain {
    /// The runtime scene: objects.
    runtime_state: VoxRuntimeState,

    /// Optional user-extension namespace; the core format assigns it no
    /// meaning.
    ext: Option<VoxValue>,
}

impl VoxMain {
    /// Adds an object, returning its id (its listing index).
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

    /// Objects in id order, as `(id, object)`.
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

    /// Adds a shared value pool, returning its id (its listing index).
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

    /// Value pools in id order, as `(id, pool)`.
    pub fn iter_value_pools(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxValuePool>, &VoxValuePool)> + '_ {
        // Safety: retained ids have a value.
        self.runtime_state
            .value_pool_ids
            .iter()
            .map(move |id| (id, unsafe { self.runtime_state.value_pools.get(id) }))
    }

    /// Resolves what `material` in `palette` draws for `binding`: the value pool
    /// it is bound to and the value-index into that pool. `None` if any id is
    /// not this state's, `binding` is not `palette`'s, or the binding names a
    /// pool this state does not hold. Read the value at the index out of the
    /// returned pool by the pool's kind.
    pub fn material_value(
        &self,
        palette: U32Id<BVoxPalette>,
        material: U32Id<BVoxMaterial>,
        binding: U32Id<BVoxPaletteBinding>,
    ) -> Option<(&VoxValuePool, u32)> {
        let palette = self.palette(palette)?;
        let index = palette.value_index(material, binding)?;
        let pool = self.value_pool(palette.binding(binding)?.pool)?;
        Some((pool, index))
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
        // Safety: a retained object id has a value.
        unsafe { self.runtime_state.objects.release(id) };
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
            object.remove_layers_to(id);
        }
        // Safety: a retained palette id has a value; its Drop frees its cells.
        unsafe { self.runtime_state.palettes.release(id) };
        self.runtime_state.palette_ids.release(id);
        Some(())
    }

    /// Removes hierarchy node `id`, detaching it from every `child_nodes` list
    /// and from the roots. Its own children keep any other parents (the
    /// hierarchy is a DAG). `None`, changing nothing, if `id` is not one of
    /// this state's nodes. Leaves a hole until [`gc`](Self::gc) renumbers.
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

    /// Removes `material` from `palette`, first repainting every live voxel that
    /// samples it onto `replacement` so no voxel is left without a material.
    /// `None`, changing nothing, if `palette` is not one of this state's
    /// palettes, if `material` or `replacement` is not one of that palette's
    /// materials, or if `replacement` is `material` itself. Leaves a hole until
    /// [`gc`](Self::gc) renumbers.
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

    /// Compacts every id pool back to a contiguous `0..len` and rewrites every
    /// cross-reference to match, so a state edited by removals numbers its
    /// entities the way a freshly loaded one does and voxsmith saves stay
    /// deterministic (id equals listing index). Call this after any removal and
    /// before saving.
    ///
    /// Requires a referentially valid state, which the `remove_*` methods
    /// preserve (each detaches what it removes) and
    /// [`validate`](Self::validate) checks. The voxel grids are dense and never
    /// compacted, so voxel ids keep equaling their raster index.
    ///
    /// Returns the [`VoxGcRemap`] recording where each id moved, so any ids
    /// held outside the state can be translated to their compacted values.
    pub fn gc(&mut self) -> VoxGcRemap {
        // Compact the shared value-pool store first, then relabel every palette
        // binding's pool ref, so the pool ids are settled before palettes are
        // compacted. With no pool-removal path today the store is already
        // contiguous, so this relabel is currently the identity; it keeps gc
        // uniformly compacting every pool and covers a future removal.
        let pool_remap = self.runtime_state.value_pool_ids.gc();
        // Safety: the value-pool column was in sync with the pre-gc pool, and
        // nothing has retained or released since.
        unsafe { self.runtime_state.value_pools.gc(&pool_remap) };

        // Compact each palette's own pools, so the material relabelings are
        // ready when object samples are translated below. They are indexed by
        // old palette id, so the column covers the palette pool's whole id
        // space.
        let palette_id_space = self.runtime_state.palette_ids.peek_next_fresh().to_u32() as usize;
        let mut material_remaps: IdVec<BVoxPalette, IdRemap<BVoxMaterial, u32>> =
            IdVec::from_vec((0..palette_id_space).map(|_| IdRemap::default()).collect());
        for palette_id in self.runtime_state.palette_ids.iter().collect::<Vec<_>>() {
            // Safety: retained palette ids have a value.
            let palette = unsafe { self.runtime_state.palettes.get_mut(palette_id) };
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
            objects: object_remap,
            palettes: palette_remap,
            hierarchy_nodes: node_remap,
            materials: material_remaps,
        }
    }

    /// Drops value-pool entries no material references, renumbers the survivors
    /// densely from `0`, and rewrites the value-indices at them. The pool-value
    /// counterpart to [`gc`](Self::gc), which compacts the id pools but not the
    /// values inside a pool.
    ///
    /// 1. references union across palettes, so a shared entry survives while any
    ///    one material uses it;
    /// 2. a pool no material references is left whole, since
    ///    [`validate`](Self::validate) requires every pool non-empty;
    /// 3. the state stays referentially valid.
    pub fn prune_value_pools(&mut self) {
        // The value-indices each pool still has a material referencing,
        // ascending so the kept order stays stable.
        let pool_ids: Vec<_> = self.runtime_state.value_pool_ids.iter().collect();
        let mut referenced: HashMap<U32Id<BVoxValuePool>, BTreeSet<u32>> =
            pool_ids.iter().map(|&id| (id, BTreeSet::new())).collect();

        let palette_ids: Vec<_> = self.runtime_state.palette_ids.iter().collect();
        for &palette_id in &palette_ids {
            // Safety: retained palette ids have a value.
            let palette = unsafe { self.runtime_state.palettes.get(palette_id) };
            for (binding_id, binding) in palette.iter_bindings() {
                let uses = referenced
                    .get_mut(&binding.pool)
                    .expect("a binding names a live value pool in a valid state");
                for material in palette.iter_materials() {
                    let index = palette
                        .value_index(material, binding_id)
                        .expect("a retained material has a value-index for every binding");
                    uses.insert(index);
                }
            }
        }

        // Prune each pool with unreferenced entries, recording where its
        // survivors moved. A pool no material references is left whole.
        let mut remaps: HashMap<U32Id<BVoxValuePool>, Vec<u32>> = HashMap::new();
        for &pool_id in &pool_ids {
            let keep: Vec<u32> = referenced[&pool_id].iter().copied().collect();
            // Safety: retained pool ids have a value.
            let pool = unsafe { self.runtime_state.value_pools.get_mut(pool_id) };
            if keep.is_empty() || keep.len() == pool.values_len() {
                continue;
            }

            let mut remap = vec![0u32; pool.values_len()];
            for (new_index, &old_index) in keep.iter().enumerate() {
                remap[old_index as usize] = new_index as u32;
            }
            pool.retain_values(&keep);
            remaps.insert(pool_id, remap);
        }

        if remaps.is_empty() {
            return;
        }

        // Follow the new numbering in every material that drew on a pruned pool.
        for &palette_id in &palette_ids {
            // Safety: retained palette ids have a value.
            let palette = unsafe { self.runtime_state.palettes.get_mut(palette_id) };
            for (pool_id, remap) in &remaps {
                palette.remap_pool_value_indices(*pool_id, remap);
            }
        }
    }

    /// Reorders `pool`'s values to `new_order` and rewrites every material
    /// value-index that draws on it, so values move without changing what any
    /// material resolves to. `new_order[new_index]` is the old index landing at
    /// `new_index`, and must be a permutation of the pool's `0..values_len`. A
    /// no-op if `pool` is not one of this state's.
    pub fn reorder_value_pool(&mut self, pool: U32Id<BVoxValuePool>, new_order: &[u32]) {
        if !self.runtime_state.value_pool_ids.is_retained(pool) {
            return;
        }
        // Safety: the id is retained, so it has a value.
        let values = unsafe { self.runtime_state.value_pools.get_mut(pool) };
        debug_assert!(
            is_permutation(new_order, values.values_len()),
            "reorder_value_pool needs a permutation of the pool's indices"
        );

        // The inverse map from old index to its new slot, applied to every
        // referencing material's value-index.
        let mut remap = vec![0u32; new_order.len()];
        for (new_index, &old_index) in new_order.iter().enumerate() {
            remap[old_index as usize] = new_index as u32;
        }
        values.retain_values(new_order);

        let palette_ids: Vec<_> = self.runtime_state.palette_ids.iter().collect();
        for palette_id in palette_ids {
            // Safety: retained palette ids have a value.
            let palette = unsafe { self.runtime_state.palettes.get_mut(palette_id) };
            palette.remap_pool_value_indices(pool, &remap);
        }
    }

    /// Checks the value pools, palettes, cross-references, and per-entity rules:
    ///
    /// 1. every value pool is non-empty, its values well-formed for its kind,
    ///    and its `min`/`max` finite, integer-valued for an `int` pool, and
    ///    ordered;
    /// 2. every palette binding names a live value pool, no palette binds the
    ///    same attribute twice, and every material value-index is within its
    ///    binding's pool;
    /// 3. every object layer references a live palette (two layers may share
    ///    one), and every live-voxel sample material is within its layer's
    ///    palette;
    /// 4. every node child node and child object resolves, and no node lists
    ///    the same one twice;
    /// 5. every root resolves, and no root repeats;
    /// 6. every node transform has a non-zero scale on each axis and a
    ///    unit-length rotation quaternion within `1e-6`;
    /// 7. the `child_nodes` graph is acyclic.
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
            check_value_pool(pool_id.to_u32(), pool)?;
        }

        // Palette bindings resolve their pools, no palette binds an attribute
        // twice, and every material value-index falls within its binding's
        // pool.
        for (palette_id, palette) in self.iter_palettes() {
            let mut seen_attributes = HashSet::with_capacity(palette.binding_count());
            for (binding_id, binding) in palette.iter_bindings() {
                if !seen_attributes.insert(binding.attribute.as_str()) {
                    return Err(Error::DuplicateBindingAttribute {
                        palette: palette_id.to_u32(),
                        binding: binding_id.to_u32(),
                    });
                }
                let pool = self.value_pool(binding.pool).ok_or(Error::BindingPool {
                    palette: palette_id.to_u32(),
                    binding: binding_id.to_u32(),
                    pool: binding.pool.to_u32(),
                })?;
                for material_id in palette.iter_materials() {
                    let index = palette
                        .value_index(material_id, binding_id)
                        .expect("a material has a value-index for every binding");
                    if index as usize >= pool.values_len() {
                        return Err(Error::MaterialValue {
                            palette: palette_id.to_u32(),
                            binding: binding_id.to_u32(),
                            material: material_id.to_u32(),
                        });
                    }
                }
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
                    object: object_id.to_u32(),
                    palette: palette_id.to_u32(),
                })?;
                layer_palettes.push((layer_id, palette));
            }
            // Every live voxel samples a material within each layer's palette.
            for voxel_id in object.iter_live() {
                for &(layer_id, palette) in &layer_palettes {
                    let material = object
                        .voxel_material(voxel_id, layer_id)
                        .expect("a live voxel has a sample for every layer");
                    if !palette.contains_material(material) {
                        return Err(Error::SampleMaterial {
                            object: object_id.to_u32(),
                            voxel: voxel_id.to_u32(),
                            material: material.to_u32(),
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
            if !rotation.is_normalized(ROTATION_TOLERANCE) {
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
    /// Call only after every child id is known live. Works over the retained
    /// node ids by position, so it holds whether or not the pool has holes.
    fn first_cycle_node(&self) -> Option<u32> {
        const WHITE: u8 = 0;
        const GREY: u8 = 1;
        const BLACK: u8 = 2;

        // Retained node ids and a lookup from id to its position here, so a
        // holed pool (ids not contiguous from zero) is handled the same as a
        // packed one.
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
            // Each frame is a node position plus how many children we have
            // walked.
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

    /// Deep copy. The runtime scene rebuilds its columns against fresh id
    /// pools.
    pub fn clone_state(&self) -> Self {
        Self {
            runtime_state: self.runtime_state.clone_runtime_state(),
            ext: self.ext.clone(),
        }
    }
}

/// Checks a value pool is non-empty and every value and bound is well-formed for
/// its kind: int/float bounds finite, integer-valued for `int`, and ordered;
/// int/float values finite and within bounds; color components within their
/// space's range. `pool_id` is the pool's listing index, for the error.
fn check_value_pool(pool_id: u32, pool: &VoxValuePool) -> Result<()> {
    if pool.values_len() == 0 {
        return Err(Error::EmptyPool { pool: pool_id });
    }
    match pool {
        VoxValuePool::Float { min, max, values } => {
            check_numeric_bounds(pool_id, min, max, false)?;
            for (index, &value) in values.iter().enumerate() {
                if !value.is_finite() || !value_in_bounds(min, max, value) {
                    return Err(Error::PoolValue {
                        pool: pool_id,
                        index: index as u32,
                    });
                }
            }
        }
        VoxValuePool::Int { min, max, values } => {
            check_numeric_bounds(pool_id, min, max, true)?;
            for (index, &value) in values.iter().enumerate() {
                if !value_in_bounds(min, max, value as f64) {
                    return Err(Error::PoolValue {
                        pool: pool_id,
                        index: index as u32,
                    });
                }
            }
        }
        VoxValuePool::Srgb { values } => check_color_components(pool_id, values, false)?,
        VoxValuePool::Srgba { values } => check_color_components(pool_id, values, false)?,
        VoxValuePool::LinearRgb { values } => check_color_components(pool_id, values, true)?,
        VoxValuePool::LinearRgba { values } => check_color_components(pool_id, values, true)?,
        VoxValuePool::Json { .. } | VoxValuePool::Bool { .. } | VoxValuePool::String { .. } => {}
    }
    Ok(())
}

/// Checks a bounded pool's `min`/`max`: each numeric bound is finite (and
/// integer-valued when `integer`), and `min <= max` when both are finite.
fn check_numeric_bounds(pool_id: u32, min: &VoxBound, max: &VoxBound, integer: bool) -> Result<()> {
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
fn bound_number(pool_id: u32, bound: &VoxBound, integer: bool) -> Result<Option<f64>> {
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

/// Checks each color's components lie in its space's range: sRGB in `[0, 1]`,
/// linear finite and `>= 0`. The sRGB range test rejects any non-finite
/// component on its own; the linear side is only lower-bounded, so it guards
/// finiteness explicitly to reject `+Infinity`, which would otherwise pass
/// `>= 0`.
fn check_color_components<const N: usize>(
    pool_id: u32,
    colors: &[[f64; N]],
    linear: bool,
) -> Result<()> {
    for (index, color) in colors.iter().enumerate() {
        for &component in color {
            let in_range = if linear {
                component.is_finite() && component >= 0.0
            } else {
                (0.0..=1.0).contains(&component)
            };
            if !in_range {
                return Err(Error::PoolValue {
                    pool: pool_id,
                    index: index as u32,
                });
            }
        }
    }
    Ok(())
}

/// Whether `order` lists each index in `0..len` exactly once.
fn is_permutation(order: &[u32], len: usize) -> bool {
    if order.len() != len {
        return false;
    }
    let mut seen = vec![false; len];
    for &index in order {
        match seen.get_mut(index as usize) {
            Some(slot) if !*slot => *slot = true,
            _ => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::{
        BVoxHierarchyNode, BVoxLayer, BVoxMaterial, BVoxObject, BVoxPalette, BVoxPaletteBinding,
        BVoxValuePool, Error, VoxBound, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette,
        VoxValuePool,
    };
    use branded_id::U32Id;
    use ty_math::{TyQuaternion, TyVector3, TyVector3U32};

    fn node_id(index: u32) -> U32Id<BVoxHierarchyNode> {
        U32Id::from_u32(index)
    }

    fn material(index: u32) -> U32Id<BVoxMaterial> {
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
        state.add_value_pool(VoxValuePool::Int {
            min: VoxBound::None,
            max: VoxBound::None,
            values,
        })
    }

    /// A palette with one binding "v" to `pool` and one material drawing
    /// value-index `index`.
    fn one_material_palette(pool: U32Id<BVoxValuePool>, index: u32) -> VoxPalette {
        let mut palette = VoxPalette::default();
        palette.add_binding("v".to_owned(), pool);
        palette.add_material(vec![index]).unwrap();
        palette
    }

    #[test]
    fn add_and_read_back_in_id_order() {
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
    fn add_and_read_back_value_pools_in_id_order() {
        let mut state = VoxMain::default();
        let colors = state.add_value_pool(VoxValuePool::Srgba {
            values: vec![[1.0, 0.0, 0.0, 1.0]],
        });
        let metallic = state.add_value_pool(VoxValuePool::Float {
            min: VoxBound::Number(0.0),
            max: VoxBound::Number(1.0),
            values: vec![0.0, 1.0],
        });

        assert_eq!(state.value_pool_count(), 2);
        assert_eq!(colors, U32Id::<BVoxValuePool>::from_u32(0));
        assert_eq!(metallic.to_u32(), 1);
        assert!(matches!(
            state.value_pool(colors),
            Some(VoxValuePool::Srgba { .. })
        ));
        assert_eq!(
            state.value_pool(metallic).map(VoxValuePool::values_len),
            Some(2)
        );
        // An id past the pool is not one of this state's.
        assert_eq!(state.value_pool(U32Id::<BVoxValuePool>::from_u32(2)), None);

        let mut pools = state.iter_value_pools();
        assert!(matches!(
            pools.next(),
            Some((_, VoxValuePool::Srgba { .. }))
        ));
        assert!(matches!(
            pools.next(),
            Some((_, VoxValuePool::Float { .. }))
        ));
        assert!(pools.next().is_none());
    }

    #[test]
    fn clone_state_deep_copies_value_pools() {
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::Int {
            min: VoxBound::None,
            max: VoxBound::None,
            values: vec![7],
        });

        let copy = state.clone_state();
        assert_eq!(copy.value_pool_count(), 1);
        assert_eq!(
            copy.value_pool(U32Id::<BVoxValuePool>::from_u32(0)),
            Some(&VoxValuePool::Int {
                min: VoxBound::None,
                max: VoxBound::None,
                values: vec![7],
            })
        );

        // Mutating the original must not touch the copy.
        state.add_value_pool(VoxValuePool::Bool { values: vec![true] });
        assert_eq!(state.value_pool_count(), 2);
        assert_eq!(copy.value_pool_count(), 1);
    }

    #[test]
    fn prune_value_pools_drops_unreferenced_entries_and_remaps() {
        let mut state = VoxMain::default();
        // Four colors; the palette references only the middle two.
        let colors = state.add_value_pool(VoxValuePool::Srgba {
            values: vec![
                [1.0, 0.0, 0.0, 1.0], // 0 red, unused
                [0.0, 1.0, 0.0, 1.0], // 1 green, used
                [0.0, 0.0, 1.0, 1.0], // 2 blue, unused
                [1.0, 1.0, 1.0, 1.0], // 3 white, used
            ],
        });
        let mut palette = VoxPalette::default();
        let binding = palette.add_binding("baseColorFactor".to_owned(), colors);
        let green = palette.add_material(vec![1]).unwrap();
        let white = palette.add_material(vec![3]).unwrap();
        let palette_id = state.add_palette(palette);
        state.validate().unwrap();

        state.prune_value_pools();

        // The pool keeps green then white in ascending old-index order.
        assert_eq!(
            state.value_pool(colors),
            Some(&VoxValuePool::Srgba {
                values: vec![[0.0, 1.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]],
            })
        );
        // The materials follow the dense numbering: green to 0, white to 1.
        let palette = state.palette(palette_id).unwrap();
        assert_eq!(palette.value_index(green, binding), Some(0));
        assert_eq!(palette.value_index(white, binding), Some(1));
        state.validate().unwrap();
    }

    #[test]
    fn prune_value_pools_keeps_entries_any_palette_still_uses() {
        let mut state = VoxMain::default();
        let ints = int_pool(&mut state, vec![10, 20, 30]);
        // Palette a draws index 0, palette b draws index 2; index 1 is unused.
        let mut a = VoxPalette::default();
        let a_binding = a.add_binding("v".to_owned(), ints);
        let a_material = a.add_material(vec![0]).unwrap();
        let a_id = state.add_palette(a);
        let mut b = VoxPalette::default();
        let b_binding = b.add_binding("v".to_owned(), ints);
        let b_material = b.add_material(vec![2]).unwrap();
        let b_id = state.add_palette(b);
        state.validate().unwrap();

        state.prune_value_pools();

        // 10 and 30 survive (indices 0 and 2 used); 20 (index 1) is dropped.
        assert_eq!(
            state.value_pool(ints),
            Some(&VoxValuePool::Int {
                min: VoxBound::None,
                max: VoxBound::None,
                values: vec![10, 30],
            })
        );
        assert_eq!(
            state
                .palette(a_id)
                .unwrap()
                .value_index(a_material, a_binding),
            Some(0)
        );
        assert_eq!(
            state
                .palette(b_id)
                .unwrap()
                .value_index(b_material, b_binding),
            Some(1)
        );
        state.validate().unwrap();
    }

    #[test]
    fn reorder_value_pool_permutes_values_and_follows_indices() {
        let mut state = VoxMain::default();
        // Three colors; two palettes bind the pool, each with materials pointing
        // at scattered indices.
        let colors = state.add_value_pool(VoxValuePool::Srgba {
            values: vec![
                [1.0, 0.0, 0.0, 1.0], // 0 red
                [0.0, 1.0, 0.0, 1.0], // 1 green
                [0.0, 0.0, 1.0, 1.0], // 2 blue
            ],
        });
        let mut a = VoxPalette::default();
        let a_binding = a.add_binding("baseColorFactor".to_owned(), colors);
        let a_blue = a.add_material(vec![2]).unwrap();
        let a_red = a.add_material(vec![0]).unwrap();
        let a_id = state.add_palette(a);
        let mut b = VoxPalette::default();
        let b_binding = b.add_binding("baseColorFactor".to_owned(), colors);
        let b_green = b.add_material(vec![1]).unwrap();
        let b_id = state.add_palette(b);
        state.validate().unwrap();

        // Move blue to 0, red to 1, green to 2.
        state.reorder_value_pool(colors, &[2, 0, 1]);

        // The pool follows the new order.
        assert_eq!(
            state.value_pool(colors),
            Some(&VoxValuePool::Srgba {
                values: vec![
                    [0.0, 0.0, 1.0, 1.0],
                    [1.0, 0.0, 0.0, 1.0],
                    [0.0, 1.0, 0.0, 1.0]
                ],
            })
        );
        // Every material still resolves to its original color: blue is now 0,
        // red 1, green 2.
        let a = state.palette(a_id).unwrap();
        assert_eq!(a.value_index(a_blue, a_binding), Some(0));
        assert_eq!(a.value_index(a_red, a_binding), Some(1));
        assert_eq!(
            state.palette(b_id).unwrap().value_index(b_green, b_binding),
            Some(2)
        );
        state.validate().unwrap();
    }

    #[test]
    fn prune_value_pools_leaves_a_fully_referenced_pool() {
        let mut state = VoxMain::default();
        let ints = int_pool(&mut state, vec![1, 2]);
        let mut palette = VoxPalette::default();
        palette.add_binding("v".to_owned(), ints);
        palette.add_material(vec![0]).unwrap();
        palette.add_material(vec![1]).unwrap();
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
                node: parent.to_u32(),
                child: leaf.to_u32(),
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
            Err(Error::DuplicateChildObject {
                node: node.to_u32(),
                object: object.to_u32(),
            })
        );
    }

    #[test]
    fn validate_rejects_a_duplicate_root() {
        let mut state = VoxMain::default();
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
    fn validate_accepts_two_layers_sharing_a_palette() {
        let mut state = VoxMain::default();
        let pool = int_pool(&mut state, vec![0]);
        let palette = state.add_palette(one_material_palette(pool, 0));
        let mut object = unit_object("o");
        // Two layers referencing the same palette is legal; layers do not merge.
        // The live voxel keeps the default material 0 in each new layer.
        object.add_layer(palette, material(0));
        object.add_layer(palette, material(0));
        state.add_object(object);
        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn validate_rejects_a_dangling_layer_palette() {
        let mut state = VoxMain::default();
        let mut object = unit_object("o");
        // Reference palette id 0, but the state has no palettes.
        object.add_layer(U32Id::<BVoxPalette>::from_u32(0), material(0));
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
        let mut state = VoxMain::default();
        state.add_hierarchy_node(node_with_children(vec![node_id(9)]));
        assert!(matches!(
            state.validate(),
            Err(Error::ChildNode { child: 9, .. })
        ));
    }

    #[test]
    fn validate_rejects_dangling_root() {
        let mut state = VoxMain::default();
        state.add_hierarchy_node(VoxHierarchyNode::default());
        state.set_root_hierarchy_nodes(vec![node_id(7)]);
        assert_eq!(state.validate(), Err(Error::Root { root: 7 }));
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
    fn validate_rejects_a_duplicate_binding_attribute() {
        let mut state = VoxMain::default();
        let pool = int_pool(&mut state, vec![0]);
        let mut palette = VoxPalette::default();
        palette.add_binding("baseColorFactor".to_owned(), pool);
        palette.add_binding("baseColorFactor".to_owned(), pool);
        state.add_palette(palette);
        assert_eq!(
            state.validate(),
            Err(Error::DuplicateBindingAttribute {
                palette: 0,
                binding: 1,
            })
        );
    }

    #[test]
    fn validate_rejects_a_zero_scale() {
        let mut state = VoxMain::default();
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
        let mut state = VoxMain::default();
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
        let binding = U32Id::<BVoxPaletteBinding>::from_u32(0);
        assert_eq!(state.object(object).unwrap().name(), "b");
        // Material 0 resolves through binding 0 to pool B's value 20.
        match state.material_value(palette, material(0), binding) {
            Some((VoxValuePool::Int { values, .. }, index)) => {
                assert_eq!(values[index as usize], 20)
            }
            other => panic!("unexpected material value {other:?}"),
        }
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
        palette.add_binding("v".to_owned(), pool);
        let keep = palette.add_material(vec![0]).unwrap();
        let drop = palette.add_material(vec![1]).unwrap();
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
        palette.add_binding("v".to_owned(), pool);
        let first = palette.add_material(vec![0]).unwrap();
        let second = palette.add_material(vec![1]).unwrap();
        let third = palette.add_material(vec![2]).unwrap();
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
        let binding = U32Id::<BVoxPaletteBinding>::from_u32(0);
        match state.material_value(palette, sampled, binding) {
            Some((VoxValuePool::Int { values, .. }, index)) => {
                assert_eq!(values[index as usize], 2)
            }
            other => panic!("unexpected material value {other:?}"),
        }
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
        // origin, which the old exact-tight-bounds rule forbade.
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

    #[test]
    fn two_layer_object_sharing_a_palette_gcs_and_resolves() {
        // The Phase 3 gate: build a two-layer object that shares one palette,
        // validate it, gc it, and read a material's resolved values back.
        let mut state = VoxMain::default();
        let colors = state.add_value_pool(VoxValuePool::Srgba {
            values: vec![[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]],
        });
        let metallic = state.add_value_pool(VoxValuePool::Float {
            min: VoxBound::Number(0.0),
            max: VoxBound::Number(1.0),
            values: vec![0.0, 1.0],
        });
        let mut palette = VoxPalette::default();
        let color = palette.add_binding("baseColorFactor".to_owned(), colors);
        let metal = palette.add_binding("metallicFactor".to_owned(), metallic);
        let matte_red = palette.add_material(vec![0, 0]).unwrap();
        let shiny_green = palette.add_material(vec![1, 1]).unwrap();
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

        // Resolve the base-layer material at v0: matte_red draws color index 0
        // (red) and metallic index 0 (0.0).
        let object = state.object(U32Id::<BVoxObject>::from_u32(0)).unwrap();
        let sampled = object.voxel_material(v0, base).unwrap();
        match state.material_value(palette, sampled, color) {
            Some((VoxValuePool::Srgba { values }, index)) => {
                assert_eq!(values[index as usize], [1.0, 0.0, 0.0, 1.0])
            }
            other => panic!("unexpected color {other:?}"),
        }
        match state.material_value(palette, sampled, metal) {
            Some((VoxValuePool::Float { values, .. }, index)) => {
                assert_eq!(values[index as usize], 0.0)
            }
            other => panic!("unexpected metallic {other:?}"),
        }

        // The overlay layer at v0 samples shiny_green, drawing color index 1
        // (green), proving the two layers resolve independently.
        let overlay_sampled = object.voxel_material(v0, overlay).unwrap();
        match state.material_value(palette, overlay_sampled, color) {
            Some((VoxValuePool::Srgba { values }, index)) => {
                assert_eq!(values[index as usize], [0.0, 1.0, 0.0, 1.0])
            }
            other => panic!("unexpected overlay color {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_an_empty_pool() {
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::Bool { values: vec![] });
        assert_eq!(state.validate(), Err(Error::EmptyPool { pool: 0 }));
    }

    #[test]
    fn validate_rejects_unordered_pool_bounds() {
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::Float {
            min: VoxBound::Number(1.0),
            max: VoxBound::Number(0.0),
            values: vec![0.5],
        });
        assert_eq!(state.validate(), Err(Error::PoolBound { pool: 0 }));
    }

    #[test]
    fn validate_rejects_a_non_integer_int_bound() {
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::Int {
            min: VoxBound::Number(0.5),
            max: VoxBound::None,
            values: vec![1],
        });
        assert_eq!(state.validate(), Err(Error::PoolBound { pool: 0 }));
    }

    #[test]
    fn validate_rejects_a_value_out_of_bounds() {
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::Float {
            min: VoxBound::Number(0.0),
            max: VoxBound::Number(1.0),
            values: vec![0.0, 2.0],
        });
        assert_eq!(
            state.validate(),
            Err(Error::PoolValue { pool: 0, index: 1 })
        );
    }

    #[test]
    fn validate_rejects_an_srgb_component_out_of_range() {
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::Srgb {
            values: vec![[0.0, 0.0, 0.0], [0.0, 1.5, 0.0]],
        });
        assert_eq!(
            state.validate(),
            Err(Error::PoolValue { pool: 0, index: 1 })
        );
    }

    #[test]
    fn validate_accepts_hdr_linear_components() {
        let mut state = VoxMain::default();
        // Linear colors allow components above 1 for HDR.
        state.add_value_pool(VoxValuePool::LinearRgb {
            values: vec![[0.0, 4.0, 12.0]],
        });
        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn validate_rejects_a_negative_linear_component() {
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::LinearRgba {
            values: vec![[0.0, -0.1, 0.0, 1.0]],
        });
        assert_eq!(
            state.validate(),
            Err(Error::PoolValue { pool: 0, index: 0 })
        );
    }

    #[test]
    fn validate_rejects_a_non_finite_linear_component() {
        let mut state = VoxMain::default();
        // A linear pool is only lower-bounded, so `+Infinity` would pass a bare
        // `>= 0` test; the finiteness guard must reject it. The wire has no
        // Infinity, so such a value could never round-trip.
        state.add_value_pool(VoxValuePool::LinearRgb {
            values: vec![[0.0, f64::INFINITY, 0.0]],
        });
        assert_eq!(
            state.validate(),
            Err(Error::PoolValue { pool: 0, index: 0 })
        );
    }

    #[test]
    fn validate_rejects_a_dangling_binding_pool() {
        let mut state = VoxMain::default();
        let mut palette = VoxPalette::default();
        // The binding references pool id 0, but the state holds no pools.
        palette.add_binding(
            "baseColorFactor".to_owned(),
            U32Id::<BVoxValuePool>::from_u32(0),
        );
        state.add_palette(palette);
        assert_eq!(
            state.validate(),
            Err(Error::BindingPool {
                palette: 0,
                binding: 0,
                pool: 0,
            })
        );
    }

    #[test]
    fn validate_rejects_a_material_value_index_out_of_range() {
        let mut state = VoxMain::default();
        let pool = int_pool(&mut state, vec![0, 1]);
        let mut palette = VoxPalette::default();
        palette.add_binding("v".to_owned(), pool);
        // Two pool values, but this material draws value-index 2.
        palette.add_material(vec![2]).unwrap();
        state.add_palette(palette);
        assert_eq!(
            state.validate(),
            Err(Error::MaterialValue {
                palette: 0,
                binding: 0,
                material: 0,
            })
        );
    }
}
