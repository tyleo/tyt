use crate::{
    ColorChannel, Error, MaterialBake, MaterialChannel, Result, UsedMaterials, default_scalar,
};
use ty_math::{TyLinearRgbaColorF64, TySrgbaColor};
use voxcore::{VoxMain, VoxValue};

/// Bakes `bake` over every material in `used` into an RGBA8 pixel buffer of
/// `width` x `height` texels, one texel per material placed row-major from the
/// top-left. Trailing texels past the material count stay transparent black. The
/// buffer's layout matches [`atlas_dimensions`](crate::atlas_dimensions), so the
/// UVs sampling it read each material's texel.
pub(crate) fn bake_atlas_pixels(
    state: &VoxMain,
    used: &UsedMaterials,
    bake: &MaterialBake,
    width: u32,
    height: u32,
) -> Result<Vec<u8>> {
    let width = width as usize;

    let mut pixels = vec![0u8; width * height as usize * 4];

    for index in 0..used.len() {
        let rgba = bake_texel(state, used, bake, index)?;

        let offset = index * 4;

        pixels[offset..offset + 4].copy_from_slice(&rgba);
    }

    Ok(pixels)
}

/// The RGBA bytes of the material at `index` under `bake`.
fn bake_texel(
    state: &VoxMain,
    used: &UsedMaterials,
    bake: &MaterialBake,
    index: usize,
) -> Result<[u8; 4]> {
    match bake {
        MaterialBake::RgbaColor => Ok(color_bytes(merged_attribute(state, used, index, "rgba"))),

        MaterialBake::EmissiveColor => Ok(emissive_color_bytes(state, used, index)),

        MaterialBake::Packing(channels) => {
            // A packing fills R, G, B from its channels; an unnamed channel and
            // an absent alpha stay `0` and opaque respectively.
            let mut rgba = [0u8, 0u8, 0u8, 255u8];

            for (channel_index, channel) in channels.iter().enumerate() {
                rgba[channel_index] = channel_byte(state, used, index, channel)?;
            }

            Ok(rgba)
        }
    }
}

/// The `0..255` byte one channel contributes for the material at `index`.
fn channel_byte(
    state: &VoxMain,
    used: &UsedMaterials,
    index: usize,
    channel: &MaterialChannel,
) -> Result<u8> {
    match channel {
        MaterialChannel::Zero => Ok(0),

        MaterialChannel::One => Ok(255),

        MaterialChannel::ComputedOcclusion => Err(Error::invalid(
            "computed-occlusion requires an unwrap atlas, which is not yet supported",
        )),

        MaterialChannel::Attribute {
            key,
            component,
            invert,
        } => {
            let value = merged_attribute(state, used, index, key);

            // Read the source as a `0..1` fraction, invert if asked, then scale
            // to a byte, so a scalar and a color component inject the same way.
            let fraction = match component {
                Some(component) => component_byte(color_bytes(value), *component) as f64 / 255.0,
                None => scalar_value(value, key).clamp(0.0, 1.0),
            };

            let fraction = if *invert { 1.0 - fraction } else { fraction };

            Ok((fraction.clamp(0.0, 1.0) * 255.0).round() as u8)
        }
    }
}

/// The merged value of attribute `key` for the material at `index`: the value
/// from the last palette reference that sets it (later references win), or
/// `None` when no reference carries it.
fn merged_attribute<'a>(
    state: &'a VoxMain,
    used: &UsedMaterials,
    index: usize,
    key: &str,
) -> Option<&'a VoxValue> {
    let mut merged = None;

    for (&(_, palette_id), &cell) in used.references().iter().zip(used.cells(index)) {
        let Some(palette) = state.palette(palette_id) else {
            continue;
        };

        let Some(attribute) = palette
            .iter_attributes()
            .find(|(_, name)| *name == key)
            .map(|(id, _)| id)
        else {
            continue;
        };

        match palette.cell_value(cell, attribute) {
            Some(VoxValue::Null) | None => {}
            Some(value) => merged = Some(value),
        }
    }

    merged
}

/// A color attribute's RGBA bytes, defaulting to opaque white (the `rgba` spec
/// default) when the value is absent or not a `#RRGGBBAA` string.
fn color_bytes(value: Option<&VoxValue>) -> [u8; 4] {
    let white = [255, 255, 255, 255];

    match value {
        Some(VoxValue::Text(text)) => TySrgbaColor::from_hex(text)
            .map(|color| color.to_array())
            .unwrap_or(white),
        _ => white,
    }
}

/// One color component as a byte.
fn component_byte(rgba: [u8; 4], component: ColorChannel) -> u8 {
    match component {
        ColorChannel::R => rgba[0],
        ColorChannel::G => rgba[1],
        ColorChannel::B => rgba[2],
        ColorChannel::A => rgba[3],
    }
}

