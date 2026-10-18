use crate::{Error, PixelHit, Result, corner_across, shadow_targets, view_grid_rays};
use branded_id::U32Id;
use voxrender::{BRenderView, RenderScene};
use voxrender_cpu::{RenderRayWalk, shadow_rays};

/// The reference's first hit through each pixel of `view_id` over a `size`
/// square image, row by row.
pub fn reference_hits(
    scene: &RenderScene,
    view_id: U32Id<BRenderView>,
    size: u32,
) -> Result<Vec<Option<PixelHit>>> {
    let grid_rays = view_grid_rays(scene, view_id, size)?;
    let placements: Vec<_> = scene.iter_placements().map(|(id, _)| id).collect();
    let targets = shadow_targets(scene);

    let mut hits = Vec::new();

    for y in 0..size {
        for x in 0..size {
            let Some(hit) =
                RenderRayWalk::new(scene, grid_rays.iter().map(|rays| rays.ray(x, y))).next()
            else {
                hits.push(None);

                continue;
            };

            let placement = placements
                .iter()
                .position(|&id| id == hit.placement_id)
                .expect("a hit lands on one of the scene's placements");

            let mut lit = Vec::new();

            for target in &targets {
                let mut bits = 0;

                for corner in 0..4 {
                    let rays = shadow_rays(scene, &hit, corner_across(corner), target)
                        .map_err(Error::Render)?;
                    let mut walk = RenderRayWalk::new(scene, rays);

                    if walk.starts_inside().is_empty() && walk.next().is_none() {
                        bits |= 1 << corner;
                    }
                }

                lit.push(bits);
            }

            hits.push(Some(PixelHit {
                placement: u32::try_from(placement).expect("placements number under 2^32"),
                voxel: hit.voxel_id.to_u32(),
                face: hit.face.d as u32 * 2 + u32::from(hit.face.sign > 0),
                lit,
            }));
        }
    }

    Ok(hits)
}
