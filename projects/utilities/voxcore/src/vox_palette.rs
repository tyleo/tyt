use crate::{
    BVoxMaterial, BVoxProperty, BVoxValuePool, BVoxValuePoolValue, Error, Result, VoxProperty,
};
use branded_id::{
    IdVec, U32Id,
    soa::{IdField, IdList, IdRemap, IdStruct, IdStructView},
};
use std::collections::HashMap;

/// One material's value ids, keyed by property id.
type MaterialRow = IdField<BVoxProperty, U32Id<BVoxValuePoolValue>>;

/// A material palette: named properties bound to the
/// [`VoxValuePool`](crate::VoxValuePool)s a [`VoxMain`](crate::VoxMain) holds,
/// and the materials that draw from them.
#[derive(Clone, Debug, Default)]
pub struct VoxPalette {
    /// The properties, in material-row value-id order.
    properties: IdList<BVoxProperty, VoxProperty>,

    /// Per material, one value id per property, keyed by property id. A row
    /// keeps filler at a released property's slot until [`gc`](Self::gc).
    materials: IdList<BVoxMaterial, MaterialRow>,

    /// Name index into `properties`, for O(1)
    /// [`property_id_by_name`](Self::property_id_by_name) lookup. Rebuilt by
    /// [`gc`](Self::gc). Doubles as the uniqueness check
    /// [`retain_property`](Self::retain_property) makes.
    property_id_by_name: HashMap<String, U32Id<BVoxProperty>>,
}

impl VoxPalette {
    /// Compacts the property and material id pools back to a contiguous
    /// `0..len`, moving every value to its relabeled id, and returns the
    /// material relabeling so a [`VoxMain`](crate::VoxMain) can translate the
    /// samples that point at these materials. Properties are referenced only
    /// within this palette; their relabelings stay internal. Value ids stay
    /// valid: they point into the referenced value pools, whose contents gc
    /// does not touch.
    pub(crate) fn gc(&mut self) -> IdRemap<BVoxMaterial, u32> {
        // Each row's cells in property listing order. The property gc
        // renumbers ids in that order.
        let rows_value_ids: Vec<Vec<_>> = self
            .material_rows()
            .map(|row| row.iter().map(|(_, value_id)| *value_id).collect())
            .collect();

        self.properties.gc();

        for ((_, row), value_ids) in self.materials.iter_mut().zip(rows_value_ids) {
            *row = material_row_in_order(self.properties.ids(), value_ids);
        }

        let material_remap = self.materials.gc();

        // Rebuild the name index against the relabeled property ids.
        self.property_id_by_name.clear();

        for (property_id, property) in self.properties.iter() {
            self.property_id_by_name
                .insert(property.name.clone(), property_id);
        }

        material_remap
    }

    /// Retains a material with one value id per property, in
    /// [`iter_properties`](Self::iter_properties) order, and returns its id.
    /// Errors, changing nothing, if `value_ids` has the wrong length. Each
    /// value id must be one of its property's value pool's values, which
    /// [`VoxMain::retain_palette`](crate::VoxMain::retain_palette) checks on
    /// insert.
    pub fn retain_material(
        &mut self,
        value_ids: Vec<U32Id<BVoxValuePoolValue>>,
    ) -> Result<U32Id<BVoxMaterial>> {
        if value_ids.len() != self.properties.len() {
            return Err(Error::MaterialValueArity {
                values: value_ids.len(),
                properties: self.properties.len(),
            });
        }

        let row = material_row_in_order(self.properties.ids(), value_ids);

        Ok(self.materials.retain(row))
    }

    /// Drops material `id` and its value-id row. Returns `None`, changing
    /// nothing, if `id` is not one of this palette's materials. The caller must
    /// first ensure no live voxel still samples it. Leaves a hole until
    /// [`gc`](Self::gc) renumbers.
    pub(crate) fn release_material(&mut self, id: U32Id<BVoxMaterial>) -> Option<()> {
        self.materials.release_stable(id)?;

        Some(())
    }

    /// Whether `id` is one of this palette's materials.
    pub fn contains_material(&self, id: U32Id<BVoxMaterial>) -> bool {
        self.materials.ids().is_retained(id)
    }

