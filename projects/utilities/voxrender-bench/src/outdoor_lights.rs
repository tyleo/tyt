use ty_math::{TyLinSrgbF64, TyQuaternionExt, TyQuaternionF64, TyVector3Ext, TyVector3F64};
use voxrender::{RenderLight, RenderScene, RenderShadow};

/// Adds a sky and a sun from `azimuth` and `elevation` degrees to `scene`.
/// The sun casts per-corner shadows.
pub fn outdoor_lights(scene: &mut RenderScene, azimuth: f64, elevation: f64) {
    let from = TyVector3F64::from_azimuth_elevation(azimuth.to_radians(), elevation.to_radians());

    scene
        .retain_light(RenderLight::Directional {
            rotation: TyQuaternionF64::from_look_direction(-from, TyVector3F64::Y)
                .expect("the sun is above the horizon"),
            color: TyLinSrgbF64::new(1.0, 0.95, 0.85),
            strength: 3.0,
            shadow: RenderShadow::PerCorner,
        })
        .expect("a sun is a valid light");

    scene
        .retain_light(RenderLight::Hemisphere {
            sky: TyLinSrgbF64::new(0.4, 0.45, 0.55),
            ground: TyLinSrgbF64::new(0.15, 0.13, 0.1),
            strength: 1.0,
        })
        .expect("a sky is a valid light");
}
