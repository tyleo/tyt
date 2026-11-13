use crate::{Error, Result};
use std::path::Path;
use ty_math::{
    TyLinSrgbF64, TyPoseF64, TyQuaternionExt, TyQuaternionF64, TySrgbU8, TyVector3Ext, TyVector3F64,
};
use voxconv::{DependenciesImpl, ReadFormat, load};
use voxrender::{
    RenderBloom, RenderLight, RenderProjection, RenderScene, RenderShadow, RenderView, fit_distance,
};

/// The degrees of azimuth and elevation vxl's hero view looks from.
const HERO: [f64; 2] = [45.0, 30.0];

/// The degrees of the hero view's vertical field of view.
const FOV: f64 = 35.0;

/// The degrees of azimuth and elevation the studio rig's key light shines
/// from, in its view's frame.
const KEY: [f64; 2] = [-30.0, 30.0];

/// The scene of the Voxel Json document at `path`: every object at its
/// nodes' placements, in meters. The scene's first view is vxl's hero view
/// of the model for a square image. Its lights are vxl's studio rig as that
/// view places the rig. The bloom is the one vxl's review renders take.
/// Errors if the document does not load, holds no live voxel, or holds a
/// material the contract cannot shade.
pub fn model_scene(path: &Path) -> Result<(RenderScene, RenderBloom)> {
    let model = |error: String| Error::Model {
        path: path.to_owned(),
        error,
    };

    let main = load(&DependenciesImpl, ReadFormat::Voxj, path)
        .map_err(|error| model(error.to_string()))?;

    let object_ids: Vec<_> = main
        .iter_objects()
        .map(|(object_id, _)| object_id)
        .collect();

    let mut scene = RenderScene::from_vox_main(&main, &object_ids, 1.0).map_err(Error::Render)?;

    let placement_ids: Vec<_> = scene.iter_placements().map(|(id, _)| id).collect();

    let bounds = scene
        .subject_bounds(&placement_ids)
        .expect("the placements are the scene's")
        .ok_or_else(|| model("the document holds no live voxel".to_owned()))?;

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

    scene
        .retain_view(RenderView {
            pose: TyPoseF64::new(
                bounds.center + hero * fit_distance(&bounds, fov, 1, 1),
                rotation,
            ),
            projection: RenderProjection::from_fov(fov, 1, 1).map_err(Error::Render)?,
        })
        .map_err(Error::Render)?;

    let gray = |level: u8| {
        TySrgbU8::new(level, level, level)
            .into_format::<f64>()
            .into_linear()
    };

    scene
        .retain_light(RenderLight::Directional {
            rotation: facing(rotation * from(KEY)),
            color: TyLinSrgbF64::new(1.0, 1.0, 1.0),
            strength: 1.95,
            shadow: RenderShadow::PerCorner,
        })
        .map_err(Error::Render)?;

    scene
        .retain_light(RenderLight::Hemisphere {
            sky: gray(255),
            ground: gray(192),
            strength: 0.78,
        })
        .map_err(Error::Render)?;

    let bloom = RenderBloom {
        strength: 1.0,
        ..RenderBloom::default()
    };

    Ok((scene, bloom))
}
