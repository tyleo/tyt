use crate::{
    BASE_COLOR_FACTOR, ColorChannel, EMISSIVE_FACTOR, EMISSIVE_STRENGTH, Error, MaterialBake,
    MaterialChannel, Result, UsedMaterials, default_scalar,
};
use ty_math::{TyLinearRgbaColorF64, TySrgbaColor};
use voxcore::{VoxMain, VoxValuePool};

/// Bakes `bake` over every material in `used` into an RGBA8 pixel buffer of
/// `width` x `height` texels, one texel per material placed row-major from the
/// top-left. Trailing texels past the material count stay transparent black. The
/// buffer's layout matches [`atlas_dimensions`](crate::atlas_dimensions) over
/// `used.len()`, so the UVs sampling it read each material's texel.
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
        MaterialBake::RgbaColor => Ok(color_bytes(material_attribute(
            state,
            used,
            index,
            BASE_COLOR_FACTOR,
        ))),

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
            let value = material_attribute(state, used, index, key);

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

/// What the material at `index` draws for attribute `key`: the value pool the
/// read layer's binding names and the value-index into it, or `None` when `index`
/// is out of range or no binding carries `key`.
fn material_attribute<'a>(
    state: &'a VoxMain,
    used: &UsedMaterials,
    index: usize,
    key: &str,
) -> Option<(&'a VoxValuePool, u32)> {
    let material = used.material(index)?;
    let palette = used.palette();
    let binding = state.palette(palette)?.binding_by_attribute(key)?;
    state.material_value(palette, material, binding)
}

/// A color attribute's RGBA bytes, defaulting to opaque white (the base-color
/// spec default) when the value is absent or its pool is not a color kind.
fn color_bytes(value: Option<(&VoxValuePool, u32)>) -> [u8; 4] {
    color_bytes_or(value, [255, 255, 255, 255])
}

/// A color attribute's RGBA bytes, decoded by the bound pool's kind, or
/// `default` when the value is absent or its pool is not a color kind. An sRGB
/// pool holds sRGB-encoded components in `[0, 1]`; a linear pool holds
/// scene-linear components re-encoded to sRGB here. A three-component color takes
/// opaque alpha.
fn color_bytes_or(value: Option<(&VoxValuePool, u32)>, default: [u8; 4]) -> [u8; 4] {
    let Some((pool, index)) = value else {
        return default;
    };
    let index = index as usize;

    match pool {
        VoxValuePool::Srgb { values } => values
            .get(index)
            .map(|&[r, g, b]| srgb_bytes([r, g, b, 1.0]))
            .unwrap_or(default),

        VoxValuePool::Srgba { values } => values
            .get(index)
            .map(|&color| srgb_bytes(color))
            .unwrap_or(default),

        VoxValuePool::LinearRgb { values } => values
            .get(index)
            .map(|&[r, g, b]| {
                TyLinearRgbaColorF64::new(r, g, b, 1.0)
                    .to_srgba()
                    .to_array()
            })
            .unwrap_or(default),

        VoxValuePool::LinearRgba { values } => values
            .get(index)
            .map(|&[r, g, b, a]| TyLinearRgbaColorF64::new(r, g, b, a).to_srgba().to_array())
            .unwrap_or(default),
        _ => default,
    }
}

