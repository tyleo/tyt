use crate::{BObjectTile, CellBox};
use branded_id::UsizeId;

/// A box of cells within one of an object's tiles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileBox {
    pub cells: CellBox,

    /// The tile among its object's tiles of the box's kind.
    pub tile: UsizeId<BObjectTile>,
}
