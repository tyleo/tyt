use crate::{BVoxAttribute, BVoxPaletteCell, VoxValue};
use branded_id::{
    U32Id,
    soa::{IdField, IdRemap, IdStruct},
};

/// A palette: a set of attributes and a set of cells, where every cell carries
/// one [`VoxValue`](crate::VoxValue) per attribute (a rectangular
/// cells-by-attributes grid). An object's voxels sample cells (see
/// [`VoxObject`](crate::VoxObject)).
///
/// Build it with [`add_attribute`](Self::add_attribute) then
/// [`add_cell`](Self::add_cell).
#[derive(Debug, Default)]
pub struct VoxPalette {
    /// Attribute id pool.
    attribute_ids: IdStruct<BVoxAttribute>,

    /// Attribute key names.
    attributes: IdField<BVoxAttribute, String>,

    /// Cell id pool.
    palette_cell_ids: IdStruct<BVoxPaletteCell>,

    /// Per cell, one value per attribute.
    palette_cells: IdField<BVoxPaletteCell, IdField<BVoxAttribute, VoxValue>>,
}

impl VoxPalette {
    /// Adds an attribute after any existing ones and returns its id, back-filling
    /// existing cells with [`VoxValue::Null`] so the palette stays rectangular.
    /// Add all attributes before any cells to avoid the back-fill.
    pub fn add_attribute(&mut self, name: String) -> U32Id<BVoxAttribute> {
        let attribute_id = self.attribute_ids.retain();
        self.attributes.retain(attribute_id, name);

        for cell_id in self.palette_cell_ids.iter() {
            // Safety: retained cell ids have a value column.
            let cell = unsafe { self.palette_cells.get_mut(cell_id) };
            cell.retain(attribute_id, VoxValue::Null);
        }

        attribute_id
    }

    /// Number of attributes.
    pub fn attribute_count(&self) -> usize {
        self.attribute_ids.len()
    }

    /// Key name of attribute `id`, or `None` if not one of this palette's.
    pub fn attribute(&self, id: U32Id<BVoxAttribute>) -> Option<&str> {
        // Safety: retained ids have a value.
        self.attribute_ids
            .is_retained(id)
            .then(|| unsafe { self.attributes.get(id) }.as_str())
    }

