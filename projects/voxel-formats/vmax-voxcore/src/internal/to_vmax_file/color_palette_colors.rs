use crate::{Error, PALETTE_COLORS, Result};
use voxcore::{VoxValuePool, color::value_pool_color, material::BASE_COLOR};

/// `value_pool` decoded to exactly [`PALETTE_COLORS`] 0-based RGBA entries,
/// padded with transparent entries to that count.
///
/// Errors when the value pool holds more colors than the budget, because the
/// table is the pool whole. Errors when it holds no color, because a
/// transparent stand-in would write a model Voxel Max renders as empty.
pub(crate) fn color_palette_colors(value_pool: &VoxValuePool) -> Result<Vec<[u8; 4]>> {
    if value_pool.len() > PALETTE_COLORS {
        return Err(Error::invalid(format!(
            "`{BASE_COLOR}` draws from a value pool of {} colors, but a Voxel Max palette holds \
             only {PALETTE_COLORS}",
            value_pool.len()
        )));
    }
    let mut cells: Vec<[u8; 4]> = Vec::new();
    for (value_id, _) in value_pool.iter_values() {
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
