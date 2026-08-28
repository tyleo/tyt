use crate::{
    AtlasShape, MaterialBake, Result, atlas_dimensions, bake_atlas_pixels,
    check_gltf_property_ranges, encode_rgba8_png, resolve_used_materials,
};
use voxcore::{VoxMain, VoxObject};

/// A baked material atlas: the shared texel dimensions and one PNG image per
/// requested [`MaterialBake`], in request order.
pub struct MaterialAtlas {
    /// Atlas width in texels.
    pub width: u32,

    /// Atlas height in texels.
    pub height: u32,

    /// One PNG per bake, in the order the bakes were given.
    pub images: Vec<Vec<u8>>,
}

/// Bakes `bakes` over the flattened materials `object` uses into a
/// [`MaterialAtlas`], the object's layers merged per property name by the
/// format's override rule: one texel per distinct material tuple the object
/// samples across its winning layers, laid out per `shape`, each image a PNG.
/// `state` resolves the object's referenced palettes. This is the
/// geometry-free material bake the palette atlas shares with the textured
/// mesh writer, and the surface a bake-only material command builds on.
/// Errors if a layer references a palette `state` does not hold, if a
/// vocabulary property carries a value outside its glTF range, or if `shape`
/// is too small to hold the materials.
pub fn object_to_material_atlas<T>(
    state: &VoxMain<T>,
    object: &VoxObject,
    bakes: &[MaterialBake],
    shape: AtlasShape,
) -> Result<MaterialAtlas> {
    // The texels this bakes reach a glTF material, so the boundary runs the
    // same vocabulary check the mesh export does rather than growing its own.
    check_gltf_property_ranges(state)?;

    let used = resolve_used_materials(state, object)?;

    let (width, height) = atlas_dimensions(used.len(), shape)?;

    let mut images = Vec::with_capacity(bakes.len());

    for bake in bakes {
        let pixels = bake_atlas_pixels(&used, bake, width, height)?;

        images.push(encode_rgba8_png(width, height, &pixels)?);
    }

    Ok(MaterialAtlas {
        width,
        height,
        images,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        AtlasShape, BASE_COLOR, MaterialBake, MaterialChannel, ROUGHNESS, object_to_material_atlas,
    };
    use branded_id::U32Id;
    use std::io::Cursor;
    use ty_math::TyVector3U32;
    use voxcore::{VoxMain, VoxObject, VoxPalette, VoxValuePool};

    /// The 8-byte PNG signature.
    const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    /// Decodes a PNG into its RGBA8 bytes.
    fn decode_rgba8_png(bytes: &[u8]) -> Vec<u8> {
        let mut reader = png::Decoder::new(Cursor::new(bytes)).read_info().unwrap();
        let mut pixels = vec![0u8; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();
        pixels.truncate(info.buffer_size());
        pixels
    }

    #[test]
    fn bakes_one_png_per_requested_map() {
        let mut state: VoxMain = VoxMain::default();

        let base_value_pool_id =
            state.retain_value_pool(VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .retain_property(
                BASE_COLOR.to_owned(),
                base_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        let red_id = palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();
        let palette_id = state.retain_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.retain_layer(palette_id, red_id);
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel_id, &[red_id]).unwrap();

        let object_id = state.retain_object(object).unwrap();

        let bakes = [
            MaterialBake::RgbaColor,
            MaterialBake::Packing(vec![MaterialChannel::Property {
                key: ROUGHNESS.to_owned(),
                component: None,
                invert: false,
            }]),
        ];

        let atlas = object_to_material_atlas(
            &state,
            state.object(object_id).unwrap(),
            &bakes,
            AtlasShape::Fit,
        )
        .unwrap();

        // One texel (one used material), one PNG per bake, each a real PNG.
        assert_eq!((atlas.width, atlas.height), (1, 1));
        assert_eq!(atlas.images.len(), 2);

        for image in &atlas.images {
            assert!(image.starts_with(&PNG_MAGIC));
        }
    }

    #[test]
    fn an_out_of_range_vocabulary_value_errors_before_the_bake() {
        // The atlas is a glTF boundary, so it runs the vocabulary check its
        // sibling mesh export runs rather than baking a lying texel.
        let mut state: VoxMain = VoxMain::default();
        let value_pool_id = state.retain_value_pool(VoxValuePool::float(vec![2.5]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .retain_property(ROUGHNESS.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        let material_id = palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();
        let palette_id = state.retain_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.retain_layer(palette_id, material_id);
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel_id, &[material_id]).unwrap();
        let object_id = state.retain_object(object).unwrap();

        let error = object_to_material_atlas(
            &state,
            state.object(object_id).unwrap(),
            &[MaterialBake::RgbaColor],
            AtlasShape::Fit,
        )
        .err()
        .expect("an out-of-range roughness errors");
        assert!(error.to_string().contains(ROUGHNESS), "{error}");
    }

    #[test]
    fn a_layerless_object_bakes_one_texel_of_spec_defaults() {
        let mut state: VoxMain = VoxMain::default();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel_id, &[]).unwrap();
        let object_id = state.retain_object(object).unwrap();

        let atlas = object_to_material_atlas(
            &state,
            state.object(object_id).unwrap(),
            &[MaterialBake::RgbaColor],
            AtlasShape::Fit,
        )
        .unwrap();

        // The empty identity key gives one texel, the base color's spec
        // default of opaque white.
        assert_eq!((atlas.width, atlas.height), (1, 1));
        let pixels = decode_rgba8_png(&atlas.images[0]);
        assert_eq!(pixels, [255, 255, 255, 255]);
    }
}
