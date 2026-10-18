use crate::{RenderScene, fixtures::cells_scene};
use ty_math::TyTransformF64;

/// Diagonal staircases along x = z in a 24 by 10 by 24 grid, common in
/// voxel art.
pub fn stairs_scene() -> RenderScene {
    let mut cells = Vec::new();

    for k in 0..24 {
        for y in 1..4 {
            cells.push(([k, y, (k + 7) % 24], 0));
            cells.push(([(k + 13) % 24, y, k], 0));
        }
    }

    cells_scene([24, 10, 24], &cells, &[TyTransformF64::IDENTITY])
}
