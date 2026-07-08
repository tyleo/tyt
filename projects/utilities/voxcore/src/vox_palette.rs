use crate::{BVoxMaterial, BVoxPaletteBinding, BVoxValuePool, VoxPaletteBinding};
use branded_id::{
    U32Id,
    soa::{IdField, IdRemap, IdStruct},
};
use std::collections::HashMap;

/// A palette: an ordered set of attribute-to-pool bindings and a set of
/// materials, stored column-major. Each binding names an attribute and the
/// [`VoxValuePool`](crate::VoxValuePool) that attribute draws from. Each
/// material carries one value-index per binding: `value_index(material,
/// binding)` is an index into the pool bound by `binding`. Resolve a value by
/// reading that index out of the pool a [`VoxMain`](crate::VoxMain) holds. An
/// object's voxels sample materials by id (see [`VoxObject`](crate::VoxObject)).
///
/// Build it with [`add_binding`](Self::add_binding) then
/// [`add_material`](Self::add_material). No two bindings share an attribute, a
/// rule [`VoxMain::validate`](crate::VoxMain::validate) checks.
#[derive(Debug, Default)]
pub struct VoxPalette {
    /// Binding id pool.
    binding_ids: IdStruct<BVoxPaletteBinding>,

    /// The attribute-to-pool bindings, one per column.
    bindings: IdField<BVoxPaletteBinding, VoxPaletteBinding>,

    /// Material id pool.
    material_ids: IdStruct<BVoxMaterial>,

    /// Per material, one value-index per binding.
    materials: IdField<BVoxMaterial, IdField<BVoxPaletteBinding, u32>>,

    /// Attribute-key index into `bindings`, for O(1)
    /// [`binding_by_attribute`](Self::binding_by_attribute) lookup. Rebuilt by
    /// [`gc`](Self::gc); exact for a validated palette, whose attribute keys are
    /// unique.
    by_attribute: HashMap<String, U32Id<BVoxPaletteBinding>>,
}

impl VoxPalette {
    /// Adds a binding after any existing ones and returns its id, back-filling
    /// existing materials with value-index 0 so every material keeps one
    /// value-index per binding. Add all bindings before any materials to avoid
    /// the back-fill placeholder, which is a valid index only if the bound pool
    /// is non-empty.
    pub fn add_binding(
        &mut self,
        attribute: String,
        pool: U32Id<BVoxValuePool>,
    ) -> U32Id<BVoxPaletteBinding> {
        let binding_id = self.binding_ids.retain();
        self.by_attribute.insert(attribute.clone(), binding_id);
        self.bindings
            .retain(binding_id, VoxPaletteBinding { attribute, pool });

        for material_id in self.material_ids.iter() {
            // Safety: retained material ids have a value column.
            let material = unsafe { self.materials.get_mut(material_id) };
            material.retain(binding_id, 0);
        }

        binding_id
    }

    /// Number of bindings.
    pub fn binding_count(&self) -> usize {
        self.binding_ids.len()
    }

    /// The binding `id`, or `None` if not one of this palette's.
    pub fn binding(&self, id: U32Id<BVoxPaletteBinding>) -> Option<&VoxPaletteBinding> {
        // Safety: retained ids have a value.
        self.binding_ids
            .is_retained(id)
            .then(|| unsafe { self.bindings.get(id) })
    }

    /// The binding bound to `attribute`, or `None` if no binding has that
    /// attribute key. O(1) through the attribute index. If a palette transiently
    /// binds the same attribute twice, which
    /// [`VoxMain::validate`](crate::VoxMain::validate) rejects, this returns the
    /// last such binding added.
    pub fn binding_by_attribute(&self, attribute: &str) -> Option<U32Id<BVoxPaletteBinding>> {
        self.by_attribute.get(attribute).copied()
    }

