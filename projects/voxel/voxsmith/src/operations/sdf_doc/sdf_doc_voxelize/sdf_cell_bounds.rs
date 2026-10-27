use ty_math::TyVector3I32;

/// The box around a set of cells, by the indices of its least and greatest
/// cells.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SdfCellBounds {
    /// The least cell index along each axis.
    pub min: TyVector3I32,

    /// The greatest cell index along each axis.
    pub max: TyVector3I32,
}

impl SdfCellBounds {
    /// The box around this box and `cell`.
    pub fn with(self, cell: TyVector3I32) -> Self {
        Self {
            min: self.min.min(cell),
            max: self.max.max(cell),
        }
    }
}
