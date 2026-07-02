use crate::{
    MaterialBake, Result, atlas_dimensions, bake_atlas_pixels, encode_rgba8_png,
    resolve_used_materials,
};
use voxcore::{VoxMain, VoxObject};

/// A baked palette material atlas: the shared texel dimensions and one PNG image
/// per requested [`MaterialBake`], in request order.
pub struct MaterialAtlas {
    /// Atlas width in texels.
    pub width: u32,

    /// Atlas height in texels.
    pub height: u32,

    /// One PNG per bake, in the order the bakes were given.
    pub images: Vec<Vec<u8>>,
}

/// Bakes `bakes` over `object`'s used palette materials into a [`MaterialAtlas`]:
/// one texel per distinct merged material the object samples, laid out
/// near-square, each image a PNG. `state` resolves the object's referenced
/// palettes. This is the geometry-free material bake the palette atlas shares
/// with the textured mesh writer, and the surface a bake-only material command
/// builds on.
pub fn object_to_material_atlas(
    state: &VoxMain,
    object: &VoxObject,
    bakes: &[MaterialBake],
) -> Result<MaterialAtlas> {
    let used = resolve_used_materials(object);

    let (width, height) = atlas_dimensions(used.len());

    let mut images = Vec::with_capacity(bakes.len());

    for bake in bakes {
        let pixels = bake_atlas_pixels(state, &used, bake, width, height)?;

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
    use crate::{MaterialBake, MaterialChannel, object_to_material_atlas};
    use ty_math::TyVector3U32;
    use voxcore::{VoxMain, VoxObject, VoxPalette, VoxValue};

    /// The 8-byte PNG signature.
    const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    #[test]
    fn bakes_one_png_per_requested_map() {
        let mut state = VoxMain::default();

        let mut palette = VoxPalette::default();

        palette.add_attribute("rgba".to_owned());

        let red = palette
            .add_cell(vec![VoxValue::Text("#FF0000FF".to_owned())])
            .unwrap();

        let palette_id = state.add_palette(palette);

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.add_palette_ref(palette_id, red);

        let voxel = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel, &[red]).unwrap();

        let object_id = state.add_object(object);

        let bakes = [
            MaterialBake::RgbaColor,
            MaterialBake::Packing(vec![MaterialChannel::Attribute {
                key: "roughness".to_owned(),
                component: None,
                invert: false,
            }]),
        ];

        let atlas =
            object_to_material_atlas(&state, state.object(object_id).unwrap(), &bakes).unwrap();

        // One texel (one used material), one PNG per bake, each a real PNG.
        assert_eq!((atlas.width, atlas.height), (1, 1));
        assert_eq!(atlas.images.len(), 2);

        for image in &atlas.images {
            assert!(image.starts_with(&PNG_MAGIC));
        }
    }
}
