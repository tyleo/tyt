use crate::{RenderScene, fixtures::cells_scene};
use ty_math::TyTransformF64;

/// A 24 by 10 by 24 grid of one material, live where a hash of `seed` and
/// the cell falls under `density`, from layer `floor` up.
pub fn random_scene(seed: u64, density: f64, floor: u32) -> RenderScene {
    let mut cells = Vec::new();

    for x in 0..24 {
        for y in floor..10 {
            for z in 0..24 {
                let mut hash = seed
                    .wrapping_add(u64::from(x * 240 + y * 24 + z))
                    .wrapping_mul(0x9e37_79b9_7f4a_7c15);
                hash = (hash ^ (hash >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                hash = (hash ^ (hash >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                hash ^= hash >> 31;

                if (hash as f64) < density * u64::MAX as f64 {
                    cells.push(([x, y, z], 0));
                }
            }
        }
    }

    cells_scene([24, 10, 24], &cells, &[TyTransformF64::IDENTITY])
}
