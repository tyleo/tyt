use crate::{
    BVoxMaterial, BVoxProperty, BVoxValuePool, BVoxValuePoolValue, Error, Result, VoxProperty,
};
use branded_id::{
    IdVec, U32Id,
    soa::{IdField, IdRemap, IdStruct},
};
use std::collections::HashMap;

/// A material palette: named properties bound to the
/// [`VoxValuePool`](crate::VoxValuePool)s a [`VoxMain`](crate::VoxMain)
/// holds, and the materials that draw from them.
#[derive(Debug, Default)]
pub struct VoxPalette {
    /// Property id pool.
    property_ids: IdStruct<BVoxProperty>,

    /// The properties, in material-row value-id order.
    properties: IdField<BVoxProperty, VoxProperty>,

    /// Material id pool.
    material_ids: IdStruct<BVoxMaterial>,

    /// Per material, one value id per property.
    materials: IdField<BVoxMaterial, IdField<BVoxProperty, U32Id<BVoxValuePoolValue>>>,

    /// Name index into `properties`, for O(1)
    /// [`property_by_name`](Self::property_by_name) lookup. Rebuilt by
    /// [`gc`](Self::gc). Doubles as the uniqueness check
    /// [`add_property`](Self::add_property) makes.
    property_by_name: HashMap<String, U32Id<BVoxProperty>>,
}

impl VoxPalette {
    /// Adds a property after any existing ones and returns its id,
    /// back-filling existing materials with `default_value` so every material
    /// keeps one value id per property. Errors, changing nothing, if a
    /// property already has this name. `default_value` must be one of
    /// `pool`'s values, which
    /// [`VoxMain::add_palette`](crate::VoxMain::add_palette) checks on
    /// insert.
    pub fn add_property(
        &mut self,
        name: String,
        pool: U32Id<BVoxValuePool>,
        default_value: U32Id<BVoxValuePoolValue>,
    ) -> Result<U32Id<BVoxProperty>> {
        if self.property_by_name.contains_key(&name) {
            return Err(Error::DuplicatePropertyName { name });
        }

        let property_id = self.property_ids.retain();

        self.property_by_name.insert(name.clone(), property_id);

        self.properties
            .retain(property_id, VoxProperty { name, pool });

        for material_id in self.material_ids.iter() {
            // Safety: retained material ids have a value row.
            let material = unsafe { self.materials.get_mut(material_id) };
            material.retain(property_id, default_value);
        }

        Ok(property_id)
    }

    /// Number of properties.
    pub fn property_count(&self) -> usize {
        self.property_ids.len()
    }

    /// The property `id`, or `None` if not one of this palette's.
    pub fn property(&self, id: U32Id<BVoxProperty>) -> Option<&VoxProperty> {
        // Safety: retained ids have a value.
        self.property_ids
            .is_retained(id)
            .then(|| unsafe { self.properties.get(id) })
    }

    /// The property named `name`, or `None` if none has that name. O(1)
    /// through the name index.
    pub fn property_by_name(&self, name: &str) -> Option<U32Id<BVoxProperty>> {
        self.property_by_name.get(name).copied()
    }