/// A scalar attribute's value, defaulting to its spec default (or `0` for a key
/// with no standard default) when absent or not a number.
fn scalar_value(value: Option<&VoxValue>, key: &str) -> f64 {
    match value {
        Some(VoxValue::Number(number)) => *number,
        _ => default_scalar(key).unwrap_or(0.0),
    }
}

/// The emissive color for the material at `index`: the `rgba` base color scaled
/// by the `emissive` strength in linear light, re-encoded to opaque sRGB. glTF's
/// emissive slot is an RGB color, so the voxel-native strength tints the base
/// color rather than writing a bare channel; the multiply is done in linear
/// light through the shared `ty_math` color types, and the alpha is dropped.
fn emissive_color_bytes(state: &VoxMain, used: &UsedMaterials, index: usize) -> [u8; 4] {
    let color = color_bytes(merged_attribute(state, used, index, "rgba"));

    let strength =
        scalar_value(merged_attribute(state, used, index, "emissive"), "emissive").clamp(0.0, 1.0);

    let linear = TySrgbaColor::from_array(color).to_linear_rgba();

    let scaled = TyLinearRgbaColorF64::new(
        linear.r * strength,
        linear.g * strength,
        linear.b * strength,
        1.0,
    );

    let srgba = scaled.to_srgba();

    [srgba.r, srgba.g, srgba.b, 255]
}

#[cfg(test)]
mod tests {
    use crate::{
        ColorChannel, MaterialBake, MaterialChannel, atlas_dimensions, bake_atlas_pixels,
        resolve_used_materials,
    };
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{BVoxObject, VoxMain, VoxObject, VoxPalette, VoxValue};

    /// A `key`-only scalar packing channel.
    fn scalar(key: &str, invert: bool) -> MaterialChannel {
        MaterialChannel::Attribute {
            key: key.to_owned(),
            component: None,
            invert,
        }
    }

    /// A two-layer document: a base `rgba` palette (red, blue) and a `metallic`
    /// / `roughness` finish palette, with a three-voxel object sampling three
    /// distinct combinations in raster order: (red, cell 0), (red, cell 1),
    /// (blue, cell 1).
    fn two_layer_state() -> (VoxMain, U32Id<BVoxObject>) {
        let mut state = VoxMain::default();

        let mut base = VoxPalette::default();
        base.add_attribute("rgba".to_owned());
        let red = base
            .add_cell(vec![VoxValue::Text("#FF0000FF".to_owned())])
            .unwrap();
        let blue = base
            .add_cell(vec![VoxValue::Text("#0000FFFF".to_owned())])
            .unwrap();
        let base_id = state.add_palette(base);

        let mut finish = VoxPalette::default();
        finish.add_attribute("metallic".to_owned());
        finish.add_attribute("roughness".to_owned());
        let shiny = finish
            .add_cell(vec![VoxValue::Number(1.0), VoxValue::Number(0.0)])
            .unwrap();
        let matte = finish
            .add_cell(vec![VoxValue::Number(0.0), VoxValue::Number(1.0)])
            .unwrap();
        let finish_id = state.add_palette(finish);

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(3, 1, 1)).unwrap();
        object.add_palette_ref(base_id, red);
        object.add_palette_ref(finish_id, shiny);

        for (x, base_cell, finish_cell) in [(0, red, shiny), (1, red, matte), (2, blue, matte)] {
            let voxel = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object
                .retain_voxel(voxel, &[base_cell, finish_cell])
                .unwrap();
        }

        let object_id = state.add_object(object);

