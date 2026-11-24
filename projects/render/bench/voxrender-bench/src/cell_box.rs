use ty_math::TyVector3U32;

/// A box of cells: its low cell and the corner past its high cell.
pub type CellBox = (TyVector3U32, TyVector3U32);
