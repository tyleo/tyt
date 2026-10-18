use crate::{RenderLight, RenderShadow};
use ty_math::{TyLinSrgbF64, TyQuaternionExt, TyQuaternionF64, TyVector3Ext, TyVector3F64};

/// The precision-sim's lights over a 24 by 10 by 24 grid, each casting
/// shadows per corner: eight suns, at 45 degrees and other angles that tie
/// often, then two point lights.
pub fn precision_lights() -> Vec<RenderLight> {
    let suns = [
        (45.0, 30.0),
        (45.0, 45.0),
        (-30.0, 30.0),
        (30.0, 30.0),
        (120.0, 10.0),
        (0.0, 90.0),
        (90.0, 0.0),
        (45.0, 35.264_389_68),
    ];

    let points = [[12.3, 15.7, 8.1], [12.5, 4.5, 12.5]];

    let white = TyLinSrgbF64::new(1.0, 1.0, 1.0);

    let suns = suns.map(|(azimuth, elevation): (f64, f64)| {
        let from =
            TyVector3F64::from_azimuth_elevation(azimuth.to_radians(), elevation.to_radians());
        let up = if elevation == 90.0 {
            -TyVector3F64::Z
        } else {
            TyVector3F64::Y
        };

        RenderLight::Directional {
            rotation: TyQuaternionF64::from_look_direction(-from, up).unwrap(),
            color: white,
            strength: 1.0,
            shadow: RenderShadow::PerCorner,
        }
    });

    let points = points.map(|position| RenderLight::Point {
        position: TyVector3F64::from_array(position),
        color: white,
        strength: 1.0,
        range: None,
        shadow: RenderShadow::PerCorner,
    });

    suns.into_iter().chain(points).collect()
}