        (state, object_id)
    }

    #[test]
    fn albedo_reads_the_base_color_across_the_merge() {
        let (state, object_id) = two_layer_state();
        let object = state.object(object_id).unwrap();
        let used = resolve_used_materials(object);
        assert_eq!(used.len(), 3);

        let (width, height) = atlas_dimensions(used.len());
        let pixels =
            bake_atlas_pixels(&state, &used, &MaterialBake::RgbaColor, width, height).unwrap();

        // Materials 0 and 1 both take the red base; material 2 is blue. The
        // fourth texel is padding.
        assert_eq!(&pixels[0..4], &[255, 0, 0, 255]);
        assert_eq!(&pixels[4..8], &[255, 0, 0, 255]);
        assert_eq!(&pixels[8..12], &[0, 0, 255, 255]);
        assert_eq!(&pixels[12..16], &[0, 0, 0, 0]);
    }

    #[test]
    fn a_scalar_packing_reads_the_finish_layer() {
        let (state, object_id) = two_layer_state();
        let used = resolve_used_materials(state.object(object_id).unwrap());
        let (width, height) = atlas_dimensions(used.len());

        let metallic = MaterialBake::Packing(vec![scalar("metallic", false)]);
        let pixels = bake_atlas_pixels(&state, &used, &metallic, width, height).unwrap();
        // Only material 0 samples the shiny (metallic 1.0) finish cell.
        assert_eq!(pixels[0], 255);
        assert_eq!(pixels[4], 0);
        assert_eq!(pixels[8], 0);
    }

    #[test]
    fn inversion_turns_roughness_into_smoothness() {
        let (state, object_id) = two_layer_state();
        let used = resolve_used_materials(state.object(object_id).unwrap());
        let (width, height) = atlas_dimensions(used.len());

        let smoothness = MaterialBake::Packing(vec![scalar("roughness", true)]);
        let pixels = bake_atlas_pixels(&state, &used, &smoothness, width, height).unwrap();
        // Material 0 is roughness 0.0 -> smoothness 1.0; material 1 is the
        // matte cell, roughness 1.0 -> smoothness 0.0.
        assert_eq!(pixels[0], 255);
        assert_eq!(pixels[4], 0);
    }

    #[test]
    fn a_missing_attribute_falls_back_to_its_spec_default() {
        let (state, object_id) = two_layer_state();
        let used = resolve_used_materials(state.object(object_id).unwrap());
        let (width, height) = atlas_dimensions(used.len());

        // Neither palette carries `occlusion`, whose spec default is 1.
        let occlusion = MaterialBake::Packing(vec![scalar("occlusion", false)]);
        let pixels = bake_atlas_pixels(&state, &used, &occlusion, width, height).unwrap();
        assert_eq!(pixels[0], 255);
        assert_eq!(pixels[4], 255);
        assert_eq!(pixels[8], 255);
    }

    #[test]
    fn a_color_component_reads_one_channel() {
        let (state, object_id) = two_layer_state();
        let used = resolve_used_materials(state.object(object_id).unwrap());
        let (width, height) = atlas_dimensions(used.len());

        let red = MaterialBake::Packing(vec![MaterialChannel::Attribute {
            key: "rgba".to_owned(),
            component: Some(ColorChannel::R),
            invert: false,
        }]);
        let pixels = bake_atlas_pixels(&state, &used, &red, width, height).unwrap();
        assert_eq!(pixels[0], 255); // material 0 is red
        assert_eq!(pixels[8], 0); // material 2 is blue
    }

    #[test]
    fn computed_occlusion_is_rejected_under_the_palette_atlas() {
        let (state, object_id) = two_layer_state();
        let used = resolve_used_materials(state.object(object_id).unwrap());
        let (width, height) = atlas_dimensions(used.len());

        let bake = MaterialBake::Packing(vec![MaterialChannel::ComputedOcclusion]);
        assert!(bake_atlas_pixels(&state, &used, &bake, width, height).is_err());
    }

    #[test]
    fn a_reference_free_object_bakes_one_default_material() {
        let mut state = VoxMain::default();
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        let voxel = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel, &[]).unwrap();
        let object_id = state.add_object(object);

        let used = resolve_used_materials(state.object(object_id).unwrap());
        assert_eq!(used.len(), 1);

        let (width, height) = atlas_dimensions(used.len());
        let pixels =
            bake_atlas_pixels(&state, &used, &MaterialBake::RgbaColor, width, height).unwrap();
        // The `rgba` default is opaque white.
        assert_eq!(&pixels[0..4], &[255, 255, 255, 255]);
    }

    #[test]
    fn emissive_color_tints_the_base_color_by_strength() {
        let mut state = VoxMain::default();

        let mut palette = VoxPalette::default();
        palette.add_attribute("rgba".to_owned());
        palette.add_attribute("emissive".to_owned());
        let blue = palette
            .add_cell(vec![
                VoxValue::Text("#0000FFFF".to_owned()),
                VoxValue::Number(1.0),
            ])
            .unwrap();
        let dim_white = palette
            .add_cell(vec![
                VoxValue::Text("#FFFFFFFF".to_owned()),
                VoxValue::Number(0.5),
            ])
            .unwrap();
        let palette_id = state.add_palette(palette);

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        object.add_palette_ref(palette_id, blue);
        for (x, cell) in [(0, blue), (1, dim_white)] {
            let voxel = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel, &[cell]).unwrap();
        }
        let object_id = state.add_object(object);

        let used = resolve_used_materials(state.object(object_id).unwrap());
        let (width, height) = atlas_dimensions(used.len());
        let pixels =
            bake_atlas_pixels(&state, &used, &MaterialBake::EmissiveColor, width, height).unwrap();

        // Full-strength blue glows blue: a strength of 1 is the identity round
        // trip through linear light, and the alpha is dropped to opaque.
        assert_eq!(&pixels[0..4], &[0, 0, 255, 255]);

        // Half-strength white glows a neutral mid gray (R == G == B), scaled
        // down from full white but not to black, opaque.
        let (r, g, b, a) = (pixels[4], pixels[5], pixels[6], pixels[7]);
        assert!(r == g && g == b);
        assert!(r > 0 && r < 255);
        assert_eq!(a, 255);
    }
}
