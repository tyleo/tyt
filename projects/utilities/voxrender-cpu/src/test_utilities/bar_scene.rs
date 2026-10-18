use branded_id::U32Id;
use ty_math::{TyLinSrgbF64, TyTransformF64, TyVector3F64, TyVector3U32};
use voxrender::{
    RenderLight, RenderMaterial, RenderObject, RenderPlacement, RenderRay, RenderScene,
};

/// A bar of one voxel per material along +X under a white hemisphere
/// light, and the ray that enters it from -X through the center of its
/// first voxel. Equal materials share one entry.
pub fn bar_scene(materials: &[RenderMaterial]) -> (RenderScene, RenderRay) {
    let mut scene = RenderScene::default();
    let mut object = RenderObject::new(
        "bar".to_owned(),
        TyVector3U32::new(materials.len() as u32, 1, 1),
    )
    .unwrap();
    let mut ids = Vec::new();

    for (x, material) in materials.iter().enumerate() {
        let material_id = match ids.iter().find(|(known, _)| known == material) {
            Some((_, material_id)) => *material_id,

            None => {
                let material_id = scene.retain_material(*material).unwrap();
                ids.push((*material, material_id));
                material_id
            }
        };

        let voxel_id = object.voxel_id(TyVector3U32::new(x as u32, 0, 0)).unwrap();
        object
            .set_voxel_material(voxel_id, Some(material_id))
            .unwrap();
    }

    let object_id = U32Id::from_u32(0);
    scene.retain_object(object_id, object).unwrap();
    scene
        .retain_placement(RenderPlacement {
            object_id,
            transform: TyTransformF64::IDENTITY,
        })
        .unwrap();
    scene
        .retain_light(RenderLight::Hemisphere {
            sky: TyLinSrgbF64::new(1.0, 1.0, 1.0),
            ground: TyLinSrgbF64::new(1.0, 1.0, 1.0),
            strength: 1.0,
        })
        .unwrap();

    let ray = RenderRay {
        origin: TyVector3F64::new(-1.0, 0.5, 0.5),
        direction: TyVector3F64::X,
    };

    (scene, ray)
}
