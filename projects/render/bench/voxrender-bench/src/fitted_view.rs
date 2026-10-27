use ty_math::{TyPoseF64, TyQuaternionExt, TyQuaternionF64, TyVector3Ext, TyVector3F64};
use voxrender::{RenderProjection, RenderScene, RenderView, fit_scale};

/// An orthographic view of every placement in `scene`, fitted to their
/// bounds, from `azimuth` and `elevation` degrees.
pub fn fitted_view(scene: &RenderScene, azimuth: f64, elevation: f64) -> RenderView {
    let placement_ids: Vec<_> = scene.iter_placements().map(|(id, _)| id).collect();
    let bounds = scene
        .subject_bounds(&placement_ids)
        .expect("the placements are the scene's")
        .expect("the scene has live cells");

    let direction =
        TyVector3F64::from_azimuth_elevation(azimuth.to_radians(), elevation.to_radians());
    let scale = fit_scale(&bounds);

    RenderView {
        pose: TyPoseF64::new(
            // The scale is the fitted sphere's diameter, so the view stands
            // outside the sphere.
            bounds.center + direction * scale,
            TyQuaternionF64::from_look_direction(-direction, TyVector3F64::Y)
                .expect("a fitted view does not look straight up or down"),
        ),
        projection: RenderProjection::Orthographic { scale },
    }
}