    /// Bindings in id order, as `(id, binding)`. Binding order is the material
    /// column order.
    pub fn iter_bindings(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxPaletteBinding>, &VoxPaletteBinding)> + '_ {
        // Safety: retained ids have a value.
        self.binding_ids
            .iter()
            .map(move |id| (id, unsafe { self.bindings.get(id) }))
    }

    /// Adds a material with one value-index per binding, in
    /// [`iter_bindings`](Self::iter_bindings) order, and returns its id. `None`,
    /// changing nothing, if `value_indices` has the wrong length. Each index is
    /// range-checked against its binding's pool by
    /// [`VoxMain::validate`](crate::VoxMain::validate), not here.
    pub fn add_material(&mut self, value_indices: Vec<u32>) -> Option<U32Id<BVoxMaterial>> {
        if value_indices.len() != self.binding_ids.len() {
            return None;
        }
        let material_id = self.material_ids.retain();
        let mut column = IdField::new();
        for (binding_id, index) in self.binding_ids.iter().zip(value_indices) {
            column.retain(binding_id, index);
        }
        self.materials.retain(material_id, column);
        Some(material_id)
    }

    /// Number of materials.
    pub fn material_count(&self) -> usize {
        self.material_ids.len()
    }

    /// Whether `id` is one of this palette's materials.
    pub fn contains_material(&self, id: U32Id<BVoxMaterial>) -> bool {
        self.material_ids.is_retained(id)
    }

    /// The value-index `material` draws for `binding`, an index into the pool
    /// bound by `binding`, or `None` if either id is not this palette's. Read
    /// the pool a [`VoxMain`](crate::VoxMain) holds by that index for the value.
    pub fn value_index(
        &self,
        material: U32Id<BVoxMaterial>,
        binding: U32Id<BVoxPaletteBinding>,
    ) -> Option<u32> {
        if !self.material_ids.is_retained(material) || !self.binding_ids.is_retained(binding) {
            return None;
        }
        // Safety: a retained material has a value-index for every binding.
        let column = unsafe { self.materials.get(material) };
        Some(*unsafe { column.get(binding) })
    }

    /// Material ids in id order; read value-indices with
    /// [`value_index`](Self::value_index).
    pub fn iter_materials(&self) -> impl Iterator<Item = U32Id<BVoxMaterial>> + '_ {
        self.material_ids.iter()
    }

    /// Deep copy. Liveness lives in the id pools, so the columns can't derive
    /// `Clone`; rebuild them against the cloned pools.
    pub fn clone_palette(&self) -> Self {
        let mut bindings = IdField::new();
        for binding_id in self.binding_ids.iter() {
            // Safety: retained ids have a value.
            let binding = unsafe { self.bindings.get(binding_id) }.clone();
            bindings.retain(binding_id, binding);
        }

        let mut materials = IdField::new();
        for material_id in self.material_ids.iter() {
            // Safety: a retained material has a value column. Its values are
            // Copy u32, so the inner column clones bytewise, unlike the bindings
            // above whose attribute strings are rebuilt.
            let column = unsafe { self.materials.get(material_id) }.clone();
            materials.retain(material_id, column);
        }

        Self {
            binding_ids: self.binding_ids.clone(),
            bindings,
            material_ids: self.material_ids.clone(),
            materials,
            by_attribute: self.by_attribute.clone(),
        }
    }

    /// Removes binding `id`, freeing its attribute string. Each material keeps a
    /// Copy value-index at the removed binding's slot, unreferenced (reads are
    /// guarded by binding retention) until [`gc`](Self::gc) compacts the
    /// columns. `None`, changing nothing, if `id` is not one of this palette's
    /// bindings. Leaves a hole until [`VoxMain::gc`](crate::VoxMain::gc)
    /// renumbers.
    pub fn remove_binding(&mut self, id: U32Id<BVoxPaletteBinding>) -> Option<()> {
        if !self.binding_ids.is_retained(id) {
            return None;
        }
        // Drop this binding's index entry, but only if the entry still points
        // here: a duplicate attribute (which validate rejects) may have
        // overwritten it with a later binding that must keep its lookup.
        // Safety: a retained binding has a value.
        let attribute = unsafe { self.bindings.get(id) }.attribute.clone();
        if self.by_attribute.get(&attribute) == Some(&id) {
            self.by_attribute.remove(&attribute);
        }
        // A value-index is Copy, so releasing each material's slot at `id` would
        // be a no-op; leave it for gc to compact and only free the binding.
        // Safety: a retained binding has a value.
        unsafe { self.bindings.release(id) };
        self.binding_ids.release(id);
        Some(())
    }

    /// Drops material `id` and its value-index column. The caller must first
    /// ensure no live voxel still samples it, which is why this is internal and
    /// reached only through
    /// [`VoxMain::remove_material`](crate::VoxMain::remove_material). Leaves a
    /// hole until [`gc`](Self::gc) renumbers.
    pub(crate) fn remove_material(&mut self, id: U32Id<BVoxMaterial>) -> Option<()> {
        if !self.material_ids.is_retained(id) {
            return None;
        }
        // The inner column holds Copy u32 value-indices, so dropping the inner
        // IdField frees its buffer with nothing to release per binding.
        // Safety: a retained material has an inner column.
        unsafe { self.materials.release(id) };
        self.material_ids.release(id);
        Some(())
    }

    /// Compacts the binding and material pools back to a contiguous `0..len`,
    /// moving every value to its relabeled id, and returns the material
    /// relabeling so a [`VoxMain`](crate::VoxMain) can translate the samples
    /// that point at these materials. Bindings are referenced only within this
    /// palette, so their relabeling stays internal. Value-indices point into the
    /// bound pools, whose contents gc does not touch, so they stay valid.
    pub(crate) fn gc(&mut self) -> IdRemap<BVoxMaterial, u32> {
        let binding_remap = self.binding_ids.gc();
        // Safety: the binding column was in sync with the pre-gc binding pool,
        // and nothing has retained or released since.
        unsafe { self.bindings.gc(&binding_remap) };

        let material_ids: Vec<_> = self.material_ids.iter().collect();
        for material_id in material_ids {
            // Safety: a retained material holds a value-index for every pre-gc
            // binding id, and the remap came from this palette's binding pool.
            let column = unsafe { self.materials.get_mut(material_id) };
            unsafe { column.gc(&binding_remap) };
        }

        let material_remap = self.material_ids.gc();
        // Safety: the material column was in sync with the pre-gc material pool,
        // and nothing has retained or released since.
        unsafe { self.materials.gc(&material_remap) };

        // Rebuild the attribute index against the relabeled binding ids.
        self.by_attribute.clear();
        let binding_ids: Vec<_> = self.binding_ids.iter().collect();
        for binding_id in binding_ids {
            // Safety: retained binding ids have a value.
            let attribute = unsafe { self.bindings.get(binding_id) }.attribute.clone();
            self.by_attribute.insert(attribute, binding_id);
        }

        material_remap
    }

    /// Maps each material's value-index `i` to `remap[i]` for every binding on
    /// `pool`. `remap` covers every pre-prune index of `pool`.
    pub(crate) fn remap_pool_value_indices(&mut self, pool: U32Id<BVoxValuePool>, remap: &[u32]) {
        // The bindings on `pool`, found once so each material's value-index
        // column is visited once for all of them.
        let bound_bindings: Vec<_> = self
            .binding_ids
            .iter()
            .filter(|&binding_id| {
                // Safety: retained binding ids have a value.
                unsafe { self.bindings.get(binding_id) }.pool == pool
            })
            .collect();
        if bound_bindings.is_empty() {
            return;
        }

        for material_id in self.material_ids.iter() {
            // Safety: a retained material holds a value-index for every
            // binding, and the value-index column is keyed by binding id.
            let column = unsafe { self.materials.get_mut(material_id) };
            for &binding_id in &bound_bindings {
                let slot = unsafe { column.get_mut(binding_id) };
                *slot = remap[*slot as usize];
            }
        }
    }

    /// Translates every binding's pool id through `remap`, matching a value-pool
    /// store a [`VoxMain`](crate::VoxMain) is compacting. Requires a
    /// referentially valid palette, so every binding names a live pool.
    pub(crate) fn relabel_value_pools(&mut self, remap: &IdRemap<BVoxValuePool, u32>) {
        let binding_ids: Vec<_> = self.binding_ids.iter().collect();
        for binding_id in binding_ids {
            // Safety: retained binding ids have a value.
            let binding = unsafe { self.bindings.get_mut(binding_id) };
            binding.pool = remap
                .new_id(binding.pool)
                .expect("a binding names a live value pool in a valid state");
        }
    }
}

