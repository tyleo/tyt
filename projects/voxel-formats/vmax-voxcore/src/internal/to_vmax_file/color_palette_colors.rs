use crate::{Error, PALETTE_COLORS, Result};
use voxcore::{VoxValuePool, color::value_pool_color, material::BASE_COLOR};

/// `value_pool` decoded to exactly [`PALETTE_COLORS`] 0-based RGBA entries,
/// padded with transparent entries or truncated to that count. Colors past
/// the budget are dropped; a voxel that would reference one is rejected by
/// [`voxel_indices`](crate::voxel_indices).
///
/// Errors when the value pool holds no color because a transparent stand-in
/// would write a model Voxel Max renders as empty.
pub(crate) fn color_palette_colors(value_pool: &VoxValuePool) -> Result<Vec<[u8; 4]>> {
    let mut cells: Vec<[u8; 4]> = Vec::new();
    for (value_id, _) in value_pool.iter_values().take(PALETTE_COLORS) {
        let color = value_pool_color(value_pool, value_id).ok_or_else(|| {
            Error::invalid(format!(
                "`{BASE_COLOR}` draws from a value pool holding no color"
            ))
        })?;
        cells.push(color);
    }
    cells.resize(PALETTE_COLORS, [0, 0, 0, 0]);
    Ok(cells)
}
