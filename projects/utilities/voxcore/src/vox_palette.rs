use crate::{
    BVoxArrayProperty, BVoxMaterial, BVoxPoolValue, BVoxScalarProperty, BVoxValuePool,
    VoxArrayProperty, VoxPropertyId, VoxScalarProperty,
};
use branded_id::{
    IdSlice, U32Id,
    soa::{IdField, IdRemap, IdStruct},
};
use std::collections::HashMap;

/// A palette: an ordered set of array properties, a set of scalar properties,
/// and a set of materials, one row per material. Each array property names a
/// property and the [`VoxValuePool`](crate::VoxValuePool) it draws from, and
/// each material row carries one value id per array property:
/// `value_id(material, array_property)` identifies a value in the pool
/// `array_property` draws from. Each scalar property pins its name to a
/// single pool value for the whole palette, with no per-material column.
/// Resolve a value by reading the id out of the pool a
/// [`VoxMain`](crate::VoxMain) holds. An object's voxels sample materials by
/// id (see [`VoxObject`](crate::VoxObject)).
///
/// Build it with [`add_array_property`](Self::add_array_property) and
/// [`add_scalar_property`](Self::add_scalar_property), then
/// [`add_material`](Self::add_material). No two properties share a name
/// across the two lists, a rule
/// [`VoxMain::validate`](crate::VoxMain::validate) checks.
#[derive(Debug, Default)]
pub struct VoxPalette {
    /// Array property id pool.
    array_property_ids: IdStruct<BVoxArrayProperty>,

    /// The array properties, in material-row value-id order.
    array_properties: IdField<BVoxArrayProperty, VoxArrayProperty>,

    /// Scalar property id pool.
    scalar_property_ids: IdStruct<BVoxScalarProperty>,

    /// The scalar properties, each carrying its own pinned value id.
    scalar_properties: IdField<BVoxScalarProperty, VoxScalarProperty>,

    /// Material id pool.
    material_ids: IdStruct<BVoxMaterial>,

    /// Per material, one value id per array property.
    materials: IdField<BVoxMaterial, IdField<BVoxArrayProperty, U32Id<BVoxPoolValue>>>,

    /// Name index into `array_properties`, for O(1)
    /// [`array_property_by_name`](Self::array_property_by_name) lookup.
    /// Rebuilt by [`gc`](Self::gc); exact for a validated palette, whose
    /// property names are unique.
    array_property_by_name: HashMap<String, U32Id<BVoxArrayProperty>>,

    /// Name index into `scalar_properties`, for O(1)
    /// [`scalar_property_by_name`](Self::scalar_property_by_name) lookup.
    /// Rebuilt by [`gc`](Self::gc); exact for a validated palette, whose
    /// property names are unique.
    scalar_property_by_name: HashMap<String, U32Id<BVoxScalarProperty>>,

    /// Name index across both property lists, tagged by arity, for O(1)
    /// [`property_by_name`](Self::property_by_name) lookup. Rebuilt by
    /// [`gc`](Self::gc); exact for a validated palette, whose property names
    /// are unique.
    property_by_name: HashMap<String, VoxPropertyId>,
}

impl VoxPalette {
    /// Adds an array property after any existing ones and returns its id,
    /// back-filling existing materials with value id 0 so every material
    /// keeps one value id per array property. Add all array properties
    /// before any materials to avoid the back-fill placeholder, which is a
    /// valid id only if the property's pool is non-empty.
    pub fn add_array_property(
        &mut self,
        name: String,
        pool: U32Id<BVoxValuePool>,
    ) -> U32Id<BVoxArrayProperty> {
        let array_property_id = self.array_property_ids.retain();

        self.array_property_by_name
            .insert(name.clone(), array_property_id);
        self.property_by_name
            .insert(name.clone(), VoxPropertyId::Array(array_property_id));

        self.array_properties
            .retain(array_property_id, VoxArrayProperty { name, pool });

        for material_id in self.material_ids.iter() {
            // Safety: retained material ids have a value row.
            let material = unsafe { self.materials.get_mut(material_id) };
            material.retain(array_property_id, U32Id::from_u32(0));
        }

        array_property_id
    }

    /// Number of array properties.
    pub fn array_property_count(&self) -> usize {
        self.array_property_ids.len()
    }

