use crate::{
    RenderLight, RenderMaterial, RenderPlacement, RenderScene, RenderShadow,
    test_utilities::{orbit_view, solid_object},
};
use branded_id::U32Id;
use ty_math::{
    TyLinSrgbF64, TyQuaternionExt, TyQuaternionF64, TyTransformF64, TyVector3Ext, TyVector3F64,
    TyVector3U32,
};

/// One rough dielectric cube lit by a sun from the front-left-top with
/// `shadow` and by a hemisphere light, seen from the front-right-top.
pub fn cube_scene(shadow: RenderShadow) -> RenderScene {
    let mut scene = RenderScene::default();

    let material_id = scene
        .retain_material(RenderMaterial {
            base_color: TyLinSrgbF64::new(0.8, 0.5, 0.2),
            metallic: 0.0,
            roughness: 0.6,
            ..RenderMaterial::default()
        })
        .unwrap();

    let object_id = U32Id::from_u32(0);

    scene
        .retain_object(
            object_id,
            solid_object("cube", TyVector3U32::splat(2), material_id, |_| true),
        )
        .unwrap();

    scene
        .retain_placement(RenderPlacement {
            object_id,
            transform: TyTransformF64::IDENTITY,
        })
        .unwrap();

    let from = TyVector3F64::from_azimuth_elevation((-40f64).to_radians(), 45f64.to_radians());

    scene
        .retain_light(RenderLight::Directional {
            rotation: TyQuaternionF64::from_look_direction(-from, TyVector3F64::Y).unwrap(),
            color: TyLinSrgbF64::new(1.0, 1.0, 1.0),
            strength: 3.0,
            shadow,
        })
        .unwrap();

    scene
        .retain_light(RenderLight::Hemisphere {
            sky: TyLinSrgbF64::new(0.4, 0.45, 0.5),
            ground: TyLinSrgbF64::new(0.15, 0.12, 0.1),
            strength: 1.0,
        })
        .unwrap();

    scene
        .retain_view(orbit_view(TyVector3F64::splat(1.0), 35.0, 25.0, 9.0))
        .unwrap();

    scene
}