    /// Properties in listing order, as `(id, property)`. Property order is the
    /// value-id order of each material row.
    pub fn iter_properties(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxProperty>, &VoxProperty)> + '_ {
        // Safety: retained ids have a value.
        self.property_ids
            .iter()
            .map(move |id| (id, unsafe { self.properties.get(id) }))
    }

    /// Adds a material with one value id per property, in
    /// [`iter_properties`](Self::iter_properties) order, and returns its id.
    /// Errors, changing nothing, if `value_ids` has the wrong length. Each
    /// value id must be one of its property's pool's values, which
    /// [`VoxMain::add_palette`](crate::VoxMain::add_palette) checks on insert.
    pub fn add_material(
        &mut self,
        value_ids: Vec<U32Id<BVoxValuePoolValue>>,
    ) -> Result<U32Id<BVoxMaterial>> {
        if value_ids.len() != self.property_ids.len() {
            return Err(Error::MaterialValueArity {
                values: value_ids.len(),
                properties: self.property_ids.len(),
            });
        }
        let material_id = self.material_ids.retain();
        let mut row = IdField::new();
        for (property_id, value_id) in self.property_ids.iter().zip(value_ids) {
            row.retain(property_id, value_id);
        }
        self.materials.retain(material_id, row);
        Ok(material_id)
    }

    /// Number of materials.
    pub fn material_count(&self) -> usize {
        self.material_ids.len()
    }

    /// Whether `id` is one of this palette's materials.
    pub fn contains_material(&self, id: U32Id<BVoxMaterial>) -> bool {
        self.material_ids.is_retained(id)
    }

    /// The value id `material` draws for `property`, identifying a
    /// value in the pool that property draws from, or `None` if either id is
    /// not this palette's. Read the pool a [`VoxMain`](crate::VoxMain) holds
    /// by that id for the value.
    pub fn value_id(
        &self,
        material: U32Id<BVoxMaterial>,
        property: U32Id<BVoxProperty>,
    ) -> Option<U32Id<BVoxValuePoolValue>> {
        if !self.material_ids.is_retained(material) || !self.property_ids.is_retained(property) {
            return None;
        }
        // Safety: a retained material has a value id for every property.
        let row = unsafe { self.materials.get(material) };
        Some(*unsafe { row.get(property) })
    }

    /// Material ids in listing order; read value ids with
    /// [`value_id`](Self::value_id).
    pub fn iter_materials(&self) -> impl Iterator<Item = U32Id<BVoxMaterial>> + '_ {
        self.material_ids.iter()
    }

    /// Moves property `id` to position `index` in the property order, shifting
    /// the properties between its old and new positions one slot. Errors,
    /// changing nothing, if `id` is not one of this palette's properties or
    /// `index` is at or past [`property_count`](Self::property_count).
    pub fn move_property(&mut self, id: U32Id<BVoxProperty>, index: usize) -> Result<()> {
        if !self.property_ids.is_retained(id) {
            return Err(Error::UnknownProperty { property: id });
        }
        let count = self.property_ids.len();
        if index >= count {
            return Err(Error::IndexPastCount { index, count });
        }
        self.property_ids.move_to(id, index);
        Ok(())
    }

    /// The position of property `id` in the property order, or `None` if `id`
    /// is not one of this palette's properties.
    pub fn property_index(&self, id: U32Id<BVoxProperty>) -> Option<usize> {
        self.property_ids.index_of(id)
    }

    /// Moves material `id` to position `index` in the material order, shifting
    /// the materials between its old and new positions one slot. Errors,
    /// changing nothing, if `id` is not one of this palette's materials or
    /// `index` is at or past [`material_count`](Self::material_count).
    pub fn move_material(&mut self, id: U32Id<BVoxMaterial>, index: usize) -> Result<()> {
        if !self.material_ids.is_retained(id) {
            return Err(Error::UnknownMaterial { material: id });
        }
        let count = self.material_ids.len();
        if index >= count {
            return Err(Error::IndexPastCount { index, count });
        }
        self.material_ids.move_to(id, index);
        Ok(())
    }

    /// The position of material `id` in the material order, or `None` if `id`
    /// is not one of this palette's materials.
    pub fn material_index(&self, id: U32Id<BVoxMaterial>) -> Option<usize> {
        self.material_ids.index_of(id)
    }

    /// Deep copy. Liveness lives in the id pools, so the columns can't derive
    /// `Clone`; rebuild them against the cloned pools.
    pub fn clone_palette(&self) -> Self {
        let mut properties = IdField::new();
        for property_id in self.property_ids.iter() {
            // Safety: retained ids have a value.
            let property = unsafe { self.properties.get(property_id) }.clone();
            properties.retain(property_id, property);
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
            property_ids: self.property_ids.clone(),
            properties,
            material_ids: self.material_ids.clone(),
            materials,
            property_by_name: self.property_by_name.clone(),
        }
    }

    /// Removes property `id`. Errors, changing nothing, if `id` is not one of
    /// this palette's properties. Each material row keeps filler at the
    /// removed slot until [`VoxMain::gc`](crate::VoxMain::gc) compacts the rows
    /// and renumbers.
    pub fn remove_property(&mut self, id: U32Id<BVoxProperty>) -> Result<()> {
        if !self.property_ids.is_retained(id) {
            return Err(Error::UnknownProperty { property: id });
        }

        // Drop the index entry still pointing here; a duplicate name may have
        // overwritten it.
        // Safety: a retained property has a value.
        let name = unsafe { self.properties.get(id) }.name.clone();
        if self.property_by_name.get(&name) == Some(&id) {
            self.property_by_name.remove(&name);
        }

        // A value id is Copy, so releasing each material's slot at `id`
        // would be a no-op; leave it for gc to compact and only free the
        // property.
        // Safety: a retained property has a value.
        unsafe { self.properties.release(id) };
        self.property_ids.release_stable(id);
        Ok(())
    }

    /// Drops material `id` and its value-id row. The caller must first ensure
    /// no live voxel still samples it. Leaves a hole until [`gc`](Self::gc)
    /// renumbers.
    pub(crate) fn remove_material(&mut self, id: U32Id<BVoxMaterial>) -> Option<()> {
        if !self.material_ids.is_retained(id) {
            return None;
        }
        // The row holds Copy value ids, so dropping the inner IdField frees
        // its buffer with nothing to release per property.
        // Safety: a retained material has a row.
        unsafe { self.materials.release(id) };
        self.material_ids.release_stable(id);
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
        let property_remap = self.property_ids.gc();
        // Safety: the property column was in sync with the pre-gc property
        // pool, and nothing has retained or released since.
        unsafe { self.properties.gc(&property_remap) };

        let material_ids: Vec<_> = self.material_ids.iter().collect();
        for material_id in material_ids {
            // Safety: a retained material holds a value id for every pre-gc
            // property id, and the remap came from this palette's
            // property pool.
            let row = unsafe { self.materials.get_mut(material_id) };
            unsafe { row.gc(&property_remap) };
        }

        let material_remap = self.material_ids.gc();
        // Safety: the material column was in sync with the pre-gc material
        // pool, and nothing has retained or released since.
        unsafe { self.materials.gc(&material_remap) };

        // Rebuild the name index against the relabeled property ids.
        self.property_by_name.clear();

        let property_ids: Vec<_> = self.property_ids.iter().collect();
        for property_id in property_ids {
            // Safety: retained property ids have a value.
            let name = unsafe { self.properties.get(property_id) }.name.clone();

            self.property_by_name.insert(name, property_id);
        }

        material_remap
    }

    /// Repoints each material's cell for a property on `pool` that draws
    /// `old` to `new`. Used by
    /// [`VoxMain::remove_pool_value`](crate::VoxMain::remove_pool_value)
    /// before `old` is released.
    pub(crate) fn repoint_pool_value(
        &mut self,
        pool: U32Id<BVoxValuePool>,
        old: U32Id<BVoxValuePoolValue>,
        new: U32Id<BVoxValuePoolValue>,
    ) {
        // The properties on `pool`, found once so each material's row
        // is visited once for all of them.
        let pool_properties: Vec<_> = self
            .property_ids
            .iter()
            .filter(|&property_id| {
                // Safety: retained property ids have a value.
                unsafe { self.properties.get(property_id) }.pool == pool
            })
            .collect();
        if !pool_properties.is_empty() {
            for material_id in self.material_ids.iter() {
                // Safety: a retained material holds a value id for every
                // property, and the row is keyed by property id.
                let row = unsafe { self.materials.get_mut(material_id) };
                for &property_id in &pool_properties {
                    let slot = unsafe { row.get_mut(property_id) };
                    if *slot == old {
                        *slot = new;
                    }
                }
            }
        }
    }

    /// Translates each material's cells through the value relabeling of the
    /// pool its property draws from, matching value pools a
    /// [`VoxMain`](crate::VoxMain) is compacting. `remaps` is indexed by the
    /// pool's pre-gc id. Requires a referentially valid palette, so every cell
    /// draws a live value.
    pub(crate) fn relabel_pool_values(
        &mut self,
        remaps: &IdVec<BVoxValuePool, IdRemap<BVoxValuePoolValue, u32>>,
    ) {
        // Each property's pool, found once so each material's row is visited
        // once for all of them.
        let property_pools: Vec<_> = self
            .property_ids
            .iter()
            .map(|property_id| {
                // Safety: retained property ids have a value.
                (
                    property_id,
                    unsafe { self.properties.get(property_id) }.pool,
                )
            })
            .collect();
        for material_id in self.material_ids.iter() {
            // Safety: a retained material holds a value id for every property,
            // and the row is keyed by property id.
            let row = unsafe { self.materials.get_mut(material_id) };
            for &(property_id, pool) in &property_pools {
                let slot = unsafe { row.get_mut(property_id) };
                *slot = remaps[pool.to_usize_id()]
                    .new_id(*slot)
                    .expect("a material cell draws a live value in a valid state");
            }
        }
    }

    /// Translates every property's pool id through `remap`, matching a
    /// value-pool store a [`VoxMain`](crate::VoxMain) is compacting. Requires
    /// a referentially valid palette, so every property names a live pool.
    pub(crate) fn relabel_value_pools(&mut self, remap: &IdRemap<BVoxValuePool, u32>) {
        let property_ids: Vec<_> = self.property_ids.iter().collect();
        for property_id in property_ids {
            // Safety: retained property ids have a value.
            let property = unsafe { self.properties.get_mut(property_id) };
            property.pool = remap
                .new_id(property.pool)
                .expect("a property names a live value pool in a valid state");
        }
    }
}