    /// The array property `id`, or `None` if not one of this palette's.
    pub fn array_property(&self, id: U32Id<BVoxArrayProperty>) -> Option<&VoxArrayProperty> {
        // Safety: retained ids have a value.
        self.array_property_ids
            .is_retained(id)
            .then(|| unsafe { self.array_properties.get(id) })
    }

    /// The array property named `name`, or `None` if none has that name. O(1)
    /// through the name index. If a palette transiently declares the same
    /// name twice, which [`VoxMain::validate`](crate::VoxMain::validate)
    /// rejects, this returns the last such property added.
    pub fn array_property_by_name(&self, name: &str) -> Option<U32Id<BVoxArrayProperty>> {
        self.array_property_by_name.get(name).copied()
    }

    /// Array properties in id order, as `(id, property)`. Array property
    /// order is the value-id order of each material row.
    pub fn iter_array_properties(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxArrayProperty>, &VoxArrayProperty)> + '_ {
        // Safety: retained ids have a value.
        self.array_property_ids
            .iter()
            .map(move |id| (id, unsafe { self.array_properties.get(id) }))
    }

    /// Adds a scalar property and returns its id. It pins `name` to the
    /// single value `value_id` in `pool` for the whole palette; scalar
    /// properties have no per-material column, so existing materials are
    /// untouched. [`VoxMain::validate`](crate::VoxMain::validate)
    /// range-checks the value id against the pool.
    pub fn add_scalar_property(
        &mut self,
        name: String,
        pool: U32Id<BVoxValuePool>,
        value_id: U32Id<BVoxPoolValue>,
    ) -> U32Id<BVoxScalarProperty> {
        let scalar_property_id = self.scalar_property_ids.retain();

        self.scalar_property_by_name
            .insert(name.clone(), scalar_property_id);
        self.property_by_name
            .insert(name.clone(), VoxPropertyId::Scalar(scalar_property_id));

        self.scalar_properties.retain(
            scalar_property_id,
            VoxScalarProperty {
                name,
                pool,
                value_id,
            },
        );

        scalar_property_id
    }

    /// Number of scalar properties.
    pub fn scalar_property_count(&self) -> usize {
        self.scalar_property_ids.len()
    }

    /// The scalar property `id`, or `None` if not one of this palette's.
    pub fn scalar_property(&self, id: U32Id<BVoxScalarProperty>) -> Option<&VoxScalarProperty> {
        // Safety: retained ids have a value.
        self.scalar_property_ids
            .is_retained(id)
            .then(|| unsafe { self.scalar_properties.get(id) })
    }

    /// The scalar property named `name`, or `None` if none has that name.
    /// O(1) through the name index. If a palette transiently declares the
    /// same name twice, which [`VoxMain::validate`](crate::VoxMain::validate)
    /// rejects, this returns the last such property added.
    pub fn scalar_property_by_name(&self, name: &str) -> Option<U32Id<BVoxScalarProperty>> {
        self.scalar_property_by_name.get(name).copied()
    }

    /// The property named `name` across both lists, tagged by arity, or
    /// `None` if none has that name. O(1) through the name index. If a
    /// palette transiently declares the same name twice, which
    /// [`VoxMain::validate`](crate::VoxMain::validate) rejects, this returns
    /// the last such property added.
    pub fn property_by_name(&self, name: &str) -> Option<VoxPropertyId> {
        self.property_by_name.get(name).copied()
    }

    /// Scalar properties in id order, as `(id, property)`.
    pub fn iter_scalar_properties(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxScalarProperty>, &VoxScalarProperty)> + '_ {
        // Safety: retained ids have a value.
        self.scalar_property_ids
            .iter()
            .map(move |id| (id, unsafe { self.scalar_properties.get(id) }))
    }

    /// Adds a material with one value id per array property, in
    /// [`iter_array_properties`](Self::iter_array_properties) order, and
    /// returns its id. `None`, changing nothing, if `value_ids` has the
    /// wrong length. Each value id is range-checked against its property's
    /// pool by [`VoxMain::validate`](crate::VoxMain::validate), not here.
    pub fn add_material(
        &mut self,
        value_ids: Vec<U32Id<BVoxPoolValue>>,
    ) -> Option<U32Id<BVoxMaterial>> {
        if value_ids.len() != self.array_property_ids.len() {
            return None;
        }
        let material_id = self.material_ids.retain();
        let mut row = IdField::new();
        for (array_property_id, value_id) in self.array_property_ids.iter().zip(value_ids) {
            row.retain(array_property_id, value_id);
        }
        self.materials.retain(material_id, row);
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

    /// The value id `material` draws for `array_property`, identifying a
    /// value in the pool that property draws from, or `None` if either id is
    /// not this palette's. Read the pool a [`VoxMain`](crate::VoxMain) holds
    /// by that id for the value.
    pub fn value_id(
        &self,
        material: U32Id<BVoxMaterial>,
        array_property: U32Id<BVoxArrayProperty>,
    ) -> Option<U32Id<BVoxPoolValue>> {
        if !self.material_ids.is_retained(material)
            || !self.array_property_ids.is_retained(array_property)
        {
            return None;
        }
        // Safety: a retained material has a value id for every array
        // property.
        let row = unsafe { self.materials.get(material) };
        Some(*unsafe { row.get(array_property) })
    }

    /// Material ids in id order; read value ids with
    /// [`value_id`](Self::value_id).
    pub fn iter_materials(&self) -> impl Iterator<Item = U32Id<BVoxMaterial>> + '_ {
        self.material_ids.iter()
    }

    /// Deep copy. Liveness lives in the id pools, so the columns can't derive
    /// `Clone`; rebuild them against the cloned pools.
    pub fn clone_palette(&self) -> Self {
        let mut array_properties = IdField::new();
        for array_property_id in self.array_property_ids.iter() {
            // Safety: retained ids have a value.
            let property = unsafe { self.array_properties.get(array_property_id) }.clone();
            array_properties.retain(array_property_id, property);
        }

        let mut scalar_properties = IdField::new();
        for scalar_property_id in self.scalar_property_ids.iter() {
            // Safety: retained ids have a value.
            let property = unsafe { self.scalar_properties.get(scalar_property_id) }.clone();
            scalar_properties.retain(scalar_property_id, property);
        }

        let mut materials = IdField::new();
        for material_id in self.material_ids.iter() {
            // Safety: a retained material has a value row. Its value ids are
            // Copy, so the row clones bytewise, unlike the properties above
            // whose name strings are rebuilt.
            let row = unsafe { self.materials.get(material_id) }.clone();
            materials.retain(material_id, row);
        }

        Self {
            array_property_ids: self.array_property_ids.clone(),
            array_properties,
            scalar_property_ids: self.scalar_property_ids.clone(),
            scalar_properties,
            material_ids: self.material_ids.clone(),
            materials,
            array_property_by_name: self.array_property_by_name.clone(),
            scalar_property_by_name: self.scalar_property_by_name.clone(),
            property_by_name: self.property_by_name.clone(),
        }
    }

    /// Removes array property `id`, freeing its name string. Each material
    /// keeps a Copy value id at the removed property's slot, unreferenced
    /// (reads are guarded by property retention) until [`gc`](Self::gc)
    /// compacts the rows. `None`, changing nothing, if `id` is not one of
    /// this palette's array properties. Leaves a hole until
    /// [`VoxMain::gc`](crate::VoxMain::gc) renumbers.
    pub fn remove_array_property(&mut self, id: U32Id<BVoxArrayProperty>) -> Option<()> {
        if !self.array_property_ids.is_retained(id) {
            return None;
        }

        // Drop index entries still pointing here; a duplicate name may have
        // overwritten one.
        // Safety: a retained array property has a value.
        let name = unsafe { self.array_properties.get(id) }.name.clone();
        if self.array_property_by_name.get(&name) == Some(&id) {
            self.array_property_by_name.remove(&name);
        }
        if self.property_by_name.get(&name) == Some(&VoxPropertyId::Array(id)) {
            self.property_by_name.remove(&name);
        }

        // A value id is Copy, so releasing each material's slot at `id`
        // would be a no-op; leave it for gc to compact and only free the
        // property.
        // Safety: a retained array property has a value.
        unsafe { self.array_properties.release(id) };
        self.array_property_ids.release(id);
        Some(())
    }

    /// Removes scalar property `id`, freeing it. Materials have no column for
    /// a scalar property, so they are untouched. `None`, changing nothing, if
    /// `id` is not one of this palette's scalar properties. Leaves a hole
    /// until [`VoxMain::gc`](crate::VoxMain::gc) renumbers.
    pub fn remove_scalar_property(&mut self, id: U32Id<BVoxScalarProperty>) -> Option<()> {
        if !self.scalar_property_ids.is_retained(id) {
            return None;
        }

        // Drop index entries still pointing here; a duplicate name may have
        // overwritten one.
        // Safety: a retained scalar property has a value.
        let name = unsafe { self.scalar_properties.get(id) }.name.clone();
        if self.scalar_property_by_name.get(&name) == Some(&id) {
            self.scalar_property_by_name.remove(&name);
        }
        if self.property_by_name.get(&name) == Some(&VoxPropertyId::Scalar(id)) {
            self.property_by_name.remove(&name);
        }

        // Safety: a retained scalar property has a value.
        unsafe { self.scalar_properties.release(id) };
        self.scalar_property_ids.release(id);
        Some(())
    }

    /// Drops material `id` and its value-id row. The caller must first
    /// ensure no live voxel still samples it, which is why this is internal
    /// and reached only through
    /// [`VoxMain::remove_material`](crate::VoxMain::remove_material). Leaves
    /// a hole until [`gc`](Self::gc) renumbers.
    pub(crate) fn remove_material(&mut self, id: U32Id<BVoxMaterial>) -> Option<()> {
        if !self.material_ids.is_retained(id) {
            return None;
        }
        // The row holds Copy value ids, so dropping the inner IdField frees
        // its buffer with nothing to release per array property.
        // Safety: a retained material has a row.
        unsafe { self.materials.release(id) };
        self.material_ids.release(id);
        Some(())
    }

    /// Compacts the property and material pools back to a contiguous
    /// `0..len`, moving every value to its relabeled id, and returns the
    /// material relabeling so a [`VoxMain`](crate::VoxMain) can translate the
    /// samples that point at these materials. Properties are referenced only
    /// within this palette, so their relabelings stay internal. Value ids
    /// point into the referenced pools, whose contents gc does not touch, so
    /// they stay valid.
    pub(crate) fn gc(&mut self) -> IdRemap<BVoxMaterial, u32> {
        let array_property_remap = self.array_property_ids.gc();
        // Safety: the property column was in sync with the pre-gc property
        // pool, and nothing has retained or released since.
        unsafe { self.array_properties.gc(&array_property_remap) };

        let material_ids: Vec<_> = self.material_ids.iter().collect();
        for material_id in material_ids {
            // Safety: a retained material holds a value id for every pre-gc
            // array property id, and the remap came from this palette's
            // array property pool.
            let row = unsafe { self.materials.get_mut(material_id) };
            unsafe { row.gc(&array_property_remap) };
        }

        let material_remap = self.material_ids.gc();
        // Safety: the material column was in sync with the pre-gc material
        // pool, and nothing has retained or released since.
        unsafe { self.materials.gc(&material_remap) };

        let scalar_property_remap = self.scalar_property_ids.gc();
        // Safety: the scalar property column was in sync with the pre-gc
        // scalar property pool, and nothing has retained or released since.
        unsafe { self.scalar_properties.gc(&scalar_property_remap) };

        // Rebuild the name indexes against the relabeled property ids.
        self.array_property_by_name.clear();
        self.scalar_property_by_name.clear();
        self.property_by_name.clear();

        let array_property_ids: Vec<_> = self.array_property_ids.iter().collect();
        for array_property_id in array_property_ids {
            // Safety: retained array property ids have a value.
            let name = unsafe { self.array_properties.get(array_property_id) }
                .name
                .clone();

            self.array_property_by_name
                .insert(name.clone(), array_property_id);
            self.property_by_name
                .insert(name, VoxPropertyId::Array(array_property_id));
        }

        let scalar_property_ids: Vec<_> = self.scalar_property_ids.iter().collect();
        for scalar_property_id in scalar_property_ids {
            // Safety: retained scalar property ids have a value.
            let name = unsafe { self.scalar_properties.get(scalar_property_id) }
                .name
                .clone();

            self.scalar_property_by_name
                .insert(name.clone(), scalar_property_id);
            self.property_by_name
                .insert(name, VoxPropertyId::Scalar(scalar_property_id));
        }

        material_remap
    }

    /// Maps every value id into `pool` through `remap`: each material's cell
    /// for an array property on `pool`, and each scalar property's pinned
    /// value id on `pool`. `remap` covers every pre-prune value id of `pool`.
    pub(crate) fn remap_pool_value_ids(
        &mut self,
        pool: U32Id<BVoxValuePool>,
        remap: &IdSlice<BVoxPoolValue, U32Id<BVoxPoolValue>>,
    ) {
        // The array properties on `pool`, found once so each material's row
        // is visited once for all of them.
        let pool_properties: Vec<_> = self
            .array_property_ids
            .iter()
            .filter(|&array_property_id| {
                // Safety: retained array property ids have a value.
                unsafe { self.array_properties.get(array_property_id) }.pool == pool
            })
            .collect();
        if !pool_properties.is_empty() {
            for material_id in self.material_ids.iter() {
                // Safety: a retained material holds a value id for every
                // array property, and the row is keyed by array property id.
                let row = unsafe { self.materials.get_mut(material_id) };
                for &array_property_id in &pool_properties {
                    let slot = unsafe { row.get_mut(array_property_id) };
                    *slot = remap[slot.to_usize_id()];
                }
            }
        }

        for scalar_property_id in self.scalar_property_ids.iter() {
            // Safety: retained scalar property ids have a value.
            let property = unsafe { self.scalar_properties.get_mut(scalar_property_id) };
            if property.pool == pool {
                property.value_id = remap[property.value_id.to_usize_id()];
            }
        }
    }

    /// Translates every property's pool id through `remap`, matching a
    /// value-pool store a [`VoxMain`](crate::VoxMain) is compacting. Requires
    /// a referentially valid palette, so every property names a live pool.
    pub(crate) fn relabel_value_pools(&mut self, remap: &IdRemap<BVoxValuePool, u32>) {
        let array_property_ids: Vec<_> = self.array_property_ids.iter().collect();
        for array_property_id in array_property_ids {
            // Safety: retained array property ids have a value.
            let property = unsafe { self.array_properties.get_mut(array_property_id) };
            property.pool = remap
                .new_id(property.pool)
                .expect("an array property names a live value pool in a valid state");
        }

        let scalar_property_ids: Vec<_> = self.scalar_property_ids.iter().collect();
        for scalar_property_id in scalar_property_ids {
            // Safety: retained scalar property ids have a value.
            let property = unsafe { self.scalar_properties.get_mut(scalar_property_id) };
            property.pool = remap
                .new_id(property.pool)
                .expect("a scalar property names a live value pool in a valid state");
        }
    }
}

