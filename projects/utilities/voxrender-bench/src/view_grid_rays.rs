use crate::{Error, Result};
use branded_id::U32Id;
use voxrender::{
    BRenderView, Error as RenderError, RenderGridViewRays, RenderScene, RenderViewRays,
};

/// The rays of `view_id` over a `size` square image in each placement's
/// grid, in placement order.
pub fn view_grid_rays(
    scene: &RenderScene,
    view_id: U32Id<BRenderView>,
    size: u32,
) -> Result<Vec<RenderGridViewRays>> {
    let view = scene.view(view_id).expect("the view is one of the scene's");
    let rays = RenderViewRays::new(view, size, size);

    scene
        .iter_placements()
        .map(|(placement_id, placement)| {
            let bounds = scene
                .object(placement.object_id)
                .expect("a placement's object is one of the scene's")
                .bounds();

            rays.to_grid_rays(&placement.transform, bounds)
                .ok_or(Error::Render(RenderError::GridRange { placement_id }))
        })
        .collect()
}
