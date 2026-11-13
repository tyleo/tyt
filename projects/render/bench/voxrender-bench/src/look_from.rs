use crate::{Error, Result};
use ty_math::{TyPoseF64, TyQuaternionExt, TyQuaternionF64, TyVector3F64};
use voxrender::{RenderScene, RenderView};

/// Moves the first view of `scene` to look from `eye` at `look_at`, both
/// in the scene's meters, with +Y up. The view keeps its projection.
/// Errors unless the scene has a view, or if the two points coincide or
/// one lies straight above the other.
pub fn look_from(scene: &mut RenderScene, eye: [f64; 3], look_at: [f64; 3]) -> Result<()> {
    let (view_id, view) = scene.iter_views().next().ok_or(Error::NoView)?;
    let projection = view.projection;
    let position = TyVector3F64::from_array(eye);

    let rotation = TyQuaternionF64::from_look_direction(
        TyVector3F64::from_array(look_at) - position,
        TyVector3F64::Y,
    )
    .ok_or(Error::Look { eye, look_at })?;

    scene
        .set_view(
            view_id,
            RenderView {
                pose: TyPoseF64::new(position, rotation),
                projection,
            },
        )
        .map_err(Error::Render)
}