impl Drop for VoxPalette {
    fn drop(&mut self) {
        // Each material's row is an IdField owning a heap buffer whose value
        // ids are Copy, so releasing the inner IdField frees the buffer with
        // nothing to release per array property. The properties own name
        // strings, freed by releasing them.
        // Safety: each column holds a value for every id in its pool.
        unsafe {
            self.materials.release_all(&self.material_ids);
            self.array_properties.release_all(&self.array_property_ids);
            self.scalar_properties
                .release_all(&self.scalar_property_ids);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BVoxArrayProperty, BVoxMaterial, BVoxPoolValue, BVoxScalarProperty, BVoxValuePool,
        VoxPalette, VoxPropertyId,
    };
    use branded_id::U32Id;

    fn pool(index: u32) -> U32Id<BVoxValuePool> {
        U32Id::from_u32(index)
    }

    fn value(index: u32) -> U32Id<BVoxPoolValue> {
        U32Id::from_u32(index)
    }

    #[test]
    fn builds_and_reads_a_material_palette() {
        let mut palette = VoxPalette::default();
        let metallic = palette.add_array_property("metallic".to_owned(), pool(0));
        let ior = palette.add_array_property("ior".to_owned(), pool(1));

        // Two materials, each a value id per array property, in property
        // order.
        let matte = palette.add_material(vec![value(0), value(3)]).unwrap();
        let shiny = palette.add_material(vec![value(1), value(3)]).unwrap();

        assert_eq!(palette.array_property_count(), 2);
        assert_eq!(palette.material_count(), 2);
        assert_eq!(palette.array_property(metallic).unwrap().name, "metallic");
        assert_eq!(palette.array_property(ior).unwrap().pool, pool(1));
        assert_eq!(palette.value_id(matte, metallic), Some(value(0)));
        assert_eq!(palette.value_id(matte, ior), Some(value(3)));
        assert_eq!(palette.value_id(shiny, metallic), Some(value(1)));
        assert_eq!(
            palette
                .iter_array_properties()
                .map(|(id, p)| (id, p.name.as_str()))
                .collect::<Vec<_>>(),
            [(metallic, "metallic"), (ior, "ior")]
        );
    }

    #[test]
    fn builds_and_reads_scalar_properties() {
        let mut palette = VoxPalette::default();
        let strength =
            palette.add_scalar_property("emissiveStrength".to_owned(), pool(0), value(3));
        let color = palette.add_array_property("baseColorFactor".to_owned(), pool(1));

        assert_eq!(palette.scalar_property_count(), 1);
        let property = palette.scalar_property(strength).unwrap();
        assert_eq!(property.name, "emissiveStrength");
        assert_eq!(property.pool, pool(0));
        assert_eq!(property.value_id, value(3));
        assert_eq!(
            palette.scalar_property_by_name("emissiveStrength"),
            Some(strength)
        );
        assert_eq!(palette.scalar_property_by_name("missing"), None);
        // The combined index tags each property with its arity.
        assert_eq!(
            palette.property_by_name("emissiveStrength"),
            Some(VoxPropertyId::Scalar(strength))
        );
        assert_eq!(
            palette.property_by_name("baseColorFactor"),
            Some(VoxPropertyId::Array(color))
        );
        assert_eq!(palette.property_by_name("missing"), None);
        assert_eq!(
            palette
                .iter_scalar_properties()
                .map(|(id, p)| (id, p.name.as_str()))
                .collect::<Vec<_>>(),
            [(strength, "emissiveStrength")]
        );
    }

    #[test]
    fn scalar_properties_leave_materials_untouched() {
        let mut palette = VoxPalette::default();
        let color = palette.add_array_property("baseColorFactor".to_owned(), pool(0));
        let material = palette.add_material(vec![value(1)]).unwrap();

        // A scalar property adds no per-material column; material rows keep
        // one value id per array property.
        palette.add_scalar_property("emissiveStrength".to_owned(), pool(1), value(0));
        assert_eq!(palette.value_id(material, color), Some(value(1)));
        assert!(palette.add_material(vec![value(0)]).is_some());
        assert_eq!(palette.add_material(vec![value(0), value(0)]), None);
    }

    #[test]
    fn remove_scalar_property_drops_indexes_then_gc_renumbers() {
        let mut palette = VoxPalette::default();
        let a = palette.add_scalar_property("a".to_owned(), pool(0), value(0));
        let b = palette.add_scalar_property("b".to_owned(), pool(0), value(1));

        assert_eq!(palette.remove_scalar_property(a), Some(()));
        assert_eq!(palette.scalar_property_count(), 1);
        assert_eq!(palette.scalar_property(a), None); // a hole until gc
        assert_eq!(palette.scalar_property_by_name("a"), None);
        assert_eq!(palette.property_by_name("a"), None);
        assert_eq!(palette.remove_scalar_property(a), None); // already gone
        assert_eq!(palette.scalar_property(b).unwrap().name, "b");

        palette.gc();
        // The survivor renumbers to 0 and the indexes follow.
        let relabeled = U32Id::<BVoxScalarProperty>::from_u32(0);
        assert_eq!(palette.scalar_property(relabeled).unwrap().name, "b");
        assert_eq!(palette.scalar_property_by_name("b"), Some(relabeled));
        assert_eq!(
            palette.property_by_name("b"),
            Some(VoxPropertyId::Scalar(relabeled))
        );
    }

    #[test]
    fn clone_palette_deep_copies_scalar_properties() {
        let mut palette = VoxPalette::default();
        let strength =
            palette.add_scalar_property("emissiveStrength".to_owned(), pool(0), value(2));

        let copy = palette.clone_palette();
        assert_eq!(copy.scalar_property(strength).unwrap().value_id, value(2));
        assert_eq!(
            copy.property_by_name("emissiveStrength"),
            Some(VoxPropertyId::Scalar(strength))
        );

        // Mutating the original must not touch the copy.
        palette.add_scalar_property("alphaCutoff".to_owned(), pool(1), value(0));
        assert_eq!(palette.scalar_property_count(), 2);
        assert_eq!(copy.scalar_property_count(), 1);
    }

    #[test]
    fn add_material_rejects_wrong_arity_without_changing_state() {
        let mut palette = VoxPalette::default();
        palette.add_array_property("baseColorFactor".to_owned(), pool(0));
        // One array property, but two value ids supplied.
        assert_eq!(palette.add_material(vec![value(0), value(1)]), None);
        assert_eq!(palette.material_count(), 0);
    }

    #[test]
    fn array_property_by_name_indexes_and_survives_gc() {
        let mut palette = VoxPalette::default();
        let color = palette.add_array_property("baseColorFactor".to_owned(), pool(0));
        let metal = palette.add_array_property("metallicFactor".to_owned(), pool(1));

        assert_eq!(
            palette.array_property_by_name("baseColorFactor"),
            Some(color)
        );
        assert_eq!(
            palette.array_property_by_name("metallicFactor"),
            Some(metal)
        );
        assert_eq!(palette.array_property_by_name("missing"), None);

        // Removing a property drops it from the index; gc renumbers the rest
        // and the index follows.
        palette.remove_array_property(color);
        assert_eq!(palette.array_property_by_name("baseColorFactor"), None);
        palette.gc();
        let metal = U32Id::<BVoxArrayProperty>::from_u32(0);
        assert_eq!(
            palette.array_property_by_name("metallicFactor"),
            Some(metal)
        );
        assert_eq!(palette.array_property_by_name("baseColorFactor"), None);
    }

    #[test]
    fn add_array_property_back_fills_existing_materials_with_zero() {
        let mut palette = VoxPalette::default();
        let color = palette.add_array_property("baseColorFactor".to_owned(), pool(0));
        let material = palette.add_material(vec![value(7)]).unwrap();

        let added = palette.add_array_property("metallicFactor".to_owned(), pool(1));
        assert_eq!(palette.value_id(material, color), Some(value(7)));
        assert_eq!(palette.value_id(material, added), Some(value(0)));
    }

    #[test]
    fn clone_palette_is_an_independent_deep_copy() {
        let mut palette = VoxPalette::default();
        let property = palette.add_array_property("baseColorFactor".to_owned(), pool(0));
        let material = palette.add_material(vec![value(2)]).unwrap();

        let copy = palette.clone_palette();
        assert_eq!(copy.value_id(material, property), Some(value(2)));
        assert_eq!(
            copy.array_property(property).unwrap().name,
            "baseColorFactor"
        );

        // Mutating the original must not touch the copy.
        palette.add_material(vec![value(5)]).unwrap();
        assert_eq!(palette.material_count(), 2);
        assert_eq!(copy.material_count(), 1);
    }

    #[test]
    fn remove_array_property_keeps_materials_then_gc_renumbers() {
        let mut palette = VoxPalette::default();
        let a = palette.add_array_property("a".to_owned(), pool(0));
        let b = palette.add_array_property("b".to_owned(), pool(1));
        let material = palette.add_material(vec![value(1), value(2)]).unwrap();

        assert_eq!(palette.remove_array_property(a), Some(()));
        assert_eq!(palette.array_property_count(), 1);
        assert_eq!(palette.array_property(a), None); // a hole until gc
        assert_eq!(palette.value_id(material, a), None);
        assert_eq!(palette.value_id(material, b), Some(value(2)));
        assert_eq!(palette.remove_array_property(a), None); // already gone

        palette.gc();
        // The surviving property and material renumber to 0.
        let property = U32Id::<BVoxArrayProperty>::from_u32(0);
        let material = U32Id::<BVoxMaterial>::from_u32(0);
        assert_eq!(palette.array_property(property).unwrap().name, "b");
        assert_eq!(palette.value_id(material, property), Some(value(2)));
    }

    #[test]
    fn remove_material_then_gc_compacts_remaining_materials() {
        let mut palette = VoxPalette::default();
        let property = palette.add_array_property("v".to_owned(), pool(0));
        let keep = palette.add_material(vec![value(0)]).unwrap();
        let drop = palette.add_material(vec![value(1)]).unwrap();
        let last = palette.add_material(vec![value(2)]).unwrap();

        assert_eq!(palette.remove_material(drop), Some(()));
        assert_eq!(palette.material_count(), 2);
        assert!(!palette.contains_material(drop));
        assert!(palette.contains_material(keep) && palette.contains_material(last));
        assert_eq!(palette.remove_material(drop), None); // already gone

        palette.gc();
        // The two survivors are contiguous; their value ids are intact.
        let value_ids: Vec<_> = palette
            .iter_materials()
            .map(|material| palette.value_id(material, property).unwrap())
            .collect();
        assert_eq!(palette.material_count(), 2);
        assert_eq!(value_ids, [value(0), value(2)]);
    }
}
