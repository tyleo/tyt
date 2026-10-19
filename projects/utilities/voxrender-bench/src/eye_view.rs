use ty_math::{TyPoseF64, TyQuaternionExt, TyQuaternionF64, TyVector3F64};
use voxrender::{RenderProjection, RenderView};

/// A 90-degree perspective view from `eye` toward `target`, as a headset's
/// eye sees.
pub fn eye_view(eye: TyVector3F64, target: TyVector3F64) -> RenderView {
    RenderView {
        pose: TyPoseF64::new(
            eye,
            TyQuaternionF64::from_look_direction(target - eye, TyVector3F64::Y)
                .expect("an eye view does not look straight up or down"),
        ),
        projection: RenderProjection::Perspective {
            fov: 90f64.to_radians(),
        },
    }
}
