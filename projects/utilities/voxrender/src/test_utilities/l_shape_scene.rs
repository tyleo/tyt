use crate::{
    RenderLight, RenderMaterial, RenderPlacement, RenderScene, RenderShadow,
    test_utilities::{orbit_view, solid_object},
};
use branded_id::U32Id;
use ty_math::{
    TyLinSrgbF64, TyQuaternionExt, TyQuaternionF64, TyTransformF64, TyVector3Ext, TyVector3F64,
    TyVector3U32,
};

/// A floor with a wall along its far edge under a sun from behind the
/// wall with `shadow`, so the wall shadows the floor and the crease
/// between them shows the corner occlusion.
pub fn l_shape_scene(shadow: RenderShadow) -> RenderScene {
    let mut scene = RenderScene::default();

    let material_id = scene
        .retain_material(RenderMaterial {
            base_color: TyLinSrgbF64::new(0.7, 0.7, 0.7),
            metallic: 0.0,
            roughness: 0.8,
            ..RenderMaterial::default()
        })
        .unwrap();

    let object_id = U32Id::from_u32(0);

    scene
        .retain_object(
            object_id,
            solid_object("l", TyVector3U32::new(6, 4, 6), material_id, |position| {
                position.y == 0 || position.z == 0
            }),
        )
        .unwrap();

    scene
        .retain_placement(RenderPlacement {
            object_id,
            transform: TyTransformF64::IDENTITY,
        })
        .unwrap();

    let from = TyVector3F64::from_azimuth_elevation(160f64.to_radians(), 35f64.to_radians());

    scene
        .retain_light(RenderLight::Directional {
            rotation: TyQuaternionF64::from_look_direction(-from, TyVector3F64::Y).unwrap(),
            color: TyLinSrgbF64::new(1.0, 0.95, 0.85),
            strength: 3.0,
            shadow,
        })
        .unwrap();

    scene
        .retain_light(RenderLight::Hemisphere {
            sky: TyLinSrgbF64::new(0.3, 0.35, 0.45),
            ground: TyLinSrgbF64::new(0.1, 0.1, 0.1),
            strength: 1.0,
        })
        .unwrap();

    scene
        .retain_view(orbit_view(
            TyVector3F64::new(3.0, 1.5, 3.0),
            20.0,
            30.0,
            17.0,
        ))
        .unwrap();

    scene
}
