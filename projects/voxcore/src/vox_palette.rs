use crate::{BVoxAttribute, BVoxPaletteCell, VoxValue};
use branded_id::soa::{IdField, IdStruct};

/// A palette: a set of attributes plus a set of cells.
///
/// Each attribute carries a key name; each cell carries one
/// [`VoxValue`](crate::VoxValue) per attribute, so a value is
/// `palette_cells[palette_cell_id][attribute_id]`. A cell is referenced by its
/// `U32Id<BVoxPaletteCell>` and an attribute by its `U32Id<BVoxAttribute>`; an
/// object's voxels sample cells (see [`VoxObject`](crate::VoxObject)).
///
/// The columns track liveness through the id pools, so this type does not
/// derive `Clone` or `PartialEq`; its [`Drop`] releases every column.
#[derive(Debug, Default)]
pub struct VoxPalette {
    /// The attribute id pool.
    pub attribute_ids: IdStruct<BVoxAttribute>,

    /// The attribute key names, keyed by `U32Id<BVoxAttribute>`.
    pub attributes: IdField<BVoxAttribute, String>,

    /// The palette cell id pool.
    pub palette_cell_ids: IdStruct<BVoxPaletteCell>,

    /// The cells, each one [`VoxValue`](crate::VoxValue) per attribute, keyed by
    /// `U32Id<BVoxPaletteCell>` then `U32Id<BVoxAttribute>`.
    pub palette_cells: IdField<BVoxPaletteCell, IdField<BVoxAttribute, VoxValue>>,
}

impl Drop for VoxPalette {
    fn drop(&mut self) {
        // Release each cell's per-attribute values first, since `VoxValue` owns
        // heap data and the inner columns would otherwise leak it.
        for palette_cell_id in &self.palette_cell_ids {
            // Safety: every retained cell has an attribute->value column in sync
            // with `attribute_ids`.
            let cell = unsafe { self.palette_cells.get_mut(palette_cell_id) };
            unsafe { cell.release_all(&self.attribute_ids) };
        }

        // Safety: `palette_cells` and `attributes` retain a value for every id
        // in their paired pools. The fields' own destructors free the backing
        // storage, so dropping the values is all that is needed here.
        unsafe {
            self.palette_cells.release_all(&self.palette_cell_ids);
            self.attributes.release_all(&self.attribute_ids);
        }
    }
}