    /// Material ids in listing order; read value ids with
    /// [`value_id`](Self::value_id).
    pub fn iter_materials(&self) -> impl Iterator<Item = U32Id<BVoxMaterial>> + '_ {
        self.materials.ids().iter()
    }

    /// Number of materials.
    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    /// Moves material `id` to position `index` in the material order, shifting
    /// the materials between its old and new positions one slot. Errors,
    /// changing nothing, if `id` is not one of this palette's materials or
    /// `index` is at or past [`material_count`](Self::material_count).
    pub fn move_material(&mut self, id: U32Id<BVoxMaterial>, index: usize) -> Result<()> {
        if !self.contains_material(id) {
            return Err(Error::UnknownMaterial { material_id: id });
        }

        let count = self.materials.len();

        if index >= count {
            return Err(Error::IndexPastCount { index, count });
        }

        self.materials.move_to(id, index);

        Ok(())
    }

    /// Retains a property after any existing ones and returns its id,
    /// back-filling existing materials with `default_value_id` so every
    /// material keeps one value id per property. Errors, changing nothing, if a
    /// property already has this name. `default_value_id` must be one of
    /// `value_pool_id`'s values, which
    /// [`VoxMain::retain_palette`](crate::VoxMain::retain_palette) checks on
    /// insert.
    pub fn retain_property(
        &mut self,
        name: String,
        value_pool_id: U32Id<BVoxValuePool>,
        default_value_id: U32Id<BVoxValuePoolValue>,
    ) -> Result<U32Id<BVoxProperty>> {
        if self.property_id_by_name.contains_key(&name) {
            return Err(Error::DuplicatePropertyName { name });
        }

        let property_id = self.properties.retain(VoxProperty {
            name: name.clone(),
            value_pool_id,
        });

        self.property_id_by_name.insert(name, property_id);

        for (_, row) in self.materials.iter_mut() {
            row.retain(property_id, default_value_id);
        }

        Ok(property_id)
    }

    /// Releases property `id`. Errors, changing nothing, if `id` is not one of
    /// this palette's properties. Each material row keeps filler at the
    /// released slot until [`VoxMain::gc`](crate::VoxMain::gc) renumbers.
    pub fn release_property(&mut self, id: U32Id<BVoxProperty>) -> Result<()> {
        let Some(property) = self.properties.release_stable(id) else {
            return Err(Error::UnknownProperty { property_id: id });
        };

        // Drop the index entry if it still points here. A duplicate name may
        // have overwritten it.
        if self.property_id_by_name.get(&property.name) == Some(&id) {
            self.property_id_by_name.remove(&property.name);
        }

        Ok(())
    }

    /// Properties in listing order, as `(id, property)`. Property order is the
    /// value-id order of each material row.
    pub fn iter_properties(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxProperty>, &VoxProperty)> + '_ {
        self.properties.iter()
    }

    /// Moves property `id` to position `index` in the property order, shifting
    /// the properties between its old and new positions one slot. Errors,
    /// changing nothing, if `id` is not one of this palette's properties or
    /// `index` is at or past [`property_count`](Self::property_count).
    pub fn move_property(&mut self, id: U32Id<BVoxProperty>, index: usize) -> Result<()> {
        if !self.properties.ids().is_retained(id) {
            return Err(Error::UnknownProperty { property_id: id });
        }

        let count = self.properties.len();

        if index >= count {
            return Err(Error::IndexPastCount { index, count });
        }

        self.properties.move_to(id, index);

        Ok(())
    }

    /// The property `id`, or `None` if not one of this palette's.
    pub fn property(&self, id: U32Id<BVoxProperty>) -> Option<&VoxProperty> {
        self.properties.get(id)
    }

    /// Number of properties.
    pub fn property_count(&self) -> usize {
        self.properties.len()
    }

    /// The property named `name`, or `None` if none has that name. O(1) through
    /// the name index.
    pub fn property_id_by_name(&self, name: &str) -> Option<U32Id<BVoxProperty>> {
        self.property_id_by_name.get(name).copied()
    }

    /// The value id `material_id` draws for `property_id`, identifying a value
    /// in the value pool that property draws from, or `None` if either id is
    /// not this palette's. Read the value pool a [`VoxMain`](crate::VoxMain)
    /// holds by that id for the value.
    pub fn value_id(
        &self,
        material_id: U32Id<BVoxMaterial>,
        property_id: U32Id<BVoxProperty>,
    ) -> Option<U32Id<BVoxValuePoolValue>> {
        let row = self.material_row(material_id)?;

        let value_id = row.get(property_id)?;

        Some(*value_id)
    }

    /// Points `material_id`'s cell for `property_id` at `value_id`. Errors,
    /// changing nothing, if either id is not this palette's. `value_id` must be
    /// one of the property's value pool's values, which
    /// [`VoxMain::set_material_value`](crate::VoxMain::set_material_value)
    /// checks.
    pub fn set_value_id(
        &mut self,
        material_id: U32Id<BVoxMaterial>,
        property_id: U32Id<BVoxProperty>,
        value_id: U32Id<BVoxValuePoolValue>,
    ) -> Result<()> {
        let Some(row) = self.material_row_mut(material_id) else {
            return Err(Error::UnknownMaterial { material_id });
        };

        let Some(cell) = row.into_mut(property_id) else {
            return Err(Error::UnknownProperty { property_id });
        };

        *cell = value_id;

        Ok(())
    }

    /// Translates each material's cells through the value relabeling of the
    /// value pool its property draws from, matching value pools a
    /// [`VoxMain`](crate::VoxMain) is compacting. `remaps` is indexed by the
    /// value pool's pre-gc id. Requires a referentially valid palette, so every
    /// cell draws a live value.
    pub(crate) fn relabel_value_pool_values(
        &mut self,
        remaps: &IdVec<BVoxValuePool, IdRemap<BVoxValuePoolValue, u32>>,
    ) {
        // Each property's value pool, found once so each material's row is
        // visited once for all of them.
        let property_value_pool_ids: Vec<_> = self
            .properties
            .iter()
            .map(|(property_id, property)| (property_id, property.value_pool_id))
            .collect();

        for mut row in self.material_rows_mut() {
            for &(property_id, value_pool_id) in &property_value_pool_ids {
                let cell = row
                    .get_mut(property_id)
                    .expect("a row has a cell for every property");

                *cell = remaps[value_pool_id.to_usize_id()]
                    .new_id(*cell)
                    .expect("a material cell draws a live value in a valid state");
            }
        }
    }

    /// Translates every property's value-pool id through `value_pool_id`, for
    /// a palette moving to another [`VoxMain`](crate::VoxMain)'s value pools.
    /// [`VoxMain::retain_palette`](crate::VoxMain::retain_palette) checks the
    /// new ids on insert.
    pub fn relabel_value_pools(
        &mut self,
        mut value_pool_id: impl FnMut(U32Id<BVoxValuePool>) -> U32Id<BVoxValuePool>,
    ) {
        for (_, property) in self.properties.iter_mut() {
            property.value_pool_id = value_pool_id(property.value_pool_id);
        }
    }

    /// Repoints each material's cell for a property on `value_pool_id` that
    /// draws `old_id` to `new_id`. Used by [`repoint_value_pool_value`].
    ///
    /// [`repoint_value_pool_value`]: crate::VoxMain::repoint_value_pool_value
    pub(crate) fn repoint_value_pool_value(
        &mut self,
        value_pool_id: U32Id<BVoxValuePool>,
        old_id: U32Id<BVoxValuePoolValue>,
        new_id: U32Id<BVoxValuePoolValue>,
    ) {
        // The properties on `value_pool_id`, found once so each material's row
        // is visited once for all of them.
        let value_pool_property_ids: Vec<_> = self
            .properties
            .iter()
            .filter(|(_, property)| property.value_pool_id == value_pool_id)
            .map(|(property_id, _)| property_id)
            .collect();

        if value_pool_property_ids.is_empty() {
            return;
        }

        for mut row in self.material_rows_mut() {
            for &property_id in &value_pool_property_ids {
                let cell = row
                    .get_mut(property_id)
                    .expect("a row has a cell for every property");

                if *cell == old_id {
                    *cell = new_id;
                }
            }
        }
    }

    /// Material `material_id`'s cells by property, or `None` if it is not one
    /// of this palette's materials.
    fn material_row(
        &self,
        material_id: U32Id<BVoxMaterial>,
    ) -> Option<IdStructView<'_, BVoxProperty, &MaterialRow>> {
        let row = self.materials.get(material_id)?;

        // Safety: every material row holds a value id for every property.
        Some(unsafe { self.properties.ids().view(row) })
    }

    /// Material `material_id`'s cells by property, writable, or `None` if it
    /// is not one of this palette's materials.
    fn material_row_mut(
        &mut self,
        material_id: U32Id<BVoxMaterial>,
    ) -> Option<IdStructView<'_, BVoxProperty, &mut MaterialRow>> {
        let row = self.materials.get_mut(material_id)?;

        // Safety: every material row holds a value id for every property.
        Some(unsafe { self.properties.ids().view(row) })
    }

    /// Every material's cells by property, in material order.
    fn material_rows(&self) -> impl Iterator<Item = IdStructView<'_, BVoxProperty, &MaterialRow>> {
        let property_ids = self.properties.ids();

        self.materials.iter().map(move |(_, row)| {
            // Safety: every material row holds a value id for every property.
            unsafe { property_ids.view(row) }
        })
    }

    /// Every material's cells by property, writable, in material order.
    fn material_rows_mut(
        &mut self,
    ) -> impl Iterator<Item = IdStructView<'_, BVoxProperty, &mut MaterialRow>> {
        let property_ids = self.properties.ids();

        self.materials.iter_mut().map(move |(_, row)| {
            // Safety: every material row holds a value id for every property.
            unsafe { property_ids.view(row) }
        })
    }
}

