use crate::{RenderPlacement, RenderScene};
use ty_math::TyVector3U32;

/// The bounds of the grid `placement` draws in `scene`.
pub fn placement_bounds(scene: &RenderScene, placement: &RenderPlacement) -> TyVector3U32 {
    scene
        .object(placement.object_id)
        .expect("a placement's object is one of the scene's")
        .bounds()
}
