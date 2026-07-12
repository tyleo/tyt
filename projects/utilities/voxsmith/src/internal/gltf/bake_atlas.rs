use crate::{
    BASE_COLOR_FACTOR, ColorChannel, EMISSIVE_FACTOR, EMISSIVE_STRENGTH, Error, MaterialBake,
    MaterialChannel, Result, UsedMaterials, default_scalar,
};
use ty_math::{TyFloatExt, TyLinSrgbaF64, TySrgbaF64, TySrgbaU8};
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

    // Only the emissive bake normalizes each texel by the mesh's greatest
    // strength.
    let max_strength = match bake {
        MaterialBake::EmissiveColor => max_emissive_strength(state, used),
        _ => 0.0,
    };

    for index in 0..used.len() {
        let rgba = bake_texel(state, used, bake, index, max_strength)?;

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
    max_strength: f64,
) -> Result<[u8; 4]> {
    match bake {
        MaterialBake::RgbaColor => Ok(color_bytes(material_attribute(
            state,
            used,
            index,
            BASE_COLOR_FACTOR,
        ))),

        MaterialBake::EmissiveColor => Ok(emissive_color_bytes(state, used, index, max_strength)),

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

            Ok(fraction.to_unorm8())
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
            .map(|&[r, g, b]| TySrgbaF64::new(r, g, b, 1.0).to_u8().to_array())
            .unwrap_or(default),

        VoxValuePool::Srgba { values } => values
            .get(index)
            .map(|&[r, g, b, a]| TySrgbaF64::new(r, g, b, a).to_u8().to_array())
            .unwrap_or(default),

        VoxValuePool::LinearRgb { values } => values
            .get(index)
            .map(|&[r, g, b]| {
                TyLinSrgbaF64::new(r, g, b, 1.0)
                    .to_srgba()
                    .to_u8()
                    .to_array()
            })
            .unwrap_or(default),

        VoxValuePool::LinearRgba { values } => values
            .get(index)
            .map(|&[r, g, b, a]| TyLinSrgbaF64::new(r, g, b, a).to_srgba().to_u8().to_array())
            .unwrap_or(default),
        _ => default,
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

/// A scalar attribute's value from a `float`, `int`, or `bool` pool, defaulting
/// to its spec default (or `0` for a key with no standard default) when absent or
/// not one of those pools. A `bool` reads as `1` or `0`.
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
        Some((VoxValuePool::Bool { values }, index)) => values
            .get(index as usize)
            .map(|&value| if value { 1.0 } else { 0.0 })
            .unwrap_or_else(fallback),
        _ => fallback(),
    }
}

/// The emissive texel for the material at `index`: its `emissiveFactor`
/// scaled in linear light by `emissiveStrength / max_strength`. Per-voxel
/// strengths survive as a gradient, and a flat
/// `KHR_materials_emissive_strength` of `max_strength` restores the absolute
/// scale. An absent color is black.
fn emissive_color_bytes(
    state: &VoxMain,
    used: &UsedMaterials,
    index: usize,
    max_strength: f64,
) -> [u8; 4] {
    let color = color_bytes_or(
        material_attribute(state, used, index, EMISSIVE_FACTOR),
        [0, 0, 0, 255],
    );

    let fraction = if max_strength > 0.0 {
        material_scalar(state, used, index, EMISSIVE_STRENGTH) / max_strength
    } else {
        0.0
    };

    let linear = TySrgbaU8::from_array(color).to_f64().to_lin_srgba();

    TyLinSrgbaF64::new(
        linear.r * fraction,
        linear.g * fraction,
        linear.b * fraction,
        1.0,
    )
    .to_srgba()
    .to_u8()
    .to_array()
}

/// The greatest `emissiveStrength` among the used materials.
pub(crate) fn max_emissive_strength(state: &VoxMain, used: &UsedMaterials) -> f64 {
    (0..used.len())
        .map(|index| material_scalar(state, used, index, EMISSIVE_STRENGTH))
        .fold(0.0, f64::max)
}

/// The scalar attribute `key` of the material at `index` in `used`, or its spec
/// default when the material omits it. The flat factor the material document
/// writes back for the KHR extension attributes.
pub(crate) fn material_scalar(
    state: &VoxMain,
    used: &UsedMaterials,
    index: usize,
    key: &str,
) -> f64 {
    scalar_value(material_attribute(state, used, index, key), key)
}

#[cfg(test)]
mod tests {
    use crate::{
        AtlasShape, BASE_COLOR_FACTOR, ColorChannel, EMISSIVE_FACTOR, EMISSIVE_STRENGTH,
        METALLIC_FACTOR, MaterialBake, MaterialChannel, OCCLUSION_STRENGTH, ROUGHNESS_FACTOR,
        atlas_dimensions, bake_atlas_pixels, resolve_used_materials,
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

        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();
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
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();

        let metallic = MaterialBake::Packing(vec![scalar(METALLIC_FACTOR, false)]);
        let pixels = bake_atlas_pixels(&state, &used, &metallic, width, height).unwrap();
        // Only material 0 is metallic (1.0); the rest are matte (0.0).
        assert_eq!(pixels[0], 255);
        assert_eq!(pixels[4], 0);
        assert_eq!(pixels[8], 0);
    }

    #[test]
    fn a_bool_packing_reads_one_or_zero() {
        let mut state = VoxMain::default();
        let flag = state.add_value_pool(VoxValuePool::Bool {
            values: vec![true, false],
        });

        let mut palette = VoxPalette::default();
        palette.add_binding("flag".to_owned(), flag);
        let on = palette.add_material(vec![0]).unwrap();
        let off = palette.add_material(vec![1]).unwrap();
        let palette_id = state.add_palette(palette);

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        let layer = object.add_layer(palette_id, on);
        for (x, material) in [(0, on), (1, off)] {
            let voxel = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel, &[material]).unwrap();
        }
        let object_id = state.add_object(object);

        let used = resolve_used_materials(state.object(object_id).unwrap(), layer).unwrap();
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();

        let mask = MaterialBake::Packing(vec![scalar("flag", false)]);
        let pixels = bake_atlas_pixels(&state, &used, &mask, width, height).unwrap();
        // A true voxel bakes 255, a false voxel bakes 0.
        assert_eq!(pixels[0], 255);
        assert_eq!(pixels[4], 0);
    }

    #[test]
    fn inversion_turns_roughness_into_smoothness() {
        let (state, object_id, layer) = single_layer_state();
        let used = resolve_used_materials(state.object(object_id).unwrap(), layer).unwrap();
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();

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
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();

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
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();

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
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();

        let bake = MaterialBake::Packing(vec![MaterialChannel::ComputedOcclusion]);
        assert!(bake_atlas_pixels(&state, &used, &bake, width, height).is_err());
    }

    #[test]
    fn emissive_color_folds_strength_toward_the_mesh_max() {
        let mut state = VoxMain::default();

        // emissiveFactor is sRGB; strengths 1.0 and 0.5 fold into the texels as
        // fractions of the mesh max, 1.0.
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
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();
        let pixels =
            bake_atlas_pixels(&state, &used, &MaterialBake::EmissiveColor, width, height).unwrap();

        // Blue at the max strength stays full blue.
        assert_eq!(&pixels[0..4], &[0, 0, 255, 255]);

        // White at half the max strength folds in linear light to a light gray,
        // equal across channels and below full white.
        let [r, g, b, a] = [pixels[4], pixels[5], pixels[6], pixels[7]];
        assert_eq!((g, b, a), (r, r, 255));
        assert!((180..=195).contains(&r), "half strength folded to {r}");
    }
}
