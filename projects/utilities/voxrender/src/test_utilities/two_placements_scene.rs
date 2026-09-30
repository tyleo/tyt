use crate::{
    RenderLight, RenderMaterial, RenderPlacement, RenderScene, RenderShadow,
    test_utilities::{orbit_view, solid_object},
};
use branded_id::U32Id;
use ty_math::{
    TyLinSrgbF64, TyQuaternionExt, TyQuaternionF64, TyTransformF64, TyVector3Ext, TyVector3F64,
    TyVector3U32,
};

/// One slab placed twice: flat on the ground, and above it turned 45
/// degrees about Y and scaled down, so the upper placement shadows the
/// lower under a sun from above with `shadow`.
pub fn two_placements_scene(shadow: RenderShadow) -> RenderScene {
    let mut scene = RenderScene::default();

    let material_id = scene
        .retain_material(RenderMaterial {
            base_color: TyLinSrgbF64::new(0.3, 0.6, 0.8),
            metallic: 0.0,
            roughness: 0.5,
            ..RenderMaterial::default()
        })
        .unwrap();

    let object_id = U32Id::from_u32(0);

    scene
        .retain_object(
            object_id,
            solid_object("slab", TyVector3U32::new(6, 1, 4), material_id, |_| true),
        )
        .unwrap();

    scene
        .retain_placement(RenderPlacement {
            object_id,
            transform: TyTransformF64::IDENTITY,
        })
        .unwrap();

    scene
        .retain_placement(RenderPlacement {
            object_id,
            transform: TyTransformF64::new(
                TyVector3F64::new(2.0, 3.0, 3.5),
                TyQuaternionF64::from_axis_angle(TyVector3F64::Y, 45f64.to_radians()),
                TyVector3F64::splat(0.5),
            ),
        })
        .unwrap();

    let from = TyVector3F64::from_azimuth_elevation(20f64.to_radians(), 70f64.to_radians());

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
            sky: TyLinSrgbF64::new(0.4, 0.4, 0.4),
            ground: TyLinSrgbF64::new(0.1, 0.1, 0.1),
            strength: 1.0,
        })
        .unwrap();

    scene
        .retain_view(orbit_view(
            TyVector3F64::new(3.0, 1.5, 2.0),
            30.0,
            35.0,
            16.0,
        ))
        .unwrap();

    scene
}
