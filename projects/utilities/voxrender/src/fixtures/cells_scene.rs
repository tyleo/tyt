use crate::{RenderMaterial, RenderObject, RenderPlacement, RenderScene};
use branded_id::U32Id;
use ty_math::{TyTransformF64, TyVector3U32};

/// A scene with one object of `bounds` under each of `transforms`. Each of
/// `cells` holds the material at its index, one distinct material per
/// index.
pub fn cells_scene(
    bounds: [u32; 3],
    cells: &[([u32; 3], usize)],
    transforms: &[TyTransformF64],
) -> RenderScene {
    let mut scene = RenderScene::default();

    let material_count = cells.iter().map(|&(_, index)| index + 1).max().unwrap_or(0);
    let material_ids: Vec<_> = (0..material_count)
        .map(|index| {
            scene
                .retain_material(RenderMaterial {
                    roughness: 1.0 / (index + 1) as f64,
                    ..RenderMaterial::default()
                })
                .unwrap()
        })
        .collect();

    let mut object = RenderObject::new("o".to_owned(), TyVector3U32::from_array(bounds)).unwrap();

    for &(position, index) in cells {
        let voxel_id = object.voxel_id(TyVector3U32::from_array(position)).unwrap();

        object
            .set_voxel_material(voxel_id, Some(material_ids[index]))
            .unwrap();
    }

    let object_id = U32Id::from_u32(0);
    scene.retain_object(object_id, object).unwrap();

    for &transform in transforms {
        scene
            .retain_placement(RenderPlacement {
                object_id,
                transform,
            })
            .unwrap();
    }

    scene
}
