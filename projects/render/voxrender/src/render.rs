use crate::{BRenderView, RenderBloom, RenderImage, RenderOcclusion, RenderScene, Result};
use branded_id::U32Id;

/// Draws a view of a scene into an image, the capability a renderer
/// provides. `voxrender-cpu`'s reference renderer implements the contract
/// literally.
pub trait Render {
    /// Draws the view `view_id` of `scene` into a `width` by `height` image
    /// under every light of the scene. `occlusion` decides whether the
    /// corner occlusion darkens the hemisphere light, and `bloom` adds its
    /// halo. Errors if the view is not one of the scene's, a side is zero, a
    /// bloom value is out of range, or a ray or light lies out of a grid's
    /// [range](crate::GRID_RANGE_BITS).
    fn render(
        &self,
        scene: &RenderScene,
        view_id: U32Id<BRenderView>,
        occlusion: RenderOcclusion,
        bloom: RenderBloom,
        width: u32,
        height: u32,
    ) -> Result<RenderImage>;
}
