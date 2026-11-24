use crate::{CellBox, CellGrid, TILE, raster_index};
use ty_math::TyVector3U32;

/// The box around the live cells of each non-empty tile of `grid`, in
/// raster order.
pub fn live_boxes(grid: &CellGrid) -> Vec<CellBox> {
    let bounds = grid.bounds;
    let counts = (bounds + TILE - 1) / TILE;

    // A tile without a live cell keeps a low corner past its high one.
    let mut boxes =
        vec![(TyVector3U32::MAX, TyVector3U32::ZERO); (counts.x * counts.y * counts.z) as usize];

    for x in 0..bounds.x {
        for y in 0..bounds.y {
            for z in 0..bounds.z {
                let cell = TyVector3U32::new(x, y, z);

                if grid.cells[raster_index(cell, bounds)] != 0 {
                    let (low, high) = &mut boxes[raster_index(cell / TILE, counts)];

                    *low = low.min(cell);
                    *high = high.max(cell + 1);
                }
            }
        }
    }

    boxes.retain(|(low, high)| low.x < high.x);

    boxes
}

#[cfg(test)]
mod tests {
    use crate::{CellGrid, live_boxes, painted_object};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;

    #[test]
    fn a_box_hugs_its_tiles_live_cells() {
        let material = U32Id::from_u32(0);

        // A slab in the first tile, nothing in the second, and one cell in
        // the third.
        let object = painted_object("o", TyVector3U32::new(48, 16, 16), |p| {
            (p.x < 16 && p.y < 2 || p == TyVector3U32::new(40, 3, 5)).then_some(material)
        });

        assert_eq!(
            live_boxes(&CellGrid::new(&object, &|id| id)),
            [
                (TyVector3U32::ZERO, TyVector3U32::new(16, 2, 16)),
                (TyVector3U32::new(40, 3, 5), TyVector3U32::new(41, 4, 6)),
            ]
        );
    }
}