/// Maps sRGB-encoded float components in `[0, 1]` to bytes.
fn srgb_bytes(color: [f64; 4]) -> [u8; 4] {
    color.map(|component| (component.clamp(0.0, 1.0) * 255.0).round() as u8)
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

/// A scalar attribute's value from a `float` or `int` pool, defaulting to its
/// spec default (or `0` for a key with no standard default) when absent or not a
/// numeric pool.
fn scalar_value(value: Option<(&VoxValuePool, u32)>, key: &str) -> f64 {
    let fallback = || default_scalar(key).unwrap_or(0.0);
    match value {
        Some((VoxValuePool::Float { values, .. }, index)) => {
            values.get(index as usize).copied().unwrap_or_else(fallback)
        }
        Some((VoxValuePool::Int { values, .. }, index)) => values
            .get(index as usize)
            .map(|&value| value as f64)
            .unwrap_or_else(fallback),
        _ => fallback(),
    }
}

/// The emissive color for the material at `index`: the `emissiveFactor` color
/// scaled by the `emissiveStrength` in linear light, re-encoded to opaque sRGB.
/// glTF's emissive slot is an RGB color, so the strength scales the emissive
/// color rather than writing a bare channel; the multiply is done in linear light
/// through the shared `ty_math` color types, and the alpha is dropped. An absent
/// `emissiveFactor` is black and an absent `emissiveStrength` is `0`, so a
/// material with no emissive attributes bakes to black.
fn emissive_color_bytes(state: &VoxMain, used: &UsedMaterials, index: usize) -> [u8; 4] {
    let color = color_bytes_or(
        material_attribute(state, used, index, EMISSIVE_FACTOR),
        [0, 0, 0, 255],
    );

    let strength = scalar_value(
        material_attribute(state, used, index, EMISSIVE_STRENGTH),
        EMISSIVE_STRENGTH,
    )
    .max(0.0);

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
        BASE_COLOR_FACTOR, ColorChannel, EMISSIVE_FACTOR, EMISSIVE_STRENGTH, METALLIC_FACTOR,
        MaterialBake, MaterialChannel, OCCLUSION_STRENGTH, ROUGHNESS_FACTOR, atlas_dimensions,
        bake_atlas_pixels, resolve_used_materials,
    };
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{BVoxLayer, BVoxObject, VoxBound, VoxMain, VoxObject, VoxPalette, VoxValuePool};

    /// A `key`-only scalar packing channel.
    fn scalar(key: &str, invert: bool) -> MaterialChannel {
        MaterialChannel::Attribute {
            key: key.to_owned(),
            component: None,
            invert,
        }
    }

    /// A single-layer document: one palette binding `baseColorFactor`,
    /// `metallicFactor`, and `roughnessFactor` over three pools, with a
    /// three-voxel object whose voxels sample three materials in raster order:
    /// (red, shiny, smooth), (red, matte, rough), (blue, matte, rough).
    fn single_layer_state() -> (VoxMain, U32Id<BVoxObject>, U32Id<BVoxLayer>) {
        let mut state = VoxMain::default();

        let base = state.add_value_pool(VoxValuePool::Srgba {
            values: vec![[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]],
        });
        let metallic = state.add_value_pool(VoxValuePool::Float {
            min: VoxBound::Number(0.0),
            max: VoxBound::Number(1.0),
            values: vec![1.0, 0.0],
        });
        let roughness = state.add_value_pool(VoxValuePool::Float {
            min: VoxBound::Number(0.0),
            max: VoxBound::Number(1.0),
            values: vec![0.0, 1.0],
        });

        let mut palette = VoxPalette::default();
        palette.add_binding(BASE_COLOR_FACTOR.to_owned(), base);
        palette.add_binding(METALLIC_FACTOR.to_owned(), metallic);
        palette.add_binding(ROUGHNESS_FACTOR.to_owned(), roughness);
        // Value-indices: base 0 = red, 1 = blue; metallic 0 = 1.0, 1 = 0.0;
        // roughness 0 = 0.0, 1 = 1.0.
        let red_shiny = palette.add_material(vec![0, 0, 0]).unwrap();
        let red_matte = palette.add_material(vec![0, 1, 1]).unwrap();
        let blue_matte = palette.add_material(vec![1, 1, 1]).unwrap();
        let palette_id = state.add_palette(palette);

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(3, 1, 1)).unwrap();
        let layer = object.add_layer(palette_id, red_shiny);

        for (x, material) in [(0, red_shiny), (1, red_matte), (2, blue_matte)] {
            let voxel = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel, &[material]).unwrap();
        }

        let object_id = state.add_object(object);

        (state, object_id, layer)
    }

    #[test]
    fn albedo_reads_the_base_color() {
        let (state, object_id, layer) = single_layer_state();
        let object = state.object(object_id).unwrap();
        let used = resolve_used_materials(object, layer).unwrap();
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
    fn a_scalar_packing_reads_the_metallic_factor() {
        let (state, object_id, layer) = single_layer_state();
        let used = resolve_used_materials(state.object(object_id).unwrap(), layer).unwrap();
        let (width, height) = atlas_dimensions(used.len());

        let metallic = MaterialBake::Packing(vec![scalar(METALLIC_FACTOR, false)]);
        let pixels = bake_atlas_pixels(&state, &used, &metallic, width, height).unwrap();
        // Only material 0 is metallic (1.0); the rest are matte (0.0).
        assert_eq!(pixels[0], 255);
        assert_eq!(pixels[4], 0);
        assert_eq!(pixels[8], 0);
    }

    #[test]
    fn inversion_turns_roughness_into_smoothness() {
        let (state, object_id, layer) = single_layer_state();
        let used = resolve_used_materials(state.object(object_id).unwrap(), layer).unwrap();
        let (width, height) = atlas_dimensions(used.len());

        let smoothness = MaterialBake::Packing(vec![scalar(ROUGHNESS_FACTOR, true)]);
        let pixels = bake_atlas_pixels(&state, &used, &smoothness, width, height).unwrap();
        // Material 0 is roughness 0.0 -> smoothness 1.0; material 1 is roughness
        // 1.0 -> smoothness 0.0.
        assert_eq!(pixels[0], 255);
        assert_eq!(pixels[4], 0);
    }

    #[test]
    fn a_missing_attribute_falls_back_to_its_spec_default() {
        let (state, object_id, layer) = single_layer_state();
        let used = resolve_used_materials(state.object(object_id).unwrap(), layer).unwrap();
        let (width, height) = atlas_dimensions(used.len());

        // The palette carries no `occlusionStrength`, whose spec default is 1.
        let occlusion = MaterialBake::Packing(vec![scalar(OCCLUSION_STRENGTH, false)]);
        let pixels = bake_atlas_pixels(&state, &used, &occlusion, width, height).unwrap();
        assert_eq!(pixels[0], 255);
        assert_eq!(pixels[4], 255);
        assert_eq!(pixels[8], 255);
    }

    #[test]
    fn a_color_component_reads_one_channel() {
        let (state, object_id, layer) = single_layer_state();
        let used = resolve_used_materials(state.object(object_id).unwrap(), layer).unwrap();
        let (width, height) = atlas_dimensions(used.len());

        let red = MaterialBake::Packing(vec![MaterialChannel::Attribute {
            key: BASE_COLOR_FACTOR.to_owned(),
            component: Some(ColorChannel::R),
            invert: false,
        }]);
        let pixels = bake_atlas_pixels(&state, &used, &red, width, height).unwrap();
        assert_eq!(pixels[0], 255); // material 0 is red
        assert_eq!(pixels[8], 0); // material 2 is blue
    }

    #[test]
    fn computed_occlusion_is_rejected_under_the_palette_atlas() {
        let (state, object_id, layer) = single_layer_state();
        let used = resolve_used_materials(state.object(object_id).unwrap(), layer).unwrap();
        let (width, height) = atlas_dimensions(used.len());

        let bake = MaterialBake::Packing(vec![MaterialChannel::ComputedOcclusion]);
        assert!(bake_atlas_pixels(&state, &used, &bake, width, height).is_err());
    }

    #[test]
    fn emissive_color_scales_the_factor_by_strength() {
        let mut state = VoxMain::default();

        // emissiveFactor is an sRGB color (no alpha); emissiveStrength scales it.
        let factor = state.add_value_pool(VoxValuePool::Srgb {
            values: vec![[0.0, 0.0, 1.0], [1.0, 1.0, 1.0]],
        });
        let strength = state.add_value_pool(VoxValuePool::Float {
            min: VoxBound::Number(0.0),
            max: VoxBound::None,
            values: vec![1.0, 0.5],
        });

        let mut palette = VoxPalette::default();
        palette.add_binding(EMISSIVE_FACTOR.to_owned(), factor);
        palette.add_binding(EMISSIVE_STRENGTH.to_owned(), strength);
        let full_blue = palette.add_material(vec![0, 0]).unwrap();
        let dim_white = palette.add_material(vec![1, 1]).unwrap();
        let palette_id = state.add_palette(palette);

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        let layer = object.add_layer(palette_id, full_blue);
        for (x, material) in [(0, full_blue), (1, dim_white)] {
            let voxel = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel, &[material]).unwrap();
        }
        let object_id = state.add_object(object);

        let used = resolve_used_materials(state.object(object_id).unwrap(), layer).unwrap();
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