    /// Attributes in id order, as `(id, name)`.
    pub fn iter_attributes(&self) -> impl Iterator<Item = (U32Id<BVoxAttribute>, &str)> + '_ {
        // Safety: retained ids have a value.
        self.attribute_ids
            .iter()
            .map(move |id| (id, unsafe { self.attributes.get(id) }.as_str()))
    }

    /// Adds a cell with one value per attribute, in
    /// [`iter_attributes`](Self::iter_attributes) order, and returns its id.
    /// `None`, changing nothing, if `values` has the wrong length.
    pub fn add_cell(&mut self, values: Vec<VoxValue>) -> Option<U32Id<BVoxPaletteCell>> {
        if values.len() != self.attribute_ids.len() {
            return None;
        }
        let cell_id = self.palette_cell_ids.retain();
        let mut cell = IdField::new();
        for (attribute_id, value) in self.attribute_ids.iter().zip(values) {
            cell.retain(attribute_id, value);
        }
        self.palette_cells.retain(cell_id, cell);
        Some(cell_id)
    }

    /// Number of cells.
    pub fn cell_count(&self) -> usize {
        self.palette_cell_ids.len()
    }

    /// Whether `id` is one of this palette's cells.
    pub fn contains_cell(&self, id: U32Id<BVoxPaletteCell>) -> bool {
        self.palette_cell_ids.is_retained(id)
    }

    /// Value of `cell` for `attribute`, or `None` if either id is not this
    /// palette's.
    pub fn cell_value(
        &self,
        cell: U32Id<BVoxPaletteCell>,
        attribute: U32Id<BVoxAttribute>,
    ) -> Option<&VoxValue> {
        if !self.palette_cell_ids.is_retained(cell) || !self.attribute_ids.is_retained(attribute) {
            return None;
        }
        // Safety: a retained cell has a value for every attribute.
        let column = unsafe { self.palette_cells.get(cell) };
        Some(unsafe { column.get(attribute) })
    }

    /// Cell ids in id order; read values with [`cell_value`](Self::cell_value).
    pub fn iter_cells(&self) -> impl Iterator<Item = U32Id<BVoxPaletteCell>> + '_ {
        self.palette_cell_ids.iter()
    }

    /// Deep copy. Liveness lives in the id pools, so the columns can't derive
    /// `Clone`; rebuild them and every cell's values against the cloned pools.
    pub fn clone_palette(&self) -> Self {
        let mut attributes = IdField::new();
        for attribute_id in self.attribute_ids.iter() {
            // Safety: retained ids have a value.
            let name = unsafe { self.attributes.get(attribute_id) }.clone();
            attributes.retain(attribute_id, name);
        }

        let mut palette_cells = IdField::new();
        for cell_id in self.palette_cell_ids.iter() {
            // Safety: a retained cell has a value for every attribute.
            let source = unsafe { self.palette_cells.get(cell_id) };
            let mut cell = IdField::new();
            for attribute_id in self.attribute_ids.iter() {
                let value = unsafe { source.get(attribute_id) }.clone();
                cell.retain(attribute_id, value);
            }
            palette_cells.retain(cell_id, cell);
        }

        Self {
            attribute_ids: self.attribute_ids.clone(),
            attributes,
            palette_cell_ids: self.palette_cell_ids.clone(),
            palette_cells,
        }
    }

    /// Removes attribute `id`, dropping its value from every cell so the palette
    /// stays rectangular (each cell keeps one value per remaining attribute).
    /// `None`, changing nothing, if `id` is not one of this palette's attributes.
    /// Leaves a hole until [`VoxState::gc`](crate::VoxState::gc) renumbers.
    pub fn remove_attribute(&mut self, id: U32Id<BVoxAttribute>) -> Option<()> {
        if !self.attribute_ids.is_retained(id) {
            return None;
        }
        let cell_ids: Vec<_> = self.palette_cell_ids.iter().collect();
        for cell_id in cell_ids {
            // Safety: a retained cell holds a value for every attribute, `id`
            // included.
            let cell = unsafe { self.palette_cells.get_mut(cell_id) };
            unsafe { cell.release(id) };
        }
        // Safety: a retained attribute has a name.
        unsafe { self.attributes.release(id) };
        self.attribute_ids.release(id);
        Some(())
    }

    /// Drops cell `id` and its per-attribute values. The caller must first ensure
    /// no live voxel still samples it, which is why this is internal and reached
    /// only through [`VoxState::remove_cell`](crate::VoxState::remove_cell).
    /// Leaves a hole until [`gc`](Self::gc) renumbers.
    pub(crate) fn remove_cell(&mut self, id: U32Id<BVoxPaletteCell>) -> Option<()> {
        if !self.palette_cell_ids.is_retained(id) {
            return None;
        }
        // Safety: a retained cell holds a value for every attribute; release each
        // before the inner column so `VoxValue`'s heap data is freed.
        let cell = unsafe { self.palette_cells.get_mut(id) };
        unsafe { cell.release_all(&self.attribute_ids) };
        // Safety: a retained cell has an inner column.
        unsafe { self.palette_cells.release(id) };
        self.palette_cell_ids.release(id);
        Some(())
    }

    /// Compacts the attribute and cell pools back to a contiguous `0..len`,
    /// moving every value to its relabeled id, and returns the cell relabeling so
    /// a [`VoxState`](crate::VoxState) can translate the samples that point at
    /// these cells. Attributes are referenced only within this palette, so their
    /// relabeling stays internal.
    pub(crate) fn gc(&mut self) -> IdRemap<BVoxPaletteCell, u32> {
        let attribute_remap = self.attribute_ids.gc();
        // Safety: the name column was in sync with the pre-gc attribute pool, and
        // nothing has retained or released since.
        unsafe { self.attributes.gc(&attribute_remap) };

        let cell_ids: Vec<_> = self.palette_cell_ids.iter().collect();
        for cell_id in cell_ids {
            // Safety: a retained cell holds a value for every pre-gc attribute id,
            // and the remap came from this palette's attribute pool.
            let cell = unsafe { self.palette_cells.get_mut(cell_id) };
            unsafe { cell.gc(&attribute_remap) };
        }

        let cell_remap = self.palette_cell_ids.gc();
        // Safety: the cell column was in sync with the pre-gc cell pool, and
        // nothing has retained or released since.
        unsafe { self.palette_cells.gc(&cell_remap) };
        cell_remap
    }
}

