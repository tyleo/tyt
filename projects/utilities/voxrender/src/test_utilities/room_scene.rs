use crate::{
    RenderLight, RenderMaterial, RenderPlacement, RenderScene, RenderShadow,
    test_utilities::{orbit_view, solid_object},
};
use branded_id::U32Id;
use ty_math::{TyLinSrgbF64, TyTransformF64, TyVector3F64, TyVector3U32};

/// A floor with two walls meeting in a corner and a pillar on the floor,
/// lit by one point light near the corner with `shadow` and a range that
/// ends inside the room, so the falloff and the pillar's shadow both show.
pub fn room_scene(shadow: RenderShadow) -> RenderScene {
    let mut scene = RenderScene::default();

    let material_id = scene
        .retain_material(RenderMaterial {
            base_color: TyLinSrgbF64::new(0.75, 0.7, 0.6),
            metallic: 0.0,
            roughness: 0.9,
            ..RenderMaterial::default()
        })
        .unwrap();

    let object_id = U32Id::from_u32(0);

    scene
        .retain_object(
            object_id,
            solid_object(
                "room",
                TyVector3U32::new(8, 6, 8),
                material_id,
                |position| {
                    position.y == 0
                        || position.x == 0
                        || position.z == 0
                        || (position.x == 5 && position.z == 5 && position.y < 3)
                },
            ),
        )
        .unwrap();

    scene
        .retain_placement(RenderPlacement {
            object_id,
            transform: TyTransformF64::IDENTITY,
        })
        .unwrap();

    scene
        .retain_light(RenderLight::Point {
            position: TyVector3F64::new(2.5, 4.0, 2.5),
            color: TyLinSrgbF64::new(1.0, 0.9, 0.7),
            strength: 40.0,
            range: Some(7.0),
            shadow,
        })
        .unwrap();

    scene
        .retain_light(RenderLight::Hemisphere {
            sky: TyLinSrgbF64::new(0.1, 0.1, 0.15),
            ground: TyLinSrgbF64::new(0.05, 0.05, 0.05),
            strength: 1.0,
        })
        .unwrap();

    scene
        .retain_view(orbit_view(
            TyVector3F64::new(4.0, 2.0, 4.0),
            45.0,
            35.0,
            22.0,
        ))
        .unwrap();

    scene
}
