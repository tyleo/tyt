use branded_id::U32Id;
use sdfcore::BSdfMaterial;

/// One cell of a grid.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SdfCell {
    /// The material the cell holds, or `None` for an empty cell.
    pub material: Option<U32Id<BSdfMaterial>>,

    /// The index in the object's step list of the last step that changed the
    /// cell, or `None` for a cell no step changed.
    pub step: Option<u32>,
}