impl Drop for VoxPalette {
    fn drop(&mut self) {
        // Each material's inner column is an IdField owning a heap buffer whose
        // values are Copy u32, so releasing the inner IdField frees the buffer
        // with nothing to release per binding. The bindings own attribute
        // strings, freed by releasing them.
        // Safety: both columns hold a value for every id in their pools.
        unsafe {
            self.materials.release_all(&self.material_ids);
            self.bindings.release_all(&self.binding_ids);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BVoxMaterial, BVoxPaletteBinding, BVoxValuePool, VoxPalette};
    use branded_id::U32Id;

    fn pool(index: u32) -> U32Id<BVoxValuePool> {
        U32Id::from_u32(index)
    }

    #[test]
    fn builds_and_reads_a_material_palette() {
        let mut palette = VoxPalette::default();
        let metallic = palette.add_binding("metallic".to_owned(), pool(0));
        let ior = palette.add_binding("ior".to_owned(), pool(1));

        // Two materials, each a value-index per binding, in binding order.
        let matte = palette.add_material(vec![0, 3]).unwrap();
        let shiny = palette.add_material(vec![1, 3]).unwrap();

        assert_eq!(palette.binding_count(), 2);
        assert_eq!(palette.material_count(), 2);
        assert_eq!(palette.binding(metallic).unwrap().attribute, "metallic");
        assert_eq!(palette.binding(ior).unwrap().pool, pool(1));
        assert_eq!(palette.value_index(matte, metallic), Some(0));
        assert_eq!(palette.value_index(matte, ior), Some(3));
        assert_eq!(palette.value_index(shiny, metallic), Some(1));
        assert_eq!(
            palette
                .iter_bindings()
                .map(|(id, b)| (id, b.attribute.as_str()))
                .collect::<Vec<_>>(),
            [(metallic, "metallic"), (ior, "ior")]
        );
    }

    #[test]
    fn add_material_rejects_wrong_arity_without_changing_state() {
        let mut palette = VoxPalette::default();
        palette.add_binding("baseColorFactor".to_owned(), pool(0));
        // One binding, but two value-indices supplied.
        assert_eq!(palette.add_material(vec![0, 1]), None);
        assert_eq!(palette.material_count(), 0);
    }

    #[test]
    fn binding_by_attribute_indexes_and_survives_gc() {
        let mut palette = VoxPalette::default();
        let color = palette.add_binding("baseColorFactor".to_owned(), pool(0));
        let metal = palette.add_binding("metallicFactor".to_owned(), pool(1));

        assert_eq!(palette.binding_by_attribute("baseColorFactor"), Some(color));
        assert_eq!(palette.binding_by_attribute("metallicFactor"), Some(metal));
        assert_eq!(palette.binding_by_attribute("missing"), None);

        // Removing a binding drops it from the index; gc renumbers the rest and
        // the index follows.
        palette.remove_binding(color);
        assert_eq!(palette.binding_by_attribute("baseColorFactor"), None);
        palette.gc();
        let metal = U32Id::<BVoxPaletteBinding>::from_u32(0);
        assert_eq!(palette.binding_by_attribute("metallicFactor"), Some(metal));
        assert_eq!(palette.binding_by_attribute("baseColorFactor"), None);
    }

    #[test]
    fn add_binding_back_fills_existing_materials_with_zero() {
        let mut palette = VoxPalette::default();
        let color = palette.add_binding("baseColorFactor".to_owned(), pool(0));
        let material = palette.add_material(vec![7]).unwrap();

        let added = palette.add_binding("metallicFactor".to_owned(), pool(1));
        assert_eq!(palette.value_index(material, color), Some(7));
        assert_eq!(palette.value_index(material, added), Some(0));
    }

    #[test]
    fn clone_palette_is_an_independent_deep_copy() {
        let mut palette = VoxPalette::default();
        let binding = palette.add_binding("baseColorFactor".to_owned(), pool(0));
        let material = palette.add_material(vec![2]).unwrap();

        let copy = palette.clone_palette();
        assert_eq!(copy.value_index(material, binding), Some(2));
        assert_eq!(copy.binding(binding).unwrap().attribute, "baseColorFactor");

        // Mutating the original must not touch the copy.
        palette.add_material(vec![5]).unwrap();
        assert_eq!(palette.material_count(), 2);
        assert_eq!(copy.material_count(), 1);
    }

    #[test]
    fn remove_binding_keeps_materials_then_gc_renumbers() {
        let mut palette = VoxPalette::default();
        let a = palette.add_binding("a".to_owned(), pool(0));
        let b = palette.add_binding("b".to_owned(), pool(1));
        let material = palette.add_material(vec![1, 2]).unwrap();

        assert_eq!(palette.remove_binding(a), Some(()));
        assert_eq!(palette.binding_count(), 1);
        assert_eq!(palette.binding(a), None); // a hole until gc
        assert_eq!(palette.value_index(material, a), None);
        assert_eq!(palette.value_index(material, b), Some(2));
        assert_eq!(palette.remove_binding(a), None); // already gone

        palette.gc();
        // The surviving binding and material renumber to 0.
        let binding = U32Id::<BVoxPaletteBinding>::from_u32(0);
        let material = U32Id::<BVoxMaterial>::from_u32(0);
        assert_eq!(palette.binding(binding).unwrap().attribute, "b");
        assert_eq!(palette.value_index(material, binding), Some(2));
    }

    #[test]
    fn remove_material_then_gc_compacts_remaining_materials() {
        let mut palette = VoxPalette::default();
        let binding = palette.add_binding("v".to_owned(), pool(0));
        let keep = palette.add_material(vec![0]).unwrap();
        let drop = palette.add_material(vec![1]).unwrap();
        let last = palette.add_material(vec![2]).unwrap();

        assert_eq!(palette.remove_material(drop), Some(()));
        assert_eq!(palette.material_count(), 2);
        assert!(!palette.contains_material(drop));
        assert!(palette.contains_material(keep) && palette.contains_material(last));
        assert_eq!(palette.remove_material(drop), None); // already gone

        palette.gc();
        // The two survivors are contiguous; their value-indices are intact.
        let indices: Vec<u32> = palette
            .iter_materials()
            .map(|material| palette.value_index(material, binding).unwrap())
            .collect();
        assert_eq!(palette.material_count(), 2);
        assert_eq!(indices, [0, 2]);
    }
}