impl Drop for VoxPalette {
    fn drop(&mut self) {
        // Release each cell's values first; `VoxValue` owns heap data the inner
        // columns won't free on their own.
        for palette_cell_id in &self.palette_cell_ids {
            // Safety: a retained cell has a value for every attribute.
            let cell = unsafe { self.palette_cells.get_mut(palette_cell_id) };
            unsafe { cell.release_all(&self.attribute_ids) };
        }

        // Safety: both columns hold a value for every id in their pools.
        unsafe {
            self.palette_cells.release_all(&self.palette_cell_ids);
            self.attributes.release_all(&self.attribute_ids);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BVoxAttribute, BVoxPaletteCell, VoxPalette, VoxValue};
    use branded_id::U32Id;

    #[test]
    fn builds_and_reads_a_rectangular_palette() {
        let mut palette = VoxPalette::default();
        let metallic = palette.add_attribute("metallic".to_owned());
        let ior = palette.add_attribute("ior".to_owned());

        let cell = palette
            .add_cell(vec![VoxValue::Bool(true), VoxValue::Number(1.5)])
            .unwrap();

        assert_eq!(palette.attribute_count(), 2);
        assert_eq!(palette.cell_count(), 1);
        assert_eq!(palette.attribute(metallic), Some("metallic"));
        assert_eq!(
            palette.cell_value(cell, metallic),
            Some(&VoxValue::Bool(true))
        );
        assert_eq!(palette.cell_value(cell, ior), Some(&VoxValue::Number(1.5)));
        assert_eq!(
            palette.iter_attributes().collect::<Vec<_>>(),
            [(metallic, "metallic"), (ior, "ior")]
        );
    }

    #[test]
    fn add_cell_rejects_wrong_arity_without_changing_state() {
        let mut palette = VoxPalette::default();
        palette.add_attribute("rgba".to_owned());
        // One attribute, but two values supplied.
        assert_eq!(
            palette.add_cell(vec![VoxValue::Number(0.0), VoxValue::Number(1.0)]),
            None
        );
        assert_eq!(palette.cell_count(), 0);
    }

    #[test]
    fn add_attribute_back_fills_existing_cells_with_null() {
        let mut palette = VoxPalette::default();
        let rgba = palette.add_attribute("rgba".to_owned());
        let cell = palette.add_cell(vec![VoxValue::Number(7.0)]).unwrap();

        let added = palette.add_attribute("metallic".to_owned());
        assert_eq!(palette.cell_value(cell, rgba), Some(&VoxValue::Number(7.0)));
        assert_eq!(palette.cell_value(cell, added), Some(&VoxValue::Null));
    }

    #[test]
    fn clone_palette_is_an_independent_deep_copy() {
        let mut palette = VoxPalette::default();
        let attribute = palette.add_attribute("rgba".to_owned());
        let cell = palette
            .add_cell(vec![VoxValue::Text("red".to_owned())])
            .unwrap();

        let copy = palette.clone_palette();
        assert_eq!(
            copy.cell_value(cell, attribute),
            Some(&VoxValue::Text("red".to_owned()))
        );

        // Mutating the original must not touch the copy.
        palette
            .add_cell(vec![VoxValue::Text("blue".to_owned())])
            .unwrap();
        assert_eq!(palette.cell_count(), 2);
        assert_eq!(copy.cell_count(), 1);
    }

    #[test]
    fn remove_attribute_keeps_cells_rectangular_then_gc_renumbers() {
        let mut palette = VoxPalette::default();
        let a = palette.add_attribute("a".to_owned());
        let b = palette.add_attribute("b".to_owned());
        let cell = palette
            .add_cell(vec![VoxValue::Number(1.0), VoxValue::Number(2.0)])
            .unwrap();

        assert_eq!(palette.remove_attribute(a), Some(()));
        assert_eq!(palette.attribute_count(), 1);
        assert_eq!(palette.attribute(a), None); // a hole until gc
        assert_eq!(palette.cell_value(cell, a), None);
        assert_eq!(palette.cell_value(cell, b), Some(&VoxValue::Number(2.0)));
        assert_eq!(palette.remove_attribute(a), None); // already gone

        palette.gc();
        // The surviving attribute and cell renumber to 0.
        let attribute = U32Id::<BVoxAttribute>::from_u32(0);
        let cell = U32Id::<BVoxPaletteCell>::from_u32(0);
        assert_eq!(palette.attribute(attribute), Some("b"));
        assert_eq!(
            palette.cell_value(cell, attribute),
            Some(&VoxValue::Number(2.0))
        );
    }

    #[test]
    fn remove_cell_then_gc_compacts_remaining_cells() {
        let mut palette = VoxPalette::default();
        palette.add_attribute("v".to_owned());
        let keep = palette.add_cell(vec![VoxValue::Number(0.0)]).unwrap();
        let drop = palette.add_cell(vec![VoxValue::Number(1.0)]).unwrap();
        let last = palette.add_cell(vec![VoxValue::Number(2.0)]).unwrap();

        assert_eq!(palette.remove_cell(drop), Some(()));
        assert_eq!(palette.cell_count(), 2);
        assert!(!palette.contains_cell(drop));
        assert!(palette.contains_cell(keep) && palette.contains_cell(last));
        assert_eq!(palette.remove_cell(drop), None); // already gone

        palette.gc();
        // The two survivors are contiguous; their values are intact.
        let attribute = U32Id::<BVoxAttribute>::from_u32(0);
        let values: Vec<f64> = palette
            .iter_cells()
            .map(|cell| match palette.cell_value(cell, attribute) {
                Some(&VoxValue::Number(n)) => n,
                other => panic!("unexpected value {other:?}"),
            })
            .collect();
        assert_eq!(palette.cell_count(), 2);
        assert_eq!(values, [0.0, 2.0]);
    }
}
