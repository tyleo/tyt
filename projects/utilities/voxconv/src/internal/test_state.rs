use branded_id::U32Id;
use ty_math::TyVector3U32;
use voxcore::{VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValuePool};

/// A state carrying `ext`, with one `baseColor` palette and one tight 1x1x1
/// object sampling its one material. The object sits under a root node so
/// every format's writer emits it.
pub fn test_state<T>(ext: T) -> VoxMain<T> {
    let mut state = VoxMain::default();

    let colors_value_pool_id =
        state.retain_value_pool(VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap());

    let mut palette = VoxPalette::default();

    palette
        .retain_property(
            "baseColor".to_owned(),
            colors_value_pool_id,
            U32Id::from_u32(0),
        )
        .unwrap();

    let material_id = palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();

    let palette_id = state.retain_palette(palette).unwrap();

    let mut object = VoxObject::new("body".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();

    object.retain_layer(palette_id, material_id);

    let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();

    object.retain_voxel(voxel_id, &[material_id]).unwrap();

    let object_id = state.retain_object(object).unwrap();

    let node_id = state
        .retain_hierarchy_node(VoxHierarchyNode {
            child_object_ids: vec![object_id],
            ..Default::default()
        })
        .unwrap();

    state.set_root_hierarchy_node_ids(vec![node_id]).unwrap();

    state.map_ext(|()| ext)
}
