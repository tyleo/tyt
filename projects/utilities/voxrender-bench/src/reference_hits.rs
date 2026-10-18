use crate::{PixelHit, Result, view_grid_rays};
use branded_id::U32Id;
use voxrender::{BRenderView, RenderRayWalk, RenderScene};

/// The reference's first hit through each pixel of `view_id` over a `size`
/// square image, row by row.
pub fn reference_hits(
    scene: &RenderScene,
    view_id: U32Id<BRenderView>,
    size: u32,
) -> Result<Vec<Option<PixelHit>>> {
    let grid_rays = view_grid_rays(scene, view_id, size)?;
    let placements: Vec<_> = scene.iter_placements().map(|(id, _)| id).collect();

    let mut hits = Vec::new();

    for y in 0..size {
        for x in 0..size {
            let hit = RenderRayWalk::new(scene, grid_rays.iter().map(|rays| rays.ray(x, y))).next();

            hits.push(hit.map(|hit| {
                let placement = placements
                    .iter()
                    .position(|&id| id == hit.placement_id)
                    .expect("a hit lands on one of the scene's placements");

                PixelHit {
                    placement: u32::try_from(placement).expect("placements number under 2^32"),
                    voxel: hit.voxel_id.to_u32(),
                    face: hit.face.d as u32 * 2 + u32::from(hit.face.sign > 0),
                }
            }));
        }
    }

    Ok(hits)
}
