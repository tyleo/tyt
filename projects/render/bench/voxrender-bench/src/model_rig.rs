use crate::{Error, Result};
use ty_math::{
    TyBoundsF64, TyLinSrgbF64, TyPoseF64, TyQuaternionExt, TyQuaternionF64, TySrgbU8, TyVector3Ext,
    TyVector3F64,
};
use voxrender::{
    RenderBloom, RenderLight, RenderProjection, RenderShadow, RenderView, fit_distance,
};

/// The degrees of azimuth and elevation vxl's hero view looks from.
const HERO: [f64; 2] = [45.0, 30.0];

/// The degrees of the hero view's vertical field of view.
const FOV: f64 = 35.0;

/// The degrees of azimuth and elevation the studio rig's key light shines
/// from, in its view's frame.
const KEY: [f64; 2] = [-30.0, 30.0];

/// What frames a model whose live voxels span `bounds`: vxl's hero view of
/// it for a square image, vxl's studio rig as that view places the rig, and
/// the bloom vxl's review renders take.
pub fn model_rig(bounds: &TyBoundsF64) -> Result<(RenderView, [RenderLight; 2], RenderBloom)> {
    let from = |[azimuth, elevation]: [f64; 2]| {
        TyVector3F64::from_azimuth_elevation(azimuth.to_radians(), elevation.to_radians())
    };

    let facing = |from: TyVector3F64| {
        TyQuaternionF64::from_look_direction(-from, TyVector3F64::Y)
            .expect("the direction leaves the vertical")
    };

    let fov = FOV.to_radians();
    let hero = from(HERO);
    let rotation = facing(hero);

    let view = RenderView {
        pose: TyPoseF64::new(
            bounds.center + hero * fit_distance(bounds, fov, 1, 1),
            rotation,
        ),
        projection: RenderProjection::from_fov(fov, 1, 1).map_err(Error::Render)?,
    };

    let gray = |level: u8| {
        TySrgbU8::new(level, level, level)
            .into_format::<f64>()
            .into_linear()
    };

    let lights = [
        RenderLight::Directional {
            rotation: facing(rotation * from(KEY)),
            color: TyLinSrgbF64::new(1.0, 1.0, 1.0),
            strength: 1.95,
            shadow: RenderShadow::PerCorner,
        },
        RenderLight::Hemisphere {
            sky: gray(255),
            ground: gray(192),
            strength: 0.78,
        },
    ];

    let bloom = RenderBloom {
        strength: 1.0,
        ..RenderBloom::default()
    };

    Ok((view, lights, bloom))
}
