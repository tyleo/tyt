use crate::{RenderProjection, RenderView};
use ty_math::{TyPoseF64, TyQuaternionExt, TyQuaternionF64, TyVector3Ext, TyVector3F64};

/// A perspective view on a sphere of `distance` about `center`, facing it,
/// at `azimuth` degrees from +Z toward +X and `elevation` degrees toward
/// +Y.
pub fn orbit_view(center: TyVector3F64, azimuth: f64, elevation: f64, distance: f64) -> RenderView {
    let direction =
        TyVector3F64::from_azimuth_elevation(azimuth.to_radians(), elevation.to_radians());

    RenderView {
        pose: TyPoseF64::new(
            center + direction * distance,
            TyQuaternionF64::from_look_direction(-direction, TyVector3F64::Y).unwrap(),
        ),
        projection: RenderProjection::Perspective {
            fov: 35f64.to_radians(),
        },
    }
}