/// A material row holding `value_ids` in `property_ids` listing order, one per
/// property.
fn material_row_in_order(
    property_ids: &IdStruct<BVoxProperty>,
    value_ids: Vec<U32Id<BVoxValuePoolValue>>,
) -> MaterialRow {
    assert_eq!(
        value_ids.len(),
        property_ids.len(),
        "a material row holds one value id per property"
    );

    let mut row = IdField::new();

    for (property_id, value_id) in property_ids.iter().zip(value_ids) {
        row.retain(property_id, value_id);
    }

    row
}

#[cfg(test)]
mod tests {
    use crate::{BVoxMaterial, BVoxProperty, BVoxValuePool, BVoxValuePoolValue, Error, VoxPalette};
    use branded_id::U32Id;

    fn value_pool_id(index: u32) -> U32Id<BVoxValuePool> {
        U32Id::from_u32(index)
    }

    fn value_id(index: u32) -> U32Id<BVoxValuePoolValue> {
        U32Id::from_u32(index)
    }

    #[test]
    fn builds_and_reads_a_material_palette() {
        let mut palette = VoxPalette::default();
        let metallic_id = palette
            .retain_property("metallic_id".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        let ior_id = palette
            .retain_property("ior_id".to_owned(), value_pool_id(1), value_id(0))
            .unwrap();

        // Two materials, each a value id per property, in property order.
        let matte_id = palette
            .retain_material(vec![value_id(0), value_id(3)])
            .unwrap();

        let shiny_id = palette
            .retain_material(vec![value_id(1), value_id(3)])
            .unwrap();

        assert_eq!(palette.property_count(), 2);
        assert_eq!(palette.material_count(), 2);
        assert_eq!(palette.property(metallic_id).unwrap().name, "metallic_id");
        assert_eq!(
            palette.property(ior_id).unwrap().value_pool_id,
            value_pool_id(1)
        );
        assert_eq!(palette.value_id(matte_id, metallic_id), Some(value_id(0)));
        assert_eq!(palette.value_id(matte_id, ior_id), Some(value_id(3)));
        assert_eq!(palette.value_id(shiny_id, metallic_id), Some(value_id(1)));
        assert_eq!(
            palette
                .iter_properties()
                .map(|(property_id, property)| (property_id, property.name.as_str()))
                .collect::<Vec<_>>(),
            [(metallic_id, "metallic_id"), (ior_id, "ior_id")]
        );
    }

    #[test]
    fn retain_material_rejects_wrong_arity_without_changing_state() {
        let mut palette = VoxPalette::default();
        palette
            .retain_property("baseColor".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        // One property, but two value ids supplied.
        assert_eq!(
            palette.retain_material(vec![value_id(0), value_id(1)]),
            Err(Error::MaterialValueArity {
                values: 2,
                properties: 1
            })
        );
        assert_eq!(palette.material_count(), 0);
    }

    #[test]
    fn set_value_id_points_one_cell() {
        let mut palette = VoxPalette::default();
        let color_id = palette
            .retain_property("baseColor".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        let roughness_id = palette
            .retain_property("roughness".to_owned(), value_pool_id(1), value_id(0))
            .unwrap();

        let a_id = palette
            .retain_material(vec![value_id(0), value_id(0)])
            .unwrap();

        let b_id = palette
            .retain_material(vec![value_id(0), value_id(0)])
            .unwrap();

        assert_eq!(
            palette.set_value_id(b_id, roughness_id, value_id(2)),
            Ok(())
        );

        assert_eq!(palette.value_id(b_id, roughness_id), Some(value_id(2)));
        assert_eq!(palette.value_id(b_id, color_id), Some(value_id(0)));
        assert_eq!(palette.value_id(a_id, roughness_id), Some(value_id(0)));

        let unknown_material_id = U32Id::<BVoxMaterial>::from_u32(9);
        assert_eq!(
            palette.set_value_id(unknown_material_id, roughness_id, value_id(1)),
            Err(Error::UnknownMaterial {
                material_id: unknown_material_id
            })
        );

        let unknown_property_id = U32Id::<BVoxProperty>::from_u32(9);
        assert_eq!(
            palette.set_value_id(a_id, unknown_property_id, value_id(1)),
            Err(Error::UnknownProperty {
                property_id: unknown_property_id
            })
        );
    }

    #[test]
    fn property_id_by_name_indexes_and_survives_gc() {
        let mut palette = VoxPalette::default();
        let color_id = palette
            .retain_property("baseColor".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        let metal_id = palette
            .retain_property("metallic".to_owned(), value_pool_id(1), value_id(0))
            .unwrap();

        assert_eq!(palette.property_id_by_name("baseColor"), Some(color_id));
        assert_eq!(palette.property_id_by_name("metallic"), Some(metal_id));
        assert_eq!(palette.property_id_by_name("missing"), None);

        // Releasing a property drops it from the index; gc renumbers the rest
        // and the index follows.
        palette.release_property(color_id).unwrap();
        assert_eq!(palette.property_id_by_name("baseColor"), None);

        palette.gc();
        let metal_id = U32Id::<BVoxProperty>::from_u32(0);
        assert_eq!(palette.property_id_by_name("metallic"), Some(metal_id));
        assert_eq!(palette.property_id_by_name("baseColor"), None);
    }

    #[test]
    fn retain_property_rejects_a_name_already_in_use() {
        let mut palette = VoxPalette::default();
        let first_id = palette
            .retain_property("baseColor".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        // A second property under the same name, even on a different value
        // pool.
        assert_eq!(
            palette.retain_property("baseColor".to_owned(), value_pool_id(1), value_id(0)),
            Err(Error::DuplicatePropertyName {
                name: "baseColor".to_owned()
            })
        );
        assert_eq!(palette.property_count(), 1);
        assert_eq!(palette.property_id_by_name("baseColor"), Some(first_id));
        assert_eq!(
            palette.property(first_id).unwrap().value_pool_id,
            value_pool_id(0)
        );
    }

    #[test]
    fn retain_property_back_fills_existing_materials_with_the_default() {
        let mut palette = VoxPalette::default();
        let color_id = palette
            .retain_property("baseColor".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        let material_id = palette.retain_material(vec![value_id(7)]).unwrap();

        let added_id = palette
            .retain_property("metallic".to_owned(), value_pool_id(1), value_id(3))
            .unwrap();

        assert_eq!(palette.value_id(material_id, color_id), Some(value_id(7)));
        assert_eq!(palette.value_id(material_id, added_id), Some(value_id(3)));
    }

    #[test]
    fn release_property_keeps_materials_then_gc_renumbers() {
        let mut palette = VoxPalette::default();
        let a_id = palette
            .retain_property("a".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        let b_id = palette
            .retain_property("b".to_owned(), value_pool_id(1), value_id(0))
            .unwrap();

        let material_id = palette
            .retain_material(vec![value_id(1), value_id(2)])
            .unwrap();

        assert_eq!(palette.release_property(a_id), Ok(()));
        assert_eq!(palette.property_count(), 1);
        assert_eq!(palette.property(a_id), None); // a_id hole until gc
        assert_eq!(palette.value_id(material_id, a_id), None);
        assert_eq!(palette.value_id(material_id, b_id), Some(value_id(2)));
        assert_eq!(
            palette.release_property(a_id),
            Err(Error::UnknownProperty { property_id: a_id })
        ); // already gone

        palette.gc();

        // The surviving property and material renumber to 0.
        let property_id = U32Id::<BVoxProperty>::from_u32(0);
        let material_id = U32Id::<BVoxMaterial>::from_u32(0);
        assert_eq!(palette.property(property_id).unwrap().name, "b");
        assert_eq!(
            palette.value_id(material_id, property_id),
            Some(value_id(2))
        );
    }

    #[test]
    fn release_property_preserves_the_survivors_order() {
        let mut palette = VoxPalette::default();
        let a_id = palette
            .retain_property("a".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        let b_id = palette
            .retain_property("b".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        let c_id = palette
            .retain_property("c".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        // Releasing the first of three is the smallest case a swap-remove would
        // get wrong, listing `c_id` before `b_id`.
        assert_eq!(palette.release_property(a_id), Ok(()));
        assert_eq!(
            palette
                .iter_properties()
                .map(|(property_id, property)| (property_id, property.name.as_str()))
                .collect::<Vec<_>>(),
            [(b_id, "b"), (c_id, "c")]
        );

        // A property retained after the release appends at the end of the
        // order.
        let d_id = palette
            .retain_property("d".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        assert_eq!(
            palette
                .iter_properties()
                .map(|(property_id, _)| property_id)
                .collect::<Vec<_>>(),
            [b_id, c_id, d_id]
        );
    }

    #[test]
    fn release_material_preserves_the_survivors_order() {
        let mut palette = VoxPalette::default();
        let first_id = palette.retain_material(vec![]).unwrap();
        let middle_id = palette.retain_material(vec![]).unwrap();
        let last_id = palette.retain_material(vec![]).unwrap();

        // Releasing the first of three is the smallest case a swap-remove would
        // get wrong, listing `last_id` before `middle_id`.
        assert_eq!(palette.release_material(first_id), Some(()));
        assert_eq!(
            palette.iter_materials().collect::<Vec<_>>(),
            [middle_id, last_id]
        );

        // A material retained after the release appends at the end of the
        // order.
        let added_id = palette.retain_material(vec![]).unwrap();
        assert_eq!(
            palette.iter_materials().collect::<Vec<_>>(),
            [middle_id, last_id, added_id]
        );
    }

    #[test]
    fn move_property_reorders_the_listing_and_validates() {
        let mut palette = VoxPalette::default();
        let a_id = palette
            .retain_property("a".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        let b_id = palette
            .retain_property("b".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        let c_id = palette
            .retain_property("c".to_owned(), value_pool_id(1), value_id(0))
            .unwrap();

        assert_eq!(
            palette
                .iter_properties()
                .map(|(property_id, _)| property_id)
                .collect::<Vec<_>>(),
            [a_id, b_id, c_id]
        );

        assert_eq!(palette.move_property(c_id, 0), Ok(()));
        assert_eq!(
            palette
                .iter_properties()
                .map(|(property_id, _)| property_id)
                .collect::<Vec<_>>(),
            [c_id, a_id, b_id]
        );

        // An out-of-range index and an unknown id are rejected.
        assert_eq!(
            palette.move_property(c_id, 3),
            Err(Error::IndexPastCount { index: 3, count: 3 })
        );
        assert_eq!(
            palette.move_property(U32Id::from_u32(9), 0),
            Err(Error::UnknownProperty {
                property_id: U32Id::from_u32(9)
            })
        );
        assert_eq!(
            palette
                .iter_properties()
                .map(|(property_id, _)| property_id)
                .collect::<Vec<_>>(),
            [c_id, a_id, b_id]
        );
    }

    #[test]
    fn move_material_reorders_the_listing_and_validates() {
        let mut palette = VoxPalette::default();
        let first_id = palette.retain_material(vec![]).unwrap();
        let second_id = palette.retain_material(vec![]).unwrap();
        let third_id = palette.retain_material(vec![]).unwrap();
        assert_eq!(
            palette.iter_materials().collect::<Vec<_>>(),
            [first_id, second_id, third_id]
        );

        assert_eq!(palette.move_material(first_id, 2), Ok(()));
        assert_eq!(
            palette.iter_materials().collect::<Vec<_>>(),
            [second_id, third_id, first_id]
        );

        // An out-of-range index and an unknown id are rejected.
        assert_eq!(
            palette.move_material(first_id, 3),
            Err(Error::IndexPastCount { index: 3, count: 3 })
        );
        assert_eq!(
            palette.move_material(U32Id::from_u32(9), 0),
            Err(Error::UnknownMaterial {
                material_id: U32Id::from_u32(9)
            })
        );
        assert_eq!(
            palette.iter_materials().collect::<Vec<_>>(),
            [second_id, third_id, first_id]
        );
    }

    #[test]
    fn gc_after_move_property_keeps_each_cell_with_its_property() {
        let mut palette = VoxPalette::default();

        palette
            .retain_property("a".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        let b_id = palette
            .retain_property("b".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        let c_id = palette
            .retain_property("c".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        palette
            .retain_material(vec![value_id(10), value_id(11), value_id(12)])
            .unwrap();

        palette
            .retain_material(vec![value_id(20), value_id(21), value_id(22)])
            .unwrap();

        // List c before a, so the ids no longer ascend in listing order.
        assert_eq!(palette.move_property(c_id, 0), Ok(()));
        assert_eq!(palette.release_property(b_id), Ok(()));

        palette.gc();

        let c_id = U32Id::<BVoxProperty>::from_u32(0);
        let a_id = U32Id::<BVoxProperty>::from_u32(1);

        let properties: Vec<_> = palette
            .iter_properties()
            .map(|(property_id, property)| (property_id, property.name.as_str()))
            .collect();

        assert_eq!(properties, [(c_id, "c"), (a_id, "a")]);
        assert_eq!(palette.property_id_by_name("a"), Some(a_id));

        let rows: Vec<_> = palette
            .iter_materials()
            .map(|material_id| {
                [
                    palette.value_id(material_id, c_id),
                    palette.value_id(material_id, a_id),
                ]
            })
            .collect();

        assert_eq!(
            rows,
            [
                [Some(value_id(12)), Some(value_id(10))],
                [Some(value_id(22)), Some(value_id(20))]
            ]
        );
    }

    #[test]
    fn release_material_then_gc_compacts_remaining_materials() {
        let mut palette = VoxPalette::default();
        let property_id = palette
            .retain_property("v".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        let keep_id = palette.retain_material(vec![value_id(0)]).unwrap();
        let drop_id = palette.retain_material(vec![value_id(1)]).unwrap();
        let last_id = palette.retain_material(vec![value_id(2)]).unwrap();

        assert_eq!(palette.release_material(drop_id), Some(()));
        assert_eq!(palette.material_count(), 2);
        assert!(!palette.contains_material(drop_id));
        assert!(palette.contains_material(keep_id) && palette.contains_material(last_id));
        assert_eq!(palette.release_material(drop_id), None); // already gone

        palette.gc();

        // The two survivors are contiguous; their value ids are intact.
        let value_ids: Vec<_> = palette
            .iter_materials()
            .map(|material_id| palette.value_id(material_id, property_id).unwrap())
            .collect();

        assert_eq!(palette.material_count(), 2);
        assert_eq!(value_ids, [value_id(0), value_id(2)]);
    }

    #[test]
    fn a_clone_keeps_ids_and_holes() {
        let mut palette = VoxPalette::default();
        let property_id = palette
            .retain_property("a".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();

        let released_id = palette.retain_material(vec![value_id(1)]).unwrap();
        let kept_id = palette.retain_material(vec![value_id(2)]).unwrap();
        palette.release_material(released_id).unwrap();

        let copy = palette.clone();

        assert_eq!(copy.iter_materials().collect::<Vec<_>>(), [kept_id]);
        assert!(!copy.contains_material(released_id));
        assert_eq!(copy.value_id(kept_id, property_id), Some(value_id(2)));
        assert_eq!(copy.property_id_by_name("a"), Some(property_id));
    }

    #[test]
    fn relabel_value_pools_moves_each_property() {
        let mut palette = VoxPalette::default();
        let a_id = palette
            .retain_property("a".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();
        let b_id = palette
            .retain_property("b".to_owned(), value_pool_id(1), value_id(0))
            .unwrap();

        palette.relabel_value_pools(|value_pool_id| U32Id::from_u32(value_pool_id.to_u32() + 5));

        assert_eq!(
            palette.property(a_id).unwrap().value_pool_id,
            value_pool_id(5)
        );
        assert_eq!(
            palette.property(b_id).unwrap().value_pool_id,
            value_pool_id(6)
        );
    }
}
