use crate::{BVoxAttribute, BVoxPaletteCell, VoxValue};
use branded_id::{
    U32Id,
    soa::{IdField, IdStruct},
};

/// A palette: a set of attributes and a set of cells, where every cell carries
/// one [`VoxValue`](crate::VoxValue) per attribute (a rectangular
/// cells-by-attributes grid). An object's voxels sample cells (see
/// [`VoxObject`](crate::VoxObject)).
///
/// Build it with [`add_attribute`](Self::add_attribute) then
/// [`add_cell`](Self::add_cell). Fields are private because the columns must stay
/// in lockstep with their id pools.
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
    use crate::{VoxPalette, VoxValue};

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
}
