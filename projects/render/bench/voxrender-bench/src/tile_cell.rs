use crate::TILE;
use ty_math::TyVector3U32;

/// The cell of `index` in a tile, where cell `(x, y, z)` has index
/// `(y * 16 + x) * 16 + z`.
pub fn tile_cell(index: usize) -> TyVector3U32 {
    let index = index as u32;

    TyVector3U32::new(index / TILE % TILE, index / (TILE * TILE), index % TILE)
}
