use crate::{MVoxVoxMain, Result, to_mvox_file};
use mvox_codec::to_mvox_file_bytes;

/// Writes a [`MVoxVoxMain`] to the bytes of a MagicaVoxel `.vox` file, the
/// bytes form of [`to_mvox_file`] and the inverse of
/// [`from_mvox_bytes`](crate::codec::from_mvox_bytes).
pub fn to_mvox_bytes(main: &MVoxVoxMain) -> Result<Vec<u8>> {
    let file = to_mvox_file(main)?;

    Ok(to_mvox_file_bytes(&file))
}

#[cfg(test)]
mod tests {
    use crate::{
        MVoxVoxMain,
        codec::{from_mvox_bytes, to_mvox_bytes},
        to_mvox_vox_main,
    };
    use branded_id::U32Id;
    use ty_math::{TySrgbaU8, TyVector3U32};
    use voxcore::{
        BVoxMaterial, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValuePool,
        color::lin_srgba_f64_from_srgba_u8, material::BASE_COLOR,
    };

    /// A main placing one red voxel at the origin.
    fn red_voxel_main() -> VoxMain<()> {
        let mut main = VoxMain::default();
        let color = lin_srgba_f64_from_srgba_u8(TySrgbaU8::from([0xFF, 0, 0, 0xFF]));
        let value_pool_id =
            main.retain_value_pool(VoxValuePool::vec_4_float(vec![color.into()]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        let material_id = palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();
        let palette_id = main.retain_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::splat(1)).unwrap();
        object.retain_layer(palette_id, U32Id::<BVoxMaterial>::from_u32(0));
        let voxel_id = object.voxel_id(TyVector3U32::splat(0)).unwrap();
        object.retain_voxel(voxel_id, &[material_id]).unwrap();
        let object_id = main.retain_object(object).unwrap();

        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                child_object_ids: vec![object_id],
                ..Default::default()
            })
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![node_id]).unwrap();
        main.validate().unwrap();
        main
    }

    /// A main written to bytes reads back with the same geometry, so the
    /// bytes functions compose the file conversion and the codec the right
    /// way round.
    #[test]
    fn round_trips_through_vox_bytes() {
        let bytes = to_mvox_bytes(&to_mvox_vox_main(red_voxel_main()).unwrap()).unwrap();
        assert!(bytes.starts_with(b"VOX "));

        let reloaded = from_mvox_bytes(&bytes).unwrap();
        assert_eq!(reloaded.object_count(), 1);
        let object = reloaded.object(U32Id::from_u32(0)).unwrap();
        assert_eq!(object.live_count(), 1);
    }

    /// A default main writes through its ext and loads back.
    #[test]
    fn round_trips_the_ext_through_bytes() {
        let bytes = to_mvox_bytes(&MVoxVoxMain::default()).unwrap();
        assert!(bytes.starts_with(b"VOX "));

        let reloaded = from_mvox_bytes(&bytes).unwrap();
        assert_eq!(reloaded.object_count(), 0);
    }
}
