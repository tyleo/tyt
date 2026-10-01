use crate::{
    RenderLight, RenderMaterial, RenderObject, RenderPlacement, RenderScene, RenderShadow,
    test_utilities::orbit_view,
};
use branded_id::U32Id;
use ty_math::{
    TyLinSrgbF64, TyQuaternionExt, TyQuaternionF64, TyTransformF64, TyVector3Ext, TyVector3F64,
    TyVector3U32,
};

/// A dark plate whose top row glows orange at four times glTF's emissive
/// strength, lit as the cube is, seen from the front-right-top. The halo
/// spills above the plate's silhouette.
pub fn glow_scene(shadow: RenderShadow) -> RenderScene {
    let mut scene = RenderScene::default();

    let dark_id = scene
        .retain_material(RenderMaterial {
            base_color: TyLinSrgbF64::new(0.12, 0.12, 0.14),
            metallic: 0.0,
            roughness: 0.8,
            ..RenderMaterial::default()
        })
        .unwrap();

    let glowing_id = scene
        .retain_material(RenderMaterial {
            base_color: TyLinSrgbF64::new(0.2, 0.1, 0.05),
            metallic: 0.0,
            roughness: 0.8,
            emissive_color: TyLinSrgbF64::new(1.0, 0.55, 0.2),
            emissive_strength: 4.0,
            ..RenderMaterial::default()
        })
        .unwrap();

    let mut object = RenderObject::new("sign".to_owned(), TyVector3U32::new(6, 4, 1)).unwrap();

    for x in 0..6 {
        for y in 0..4 {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, y, 0)).unwrap();
            let material_id = if y == 3 { glowing_id } else { dark_id };

            object
                .set_voxel_material(voxel_id, Some(material_id))
                .unwrap();
        }
    }

    let object_id = U32Id::from_u32(0);

    scene.retain_object(object_id, object).unwrap();

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
        .retain_view(orbit_view(
            TyVector3F64::new(3.0, 2.0, 0.5),
            35.0,
            25.0,
            12.0,
        ))
        .unwrap();

    scene
}
