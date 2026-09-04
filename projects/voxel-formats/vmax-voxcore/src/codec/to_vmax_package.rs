use crate::{Result, VMaxColorFormat, VMaxVoxMain, to_vmax_file};
use vmax_codec::{
    CompressLzfse, EncodePng, EncodeVMaxPlist, EncodeVMaxSceneJson, Result as CodecResult,
    to_vmax_package as write_vmax_package,
};

/// Writes a [`VMaxVoxMain`] to a `.vmax` package through `dependencies`,
/// the package form of [`to_vmax_file`] and the inverse of
/// [`from_vmax_package`](crate::codec::from_vmax_package). For control over
/// the scene camera, build the file with
/// [`VmaxFileBuilder`](crate::VmaxFileBuilder) and write it through
/// [`vmax_codec::to_vmax_package`].
///
/// # Arguments
/// * `vmax_color_format` - where each palette's colors are stored.
/// * `write` - receives each file's package-relative name and bytes and
///   performs the actual write, creating any subdirectory a `QuickLook/` name
///   implies.
pub fn to_vmax_package<D, W>(
    dependencies: &D,
    state: &VMaxVoxMain,
    vmax_color_format: VMaxColorFormat,
    write: W,
) -> Result<()>
where
    D: CompressLzfse + EncodeVMaxPlist + EncodePng + EncodeVMaxSceneJson,
    W: FnMut(&str, &[u8]) -> CodecResult<()>,
{
    let file = to_vmax_file(state, vmax_color_format)?;
    Ok(write_vmax_package(dependencies, &file, write)?)
}

#[cfg(test)]
mod tests {
    use crate::{
        VMaxColorFormat, VMaxVoxMain,
        codec::{from_vmax_package, to_vmax_package},
    };
    use branded_id::U32Id;
    use std::collections::HashMap;
    use ty_math::{TySrgbaU8, TyVector3U32};
    use vmax_codec::DependenciesImpl;
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette,
        VoxValuePool, color::lin_srgba_f64_from_srgba_u8, material::BASE_COLOR,
    };

    /// A state placing one red voxel at the origin.
    fn red_voxel_state() -> VMaxVoxMain {
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
        state
            .set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(
                node_id.to_u32(),
            )])
            .unwrap();
        state.validate().unwrap();
        state
    }

    /// A state written to an in-memory package reads back with the same
    /// geometry, so the package functions compose the file conversion and
    /// the byte codec the right way round.
    #[test]
    fn round_trips_through_an_in_memory_package() {
        let mut package: HashMap<String, Vec<u8>> = HashMap::new();
        to_vmax_package(
            &DependenciesImpl,
            &red_voxel_state(),
            VMaxColorFormat::Png,
            |name, bytes| {
                package.insert(name.to_owned(), bytes.to_vec());
                Ok(())
            },
        )
        .unwrap();
        assert!(package.contains_key("scene.json"));
        assert!(package.contains_key("contents.vmaxb"));
        assert!(package.contains_key("palette1.png"));

        let reloaded = from_vmax_package(
            &DependenciesImpl,
            || Ok(package.keys().cloned().collect()),
            |name| Ok(package.get(name).cloned()),
        )
        .unwrap();
        assert_eq!(reloaded.object_count(), 1);
        let object = reloaded.object(U32Id::from_u32(0)).unwrap();
        assert_eq!(object.live_count(), 1);
        assert!(reloaded.ext().is_some(), "a loaded package carries its ext");
    }
}
