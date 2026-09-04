use crate::{GoxelVoxMain, Result, to_goxl_file};
use goxl_codec::{EncodePng, to_gox_file_bytes};

/// Writes a [`GoxelVoxMain`] to the bytes of a Goxel `.gox` file through
/// `dependencies`, the bytes form of [`to_goxl_file`] and the inverse of
/// [`from_goxl_bytes`](crate::codec::from_goxl_bytes).
pub fn to_goxl_bytes<D: EncodePng>(dependencies: &D, state: &GoxelVoxMain) -> Result<Vec<u8>> {
    let file = to_goxl_file(state)?;
    Ok(to_gox_file_bytes(dependencies, &file))
}

#[cfg(test)]
mod tests {
    use crate::codec::{from_goxl_bytes, to_goxl_bytes};
    use branded_id::U32Id;
    use goxl_codec::DependenciesImpl;
    use ty_math::{TySrgbaU8, TyVector3U32};
    use voxcore::{
        BVoxMaterial, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValuePool,
        color::lin_srgba_f64_from_srgba_u8, material::BASE_COLOR,
    };

    /// A state with no ext placing one red voxel at the origin.
    fn red_voxel_state() -> VoxMain<Option<crate::GoxelExt>> {
        let mut state = VoxMain::default();
        let color = lin_srgba_f64_from_srgba_u8(TySrgbaU8::from([0xFF, 0, 0, 0xFF]));
        let value_pool_id =
            state.retain_value_pool(VoxValuePool::vec_4_float(vec![color.into()]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        let material_id = palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();
        let palette_id = state.retain_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::splat(1)).unwrap();
        object.retain_layer(palette_id, U32Id::<BVoxMaterial>::from_u32(0));
        let voxel_id = object.voxel_id(TyVector3U32::splat(0)).unwrap();
        object.retain_voxel(voxel_id, &[material_id]).unwrap();
        let object_id = state.retain_object(object).unwrap();

        let node_id = state
            .retain_hierarchy_node(VoxHierarchyNode {
                child_object_ids: vec![object_id],
                ..Default::default()
            })
            .unwrap();
        state.set_root_hierarchy_node_ids(vec![node_id]).unwrap();
        state.validate().unwrap();
        state
    }

    /// A state written to bytes reads back with the same geometry and gains
    /// the ext, so the bytes functions compose the file conversion and the
    /// codec the right way round.
    #[test]
    fn round_trips_through_gox_bytes() {
        let bytes = to_goxl_bytes(&DependenciesImpl, &red_voxel_state()).unwrap();
        assert!(bytes.starts_with(b"GOX "));

        let reloaded = from_goxl_bytes(&DependenciesImpl, &bytes).unwrap();
        assert_eq!(reloaded.object_count(), 1);
        let object = reloaded.object(U32Id::from_u32(0)).unwrap();
        assert_eq!(object.live_count(), 1);
        assert!(reloaded.ext().is_some(), "a loaded file carries its ext");
    }
}