impl Drop for VoxPalette {
    fn drop(&mut self) {
        // Each material's row is an IdField owning a heap buffer whose value
        // ids are Copy, so releasing the inner IdField frees the buffer with
        // nothing to release per property. The properties own name
        // strings, freed by releasing them.
        // Safety: each column holds a value for every id in its pool.
        unsafe {
            self.materials.release_all(&self.material_ids);
            self.properties.release_all(&self.property_ids);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BVoxMaterial, BVoxProperty, BVoxValuePool, BVoxValuePoolValue, Error, VoxPalette};
    use branded_id::U32Id;

    fn pool(index: u32) -> U32Id<BVoxValuePool> {
        U32Id::from_u32(index)
    }

    fn value(index: u32) -> U32Id<BVoxValuePoolValue> {
        U32Id::from_u32(index)
    }

    #[test]
    fn builds_and_reads_a_material_palette() {
        let mut palette = VoxPalette::default();
        let metallic = palette
            .add_property("metallic".to_owned(), pool(0), value(0))
            .unwrap();
        let ior = palette
            .add_property("ior".to_owned(), pool(1), value(0))
            .unwrap();

        // Two materials, each a value id per property, in property
        // order.
        let matte = palette.add_material(vec![value(0), value(3)]).unwrap();
        let shiny = palette.add_material(vec![value(1), value(3)]).unwrap();

        assert_eq!(palette.property_count(), 2);
        assert_eq!(palette.material_count(), 2);
        assert_eq!(palette.property(metallic).unwrap().name, "metallic");
        assert_eq!(palette.property(ior).unwrap().pool, pool(1));
        assert_eq!(palette.value_id(matte, metallic), Some(value(0)));
        assert_eq!(palette.value_id(matte, ior), Some(value(3)));
        assert_eq!(palette.value_id(shiny, metallic), Some(value(1)));
        assert_eq!(
            palette
                .iter_properties()
                .map(|(id, p)| (id, p.name.as_str()))
                .collect::<Vec<_>>(),
            [(metallic, "metallic"), (ior, "ior")]
        );
    }

    #[test]
    fn add_material_rejects_wrong_arity_without_changing_state() {
        let mut palette = VoxPalette::default();
        palette
            .add_property("baseColorFactor".to_owned(), pool(0), value(0))
            .unwrap();
        // One property, but two value ids supplied.
        assert_eq!(
            palette.add_material(vec![value(0), value(1)]),
            Err(Error::MaterialValueArity {
                values: 2,
                properties: 1
            })
        );
        assert_eq!(palette.material_count(), 0);
    }

    #[test]
    fn property_by_name_indexes_and_survives_gc() {
        let mut palette = VoxPalette::default();
        let color = palette
            .add_property("baseColorFactor".to_owned(), pool(0), value(0))
            .unwrap();
        let metal = palette
            .add_property("metallicFactor".to_owned(), pool(1), value(0))
            .unwrap();

        assert_eq!(palette.property_by_name("baseColorFactor"), Some(color));
        assert_eq!(palette.property_by_name("metallicFactor"), Some(metal));
        assert_eq!(palette.property_by_name("missing"), None);

        // Removing a property drops it from the index; gc renumbers the rest
        // and the index follows.
        palette.remove_property(color).unwrap();
        assert_eq!(palette.property_by_name("baseColorFactor"), None);
        palette.gc();
        let metal = U32Id::<BVoxProperty>::from_u32(0);
        assert_eq!(palette.property_by_name("metallicFactor"), Some(metal));
        assert_eq!(palette.property_by_name("baseColorFactor"), None);
    }

    #[test]
    fn add_property_rejects_a_name_already_in_use() {
        let mut palette = VoxPalette::default();
        let first = palette
            .add_property("baseColorFactor".to_owned(), pool(0), value(0))
            .unwrap();

        // A second property under the same name, even on a different pool.
        assert_eq!(
            palette.add_property("baseColorFactor".to_owned(), pool(1), value(0)),
            Err(Error::DuplicatePropertyName {
                name: "baseColorFactor".to_owned()
            })
        );
        assert_eq!(palette.property_count(), 1);
        assert_eq!(palette.property_by_name("baseColorFactor"), Some(first));
        assert_eq!(palette.property(first).unwrap().pool, pool(0));
    }

    #[test]
    fn add_property_back_fills_existing_materials_with_the_default() {
        let mut palette = VoxPalette::default();
        let color = palette
            .add_property("baseColorFactor".to_owned(), pool(0), value(0))
            .unwrap();
        let material = palette.add_material(vec![value(7)]).unwrap();

        let added = palette
            .add_property("metallicFactor".to_owned(), pool(1), value(3))
            .unwrap();
        assert_eq!(palette.value_id(material, color), Some(value(7)));
        assert_eq!(palette.value_id(material, added), Some(value(3)));
    }

    #[test]
    fn clone_palette_is_an_independent_deep_copy() {
        let mut palette = VoxPalette::default();
        let property = palette
            .add_property("baseColorFactor".to_owned(), pool(0), value(0))
            .unwrap();
        let material = palette.add_material(vec![value(2)]).unwrap();

        let copy = palette.clone_palette();
        assert_eq!(copy.value_id(material, property), Some(value(2)));
        assert_eq!(copy.property(property).unwrap().name, "baseColorFactor");

        // Mutating the original must not touch the copy.
        palette.add_material(vec![value(5)]).unwrap();
        assert_eq!(palette.material_count(), 2);
        assert_eq!(copy.material_count(), 1);
    }

    #[test]
    fn remove_property_keeps_materials_then_gc_renumbers() {
        let mut palette = VoxPalette::default();
        let a = palette
            .add_property("a".to_owned(), pool(0), value(0))
            .unwrap();
        let b = palette
            .add_property("b".to_owned(), pool(1), value(0))
            .unwrap();
        let material = palette.add_material(vec![value(1), value(2)]).unwrap();

        assert_eq!(palette.remove_property(a), Ok(()));
        assert_eq!(palette.property_count(), 1);
        assert_eq!(palette.property(a), None); // a hole until gc
        assert_eq!(palette.value_id(material, a), None);
        assert_eq!(palette.value_id(material, b), Some(value(2)));
        assert_eq!(
            palette.remove_property(a),
            Err(Error::UnknownProperty { property: a })
        ); // already gone

        palette.gc();
        // The surviving property and material renumber to 0.
        let property = U32Id::<BVoxProperty>::from_u32(0);
        let material = U32Id::<BVoxMaterial>::from_u32(0);
        assert_eq!(palette.property(property).unwrap().name, "b");
        assert_eq!(palette.value_id(material, property), Some(value(2)));
    }

    #[test]
    fn remove_property_preserves_the_survivors_order() {
        let mut palette = VoxPalette::default();
        let a = palette
            .add_property("a".to_owned(), pool(0), value(0))
            .unwrap();
        let b = palette
            .add_property("b".to_owned(), pool(0), value(0))
            .unwrap();
        let c = palette
            .add_property("c".to_owned(), pool(0), value(0))
            .unwrap();

        // Removing the first of three is the smallest case a swap-remove would
        // get wrong, listing `c` before `b`.
        assert_eq!(palette.remove_property(a), Ok(()));
        assert_eq!(
            palette
                .iter_properties()
                .map(|(id, property)| (id, property.name.as_str()))
                .collect::<Vec<_>>(),
            [(b, "b"), (c, "c")]
        );

        // A property added after the removal appends at the end of the order.
        let d = palette
            .add_property("d".to_owned(), pool(0), value(0))
            .unwrap();
        assert_eq!(
            palette
                .iter_properties()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            [b, c, d]
        );
    }

    #[test]
    fn remove_material_preserves_the_survivors_order() {
        let mut palette = VoxPalette::default();
        let first = palette.add_material(vec![]).unwrap();
        let middle = palette.add_material(vec![]).unwrap();
        let last = palette.add_material(vec![]).unwrap();

        // Removing the first of three is the smallest case a swap-remove would
        // get wrong, listing `last` before `middle`.
        assert_eq!(palette.remove_material(first), Some(()));
        assert_eq!(palette.iter_materials().collect::<Vec<_>>(), [middle, last]);

        // A material added after the removal appends at the end of the order.
        let added = palette.add_material(vec![]).unwrap();
        assert_eq!(
            palette.iter_materials().collect::<Vec<_>>(),
            [middle, last, added]
        );
    }

    #[test]
    fn move_property_reorders_the_listing_and_validates() {
        let mut palette = VoxPalette::default();
        let a = palette
            .add_property("a".to_owned(), pool(0), value(0))
            .unwrap();
        let b = palette
            .add_property("b".to_owned(), pool(0), value(0))
            .unwrap();
        let c = palette
            .add_property("c".to_owned(), pool(1), value(0))
            .unwrap();
        assert_eq!(palette.property_index(b), Some(1));

        assert_eq!(palette.move_property(c, 0), Ok(()));
        assert_eq!(
            palette
                .iter_properties()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            [c, a, b]
        );
        assert_eq!(palette.property_index(c), Some(0));

        // An out-of-range index and an unknown id are rejected.
        assert_eq!(
            palette.move_property(c, 3),
            Err(Error::IndexPastCount { index: 3, count: 3 })
        );
        assert_eq!(
            palette.move_property(U32Id::from_u32(9), 0),
            Err(Error::UnknownProperty {
                property: U32Id::from_u32(9)
            })
        );
        assert_eq!(palette.property_index(U32Id::from_u32(9)), None);
        assert_eq!(
            palette
                .iter_properties()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            [c, a, b]
        );
    }

    #[test]
    fn move_material_reorders_the_listing_and_validates() {
        let mut palette = VoxPalette::default();
        let first = palette.add_material(vec![]).unwrap();
        let second = palette.add_material(vec![]).unwrap();
        let third = palette.add_material(vec![]).unwrap();
        assert_eq!(palette.material_index(second), Some(1));

        assert_eq!(palette.move_material(first, 2), Ok(()));
        assert_eq!(
            palette.iter_materials().collect::<Vec<_>>(),
            [second, third, first]
        );
        assert_eq!(palette.material_index(first), Some(2));

        // An out-of-range index and an unknown id are rejected.
        assert_eq!(
            palette.move_material(first, 3),
            Err(Error::IndexPastCount { index: 3, count: 3 })
        );
        assert_eq!(
            palette.move_material(U32Id::from_u32(9), 0),
            Err(Error::UnknownMaterial {
                material: U32Id::from_u32(9)
            })
        );
        assert_eq!(palette.material_index(U32Id::from_u32(9)), None);
        assert_eq!(
            palette.iter_materials().collect::<Vec<_>>(),
            [second, third, first]
        );
    }

    #[test]
    fn remove_material_then_gc_compacts_remaining_materials() {
        let mut palette = VoxPalette::default();
        let property = palette
            .add_property("v".to_owned(), pool(0), value(0))
            .unwrap();
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
