use crate::CellBox;

/// A box of cells within one of an object's tiles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileBox {
    pub cells: CellBox,

    /// The tile's place among its object's tiles of the box's kind.
    pub tile: usize,
}
