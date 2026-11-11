use ty_math::TyVector3U32;
use voxrender::RenderObject;
use voxrender_wgpu::dense_cells;

/// The box around the live cells of each non-empty brick of `object`, as
/// its low cell and the corner past its high cell, in raster order. A brick
/// spans `brick` cells a side.
pub fn live_boxes(object: &RenderObject, brick: u32) -> Vec<(TyVector3U32, TyVector3U32)> {
    let bounds = object.bounds();
    let counts = (bounds + brick - 1) / brick;
    let cells = dense_cells(object);

    // A brick without a live cell keeps a low corner past its high one.
    let mut boxes =
        vec![(TyVector3U32::MAX, TyVector3U32::ZERO); (counts.x * counts.y * counts.z) as usize];

    for x in 0..bounds.x {
        for y in 0..bounds.y {
            for z in 0..bounds.z {
                if cells[((x * bounds.y + y) * bounds.z + z) as usize] != 0 {
                    let cell = TyVector3U32::new(x, y, z);
                    let at = cell / brick;
                    let (low, high) =
                        &mut boxes[((at.x * counts.y + at.y) * counts.z + at.z) as usize];

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
    use crate::{live_boxes, painted_object};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;

    #[test]
    fn a_box_hugs_its_bricks_live_cells() {
        let material = U32Id::from_u32(0);

        // A slab in the first brick, nothing in the second, and one cell in
        // the third.
        let object = painted_object("o", TyVector3U32::new(48, 16, 16), |p| {
            (p.x < 16 && p.y < 2 || p == TyVector3U32::new(40, 3, 5)).then_some(material)
        });

        assert_eq!(
            live_boxes(&object, 16),
            [
                (TyVector3U32::ZERO, TyVector3U32::new(16, 2, 16)),
                (TyVector3U32::new(40, 3, 5), TyVector3U32::new(41, 4, 6)),
            ]
        );
    }
}
