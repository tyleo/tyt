use crate::test_utilities::HookRecorder;
use branded_id::{IdRange, U32Id};
use ty_math::{TyVector3I32, TyVector3U32};
use voxcore::{VoxMain, VoxObject, VoxPalette, VoxValuePool};

/// One object `a` of `bounds` at `origin` on a palette of two materials, live
/// at each `cells` position with the material index beside it.
pub fn two_material_scene(
    bounds: TyVector3U32,
    origin: TyVector3I32,
    cells: &[(TyVector3U32, u32)],
) -> VoxMain<HookRecorder> {
    let mut main: VoxMain = VoxMain::default();

    let value_pool =
        VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]]).unwrap();

    let value_pool_id = main.retain_value_pool(value_pool);

    let mut palette = VoxPalette::default();

    palette
        .retain_property("baseColor".to_owned(), value_pool_id, U32Id::from_u32(0))
        .unwrap();

    for value_id in IdRange::from_len(2) {
        palette.retain_material(vec![value_id]).unwrap();
    }

    let palette_id = main.retain_palette(palette).unwrap();

    let mut object = VoxObject::new("a".to_owned(), bounds).unwrap();

    object.set_origin(origin);

    object.retain_layer(palette_id, U32Id::from_u32(0));

    for &(position, material) in cells {
        let voxel_id = object.voxel_id(position).unwrap();

        object
            .retain_voxel(voxel_id, &[U32Id::from_u32(material)])
            .unwrap();
    }

    main.retain_object(object).unwrap();

    main.put_ext(HookRecorder::default())
}
