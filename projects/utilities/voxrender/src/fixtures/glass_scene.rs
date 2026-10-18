use crate::{
    RenderLight, RenderMaterial, RenderObject, RenderPlacement, RenderScene, RenderShadow,
    fixtures::orbit_view,
};
use branded_id::U32Id;
use ty_math::{
    TyLinSrgbF64, TyLinSrgbaF64, TyQuaternionExt, TyQuaternionF64, TyTransformF64, TyVector3Ext,
    TyVector3F64, TyVector3U32,
};

/// The glass asset: a grey floor with a blue wall along its left edge, a
/// clear block three voxels thick against the wall, a red pane two voxels
/// thick standing on the floor and rising past the wall into the empty
/// background, and a half-alpha green voxel in front of the block. A sun
/// from behind the pane with `shadow` throws the pane's red shadow onto the
/// floor. Seen from the front-right-top.
pub fn glass_scene(shadow: RenderShadow) -> RenderScene {
    let mut scene = RenderScene::default();

    let matte = |base_color: TyLinSrgbaF64| RenderMaterial {
        base_color,
        metallic: 0.0,
        roughness: 0.8,
        ..RenderMaterial::default()
    };
    let glass = |base_color: TyLinSrgbaF64| RenderMaterial {
        roughness: 0.2,
        transmission: 1.0,
        ..matte(base_color)
    };

    let floor_id = scene
        .retain_material(matte(TyLinSrgbaF64::new(0.6, 0.6, 0.6, 1.0)))
        .unwrap();
    let wall_id = scene
        .retain_material(matte(TyLinSrgbaF64::new(0.2, 0.35, 0.85, 1.0)))
        .unwrap();
    let clear_id = scene
        .retain_material(glass(TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0)))
        .unwrap();
    let red_id = scene
        .retain_material(glass(TyLinSrgbaF64::new(1.0, 0.1, 0.1, 1.0)))
        .unwrap();
    let half_id = scene
        .retain_material(matte(TyLinSrgbaF64::new(0.2, 0.8, 0.3, 0.5)))
        .unwrap();

    let mut object = RenderObject::new("glass".to_owned(), TyVector3U32::new(8, 5, 7)).unwrap();

    let boxes = [
        ([0, 0, 0], [8, 1, 7], floor_id),
        ([0, 1, 0], [1, 5, 7], wall_id),
        ([1, 1, 1], [4, 3, 4], clear_id),
        ([5, 1, 3], [7, 5, 5], red_id),
        ([2, 1, 5], [3, 2, 6], half_id),
    ];

    for (min, max, material_id) in boxes {
        for x in min[0]..max[0] {
            for y in min[1]..max[1] {
                for z in min[2]..max[2] {
                    let voxel_id = object.voxel_id(TyVector3U32::new(x, y, z)).unwrap();

                    object
                        .set_voxel_material(voxel_id, Some(material_id))
                        .unwrap();
                }
            }
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

    let from = TyVector3F64::from_azimuth_elevation(150f64.to_radians(), 50f64.to_radians());

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
            TyVector3F64::new(4.0, 2.0, 3.5),
            40.0,
            28.0,
            18.0,
        ))
        .unwrap();

    scene
}
