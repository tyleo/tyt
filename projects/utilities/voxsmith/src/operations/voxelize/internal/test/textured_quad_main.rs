use crate::operations::voxelize::{MapSpec, full_square, pbr_quad_main};
use meshdoc::MeshMain;

/// A unit quad with a base-color texture of the given PNG over the full
/// square and the given linear `factor`.
pub(crate) fn textured_quad_main(png: &[u8], factor: [f64; 4]) -> MeshMain<()> {
    pbr_quad_main(
        full_square(),
        full_square(),
        &[MapSpec::BaseColor {
            png,
            stream: 0,
            factor,
        }],
    )
}
