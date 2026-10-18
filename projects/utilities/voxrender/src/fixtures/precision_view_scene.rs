use crate::{RenderProjection, RenderScene, RenderView, fixtures::random_scene};
use ty_math::{TyPoseF64, TyQuaternionExt, TyQuaternionF64, TyVector3Ext, TyVector3F64};

/// A sparse random grid seen through ten views, each orbit in perspective
/// and then orthographic. The orbits include axis-aligned and straight-down
/// views, whose rays tie most often, and one off-center view.
pub fn precision_view_scene() -> RenderScene {
    let mut scene = random_scene(3, 0.08, 0);

    let orbits = [
        (45.0, 30.0, [12.0, 5.0, 12.0]),
        (-30.0, 30.0, [12.0, 5.0, 12.0]),
        (0.0, 0.0, [12.0, 5.0, 12.0]),
        (0.0, 90.0, [12.0, 5.0, 12.0]),
        (0.0, 0.0, [12.3, 5.1, 12.7]),
    ];

    let projections = [
        RenderProjection::Perspective {
            fov: 30f64.to_radians(),
        },
        RenderProjection::Orthographic { scale: 16.0 },
    ];

    for (azimuth, elevation, center) in orbits {
        let back = TyVector3F64::from_azimuth_elevation(
            f64::to_radians(azimuth),
            f64::to_radians(elevation),
        );
        let up = if elevation == 90.0 {
            -TyVector3F64::Z
        } else {
            TyVector3F64::Y
        };
        let pose = TyPoseF64::new(
            TyVector3F64::from_array(center) + back * 40.0,
            TyQuaternionF64::from_look_direction(-back, up).unwrap(),
        );

        for projection in projections {
            scene.retain_view(RenderView { pose, projection }).unwrap();
        }
    }

    scene
}
