use ty_math::{TyPoseF64, TyQuaternionExt, TyQuaternionF64, TyVector3F64};
use voxrender::{RenderProjection, RenderView};

/// A 90-degree perspective view for a square image from `eye` toward
/// `target`.
pub fn eye_view(eye: TyVector3F64, target: TyVector3F64) -> RenderView {
    RenderView {
        pose: TyPoseF64::new(
            eye,
            TyQuaternionF64::from_look_direction(target - eye, TyVector3F64::Y)
                .expect("an eye view does not look straight up or down"),
        ),
        projection: RenderProjection::from_fov(90f64.to_radians(), 1, 1)
            .expect("the view's field of view is within range"),
    }
}
