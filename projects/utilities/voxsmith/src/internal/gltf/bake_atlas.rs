use crate::{
    BASE_COLOR, ColorChannel, EMISSIVE_COLOR, EMISSIVE_STRENGTH, Error, MaterialBake,
    MaterialChannel, Result, UsedMaterials, default_color, default_scalar, lin_srgba_from_srgba_u8,
    srgba_u8_from_lin_srgba, value_pool_linear_color,
};
use branded_id::U32Id;
use ty_math::{TyFloatExt, TyLinSrgbaF64, TySrgbaU8};
use voxcore::{BVoxValuePoolValue, VoxValuePool, VoxValuePoolValueRef};

/// Bakes `bake` over every material in `used` into an RGBA8 pixel buffer of
/// `width` x `height` texels, one texel per material placed row-major from the
/// top-left. Trailing texels past the material count stay transparent black. The
/// buffer's layout matches [`atlas_dimensions`](crate::atlas_dimensions) over
/// `used.len()`, so the UVs sampling it read each material's texel.
pub(crate) fn bake_atlas_pixels(
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
        MaterialBake::EmissiveColor => max_emissive_strength(used)?,
        _ => 0.0,
    };

    for index in 0..used.len() {
        let rgba = bake_texel(used, bake, index, max_strength)?;

        let offset = index * 4;

        pixels[offset..offset + 4].copy_from_slice(&rgba);
    }

    Ok(pixels)
}

/// The RGBA bytes of the material at `index` under `bake`.
fn bake_texel(
    used: &UsedMaterials,
    bake: &MaterialBake,
    index: usize,
    max_strength: f64,
) -> Result<[u8; 4]> {
    match bake {
        MaterialBake::RgbaColor => color_bytes(used.attribute(index, BASE_COLOR), BASE_COLOR),

        MaterialBake::EmissiveColor => emissive_color_bytes(used, index, max_strength),

        MaterialBake::Packing(channels) => {
            // A packing fills R, G, B from its channels; an unnamed channel and
            // an absent alpha stay `0` and opaque respectively.
            let mut rgba = [0u8, 0u8, 0u8, 255u8];

            for (channel_index, channel) in channels.iter().enumerate() {
                rgba[channel_index] = channel_byte(used, index, channel)?;
            }

            Ok(rgba)
        }
    }
}

/// The `0..255` byte one channel contributes for the material at `index`.
fn channel_byte(used: &UsedMaterials, index: usize, channel: &MaterialChannel) -> Result<u8> {
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
            let value = used.attribute(index, key);

            // Read the source as a `0..1` fraction, invert if asked, then scale
            // to a byte, so a scalar and a color component inject the same way.
            let fraction = match component {
                Some(component) => {
                    component_byte(color_bytes(value, key)?, *component) as f64 / 255.0
                }
                None => scalar_value(value, key)?,
            };

            // The check precedes the conversion, since `to_unorm8` clamps on its
            // own and an attribute the spec leaves unbounded above, such as
            // `emissiveStrength`, reaches here.
            if !(0.0..=1.0).contains(&fraction) {
                return Err(Error::invalid(format!(
                    "`{key}` is {fraction}, outside the 0 to 1 a packed channel stores"
                )));
            }

            let fraction = if *invert { 1.0 - fraction } else { fraction };

            Ok(fraction.to_unorm8())
        }
    }
}

/// A color attribute's RGBA bytes: [`linear_color`] re-encoded to sRGB.
fn color_bytes(
    value: Option<(&VoxValuePool, U32Id<BVoxValuePoolValue>)>,
    key: &str,
) -> Result<[u8; 4]> {
    let color = linear_color(value, key)?;
    Ok(<[u8; 4]>::from(srgba_u8_from_lin_srgba(
        TyLinSrgbaF64::from(color),
    )))
}

/// A color attribute's linear-light components, or `key`'s glTF spec default
/// when no layer binds it. A three-component color takes opaque alpha. Errors
/// when the bound value pool holds no float vectors, or when an unbound `key`
/// has no spec default.
fn linear_color(
    value: Option<(&VoxValuePool, U32Id<BVoxValuePoolValue>)>,
    key: &str,
) -> Result<[f64; 4]> {
    let Some((value_pool, value_id)) = value else {
        let bytes = default_color(key).ok_or_else(|| unbound(key))?;
        return Ok(<[f64; 4]>::from(lin_srgba_from_srgba_u8(TySrgbaU8::from(
            bytes,
        ))));
    };

    value_pool_linear_color(value_pool, value_id)
        .ok_or_else(|| Error::invalid(format!("`{key}` draws from a value pool holding no color")))
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

/// A scalar attribute's value from a `float`, `int`, or `bool` value pool, or
/// `key`'s glTF spec default when no layer binds it. A `bool` reads as `1` or
/// `0`. Errors when the bound value pool is not one of those kinds, or when an
/// unbound `key` has no spec default.
fn scalar_value(
    value: Option<(&VoxValuePool, U32Id<BVoxValuePoolValue>)>,
    key: &str,
) -> Result<f64> {
    let Some((value_pool, value_id)) = value else {
        return default_scalar(key).ok_or_else(|| unbound(key));
    };

    // `VoxMain::add_palette` checks every material's value against its
    // property's value pool, so a bound attribute always resolves.
    let value = value_pool
        .value(value_id)
        .expect("a palette's material draws a value its property's value pool holds");

    match value {
        VoxValuePoolValueRef::Float(number) => Ok(number),
        VoxValuePoolValueRef::Int(number) => Ok(number as f64),
        VoxValuePoolValueRef::Bool(flag) => Ok(if flag { 1.0 } else { 0.0 }),
        _ => Err(Error::invalid(format!(
            "`{key}` draws from a value pool holding no scalar"
        ))),
    }
}

/// The error for a `key` no layer binds that has no glTF spec default.
fn unbound(key: &str) -> Error {
    Error::invalid(format!(
        "`{key}` is not bound by any of the object's layers and has no glTF default"
    ))
}

/// The emissive texel for the material at `index`: its `emissiveColor`
/// scaled in linear light by `emissiveStrength / max_strength` and encoded
/// once, at the texel write. Per-voxel strengths survive as a gradient, and a
/// flat `KHR_materials_emissive_strength` of `max_strength` restores the
/// absolute scale. An absent color is black, its spec default.
fn emissive_color_bytes(used: &UsedMaterials, index: usize, max_strength: f64) -> Result<[u8; 4]> {
    let [red, green, blue, _] =
        linear_color(used.attribute(index, EMISSIVE_COLOR), EMISSIVE_COLOR)?;

    let fraction = if max_strength > 0.0 {
        material_scalar(used, index, EMISSIVE_STRENGTH)? / max_strength
    } else {
        0.0
    };

    let scaled = TyLinSrgbaF64::new(red * fraction, green * fraction, blue * fraction, 1.0);

    Ok(<[u8; 4]>::from(srgba_u8_from_lin_srgba(scaled)))
}

/// The greatest `emissiveStrength` among the used materials.
pub(crate) fn max_emissive_strength(used: &UsedMaterials) -> Result<f64> {
    (0..used.len())
        .map(|index| material_scalar(used, index, EMISSIVE_STRENGTH))
        .try_fold(0.0f64, |max, strength| strength.map(|value| max.max(value)))
}

/// The scalar attribute `key` of the material at `index` in `used`, or its spec
/// default when the material omits it. The flat factor the material document
/// writes back for the KHR extension attributes.
pub(crate) fn material_scalar(used: &UsedMaterials, index: usize, key: &str) -> Result<f64> {
    scalar_value(used.attribute(index, key), key)
}

#[cfg(test)]
mod tests {
    use crate::{
        AtlasShape, BASE_COLOR, ColorChannel, EMISSIVE_COLOR, EMISSIVE_STRENGTH, METALLIC,
        MaterialBake, MaterialChannel, OCCLUSION_STRENGTH, ROUGHNESS, Result, atlas_dimensions,
        bake_atlas_pixels, resolve_used_materials,
    };
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{BVoxObject, BVoxValuePoolValue, VoxMain, VoxObject, VoxPalette, VoxValuePool};

    /// The branded value id `index`.
    fn value_id(index: u32) -> U32Id<BVoxValuePoolValue> {
        U32Id::from_u32(index)
    }

    /// A `key`-only scalar packing channel.
    fn scalar(key: &str, invert: bool) -> MaterialChannel {
        MaterialChannel::Attribute {
            key: key.to_owned(),
            component: None,
            invert,
        }
    }

    /// A single-layer document: one palette carrying `baseColor`,
    /// `metallic`, and `roughness` over three value pools, with a
    /// three-voxel object whose voxels sample three materials in raster order:
    /// (red, shiny, smooth), (red, matte, rough), (blue, matte, rough).
    fn single_layer_state() -> (VoxMain, U32Id<BVoxObject>) {
        let mut state = VoxMain::default();

        let base_value_pool_id = state.add_value_pool(
            VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]]).unwrap(),
        );
        let metallic_value_pool_id =
            state.add_value_pool(VoxValuePool::float(vec![1.0, 0.0]).unwrap());
        let roughness_value_pool_id =
            state.add_value_pool(VoxValuePool::float(vec![0.0, 1.0]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .add_property(
                BASE_COLOR.to_owned(),
                base_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette
            .add_property(
                METALLIC.to_owned(),
                metallic_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette
            .add_property(
                ROUGHNESS.to_owned(),
                roughness_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        // Value ids: base 0 = red, 1 = blue; metallic 0 = 1.0, 1 = 0.0;
        // roughness 0 = 0.0, 1 = 1.0.
        let red_shiny_id = palette
            .add_material(vec![value_id(0), value_id(0), value_id(0)])
            .unwrap();
        let red_matte_id = palette
            .add_material(vec![value_id(0), value_id(1), value_id(1)])
            .unwrap();
        let blue_matte_id = palette
            .add_material(vec![value_id(1), value_id(1), value_id(1)])
            .unwrap();
        let palette_id = state.add_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(3, 1, 1)).unwrap();
        object.add_layer(palette_id, red_shiny_id);

        for (x, material_id) in [(0, red_shiny_id), (1, red_matte_id), (2, blue_matte_id)] {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel_id, &[material_id]).unwrap();
        }

        let object_id = state.add_object(object).unwrap();

        (state, object_id)
    }

    #[test]
    fn albedo_reads_the_base_color() {
        let (state, object_id) = single_layer_state();
        let object = state.object(object_id).unwrap();
        let used = resolve_used_materials(&state, object).unwrap();
        assert_eq!(used.len(), 3);

        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();
        let pixels = bake_atlas_pixels(&used, &MaterialBake::RgbaColor, width, height).unwrap();

        // Materials 0 and 1 both take the red base; material 2 is blue. The
        // fourth texel is padding.
        assert_eq!(&pixels[0..4], &[255, 0, 0, 255]);
        assert_eq!(&pixels[4..8], &[255, 0, 0, 255]);
        assert_eq!(&pixels[8..12], &[0, 0, 255, 255]);
        assert_eq!(&pixels[12..16], &[0, 0, 0, 0]);
    }

    #[test]
    fn a_scalar_packing_reads_the_metallic_property() {
        let (state, object_id) = single_layer_state();
        let used = resolve_used_materials(&state, state.object(object_id).unwrap()).unwrap();
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();

        let metallic = MaterialBake::Packing(vec![scalar(METALLIC, false)]);
        let pixels = bake_atlas_pixels(&used, &metallic, width, height).unwrap();
        // Only material 0 is metallic (1.0); the rest are matte (0.0).
        assert_eq!(pixels[0], 255);
        assert_eq!(pixels[4], 0);
        assert_eq!(pixels[8], 0);
    }

    #[test]
    fn a_bool_packing_reads_one_or_zero() {
        let mut state = VoxMain::default();
        let flag_value_pool_id =
            state.add_value_pool(VoxValuePool::boolean(vec![true, false]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .add_property("flag".to_owned(), flag_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        let on_id = palette.add_material(vec![value_id(0)]).unwrap();
        let off_id = palette.add_material(vec![value_id(1)]).unwrap();
        let palette_id = state.add_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        object.add_layer(palette_id, on_id);
        for (x, material_id) in [(0, on_id), (1, off_id)] {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel_id, &[material_id]).unwrap();
        }
        let object_id = state.add_object(object).unwrap();

        let used = resolve_used_materials(&state, state.object(object_id).unwrap()).unwrap();
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();

        let mask = MaterialBake::Packing(vec![scalar("flag", false)]);
        let pixels = bake_atlas_pixels(&used, &mask, width, height).unwrap();
        // A true voxel bakes 255, a false voxel bakes 0.
        assert_eq!(pixels[0], 255);
        assert_eq!(pixels[4], 0);
    }

    #[test]
    fn a_shared_value_pool_cell_bakes_one_value_for_every_material() {
        // Both material rows repeat the strength value pool's one cell, so both
        // bake the same half strength.
        let mut state = VoxMain::default();
        let base_value_pool_id = state.add_value_pool(
            VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]]).unwrap(),
        );
        let strength_value_pool_id = state.add_value_pool(VoxValuePool::float(vec![0.5]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .add_property(
                BASE_COLOR.to_owned(),
                base_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette
            .add_property(
                EMISSIVE_STRENGTH.to_owned(),
                strength_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        let red_id = palette
            .add_material(vec![value_id(0), value_id(0)])
            .unwrap();
        let blue_id = palette
            .add_material(vec![value_id(1), value_id(0)])
            .unwrap();
        let palette_id = state.add_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        object.add_layer(palette_id, red_id);
        for (x, material_id) in [(0, red_id), (1, blue_id)] {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel_id, &[material_id]).unwrap();
        }
        let object_id = state.add_object(object).unwrap();

        let used = resolve_used_materials(&state, state.object(object_id).unwrap()).unwrap();
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();

        let bake = MaterialBake::Packing(vec![scalar(EMISSIVE_STRENGTH, false)]);
        let pixels = bake_atlas_pixels(&used, &bake, width, height).unwrap();
        // 0.5 scales to the unorm byte 128 in both texels.
        assert_eq!(pixels[0], 128);
        assert_eq!(pixels[4], 128);
    }

    #[test]
    fn inversion_turns_roughness_into_smoothness() {
        let (state, object_id) = single_layer_state();
        let used = resolve_used_materials(&state, state.object(object_id).unwrap()).unwrap();
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();

        let smoothness = MaterialBake::Packing(vec![scalar(ROUGHNESS, true)]);
        let pixels = bake_atlas_pixels(&used, &smoothness, width, height).unwrap();
        // Material 0 is roughness 0.0 -> smoothness 1.0; material 1 is roughness
        // 1.0 -> smoothness 0.0.
        assert_eq!(pixels[0], 255);
        assert_eq!(pixels[4], 0);
    }

    #[test]
    fn a_missing_attribute_falls_back_to_its_spec_default() {
        let (state, object_id) = single_layer_state();
        let used = resolve_used_materials(&state, state.object(object_id).unwrap()).unwrap();
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();

        // The palette carries no `occlusionStrength`, whose spec default is 1.
        let occlusion = MaterialBake::Packing(vec![scalar(OCCLUSION_STRENGTH, false)]);
        let pixels = bake_atlas_pixels(&used, &occlusion, width, height).unwrap();
        assert_eq!(pixels[0], 255);
        assert_eq!(pixels[4], 255);
        assert_eq!(pixels[8], 255);
    }

    #[test]
    fn a_color_component_reads_one_channel() {
        let (state, object_id) = single_layer_state();
        let used = resolve_used_materials(&state, state.object(object_id).unwrap()).unwrap();
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();

        let red = MaterialBake::Packing(vec![MaterialChannel::Attribute {
            key: BASE_COLOR.to_owned(),
            component: Some(ColorChannel::R),
            invert: false,
        }]);
        let pixels = bake_atlas_pixels(&used, &red, width, height).unwrap();
        assert_eq!(pixels[0], 255); // material 0 is red
        assert_eq!(pixels[8], 0); // material 2 is blue
    }

    #[test]
    fn computed_occlusion_is_rejected_under_the_palette_atlas() {
        let (state, object_id) = single_layer_state();
        let used = resolve_used_materials(&state, state.object(object_id).unwrap()).unwrap();
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();

        let bake = MaterialBake::Packing(vec![MaterialChannel::ComputedOcclusion]);
        assert!(bake_atlas_pixels(&used, &bake, width, height).is_err());
    }

    #[test]
    fn emissive_color_folds_strength_toward_the_mesh_max() {
        let mut state = VoxMain::default();

        // emissiveColor is sRGB; strengths 1.0 and 0.5 fold into the texels as
        // fractions of the mesh max, 1.0.
        let factor_value_pool_id = state.add_value_pool(
            VoxValuePool::vec_3_float(vec![[0.0, 0.0, 1.0], [1.0, 1.0, 1.0]]).unwrap(),
        );
        let strength_value_pool_id =
            state.add_value_pool(VoxValuePool::float(vec![1.0, 0.5]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .add_property(
                EMISSIVE_COLOR.to_owned(),
                factor_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette
            .add_property(
                EMISSIVE_STRENGTH.to_owned(),
                strength_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        let full_blue_id = palette
            .add_material(vec![value_id(0), value_id(0)])
            .unwrap();
        let dim_white_id = palette
            .add_material(vec![value_id(1), value_id(1)])
            .unwrap();
        let palette_id = state.add_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        object.add_layer(palette_id, full_blue_id);
        for (x, material_id) in [(0, full_blue_id), (1, dim_white_id)] {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel_id, &[material_id]).unwrap();
        }
        let object_id = state.add_object(object).unwrap();

        let used = resolve_used_materials(&state, state.object(object_id).unwrap()).unwrap();
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();
        let pixels = bake_atlas_pixels(&used, &MaterialBake::EmissiveColor, width, height).unwrap();

        // Blue at the max strength stays full blue.
        assert_eq!(&pixels[0..4], &[0, 0, 255, 255]);

        // White at half the max strength folds in linear light to a light gray,
        // equal across channels and below full white.
        let [r, g, b, a] = [pixels[4], pixels[5], pixels[6], pixels[7]];
        assert_eq!((g, b, a), (r, r, 255));
        assert!((180..=195).contains(&r), "half strength folded to {r}");
    }

    /// Bakes `bake` over a one-voxel object whose one layer binds `key` to
    /// `value_pool`.
    fn bake_one(key: &str, value_pool: VoxValuePool, bake: &MaterialBake) -> Result<Vec<u8>> {
        let mut state = VoxMain::default();
        let value_pool_id = state.add_value_pool(value_pool);

        let mut palette = VoxPalette::default();
        palette
            .add_property(key.to_owned(), value_pool_id, value_id(0))
            .unwrap();
        let material_id = palette.add_material(vec![value_id(0)]).unwrap();
        let palette_id = state.add_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.add_layer(palette_id, material_id);
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel_id, &[material_id]).unwrap();
        let object_id = state.add_object(object).unwrap();

        let used = resolve_used_materials(&state, state.object(object_id).unwrap())?;
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit)?;

        bake_atlas_pixels(&used, bake, width, height)
    }

    /// An unbounded-above `float` value pool holding `value`, the shape
    /// `emissiveStrength` takes.
    fn strength_value_pool(value: f64) -> VoxValuePool {
        VoxValuePool::float(vec![value]).unwrap()
    }

    #[test]
    fn a_scalar_past_one_errors_rather_than_saturating() {
        // `emissiveStrength` runs 0 upward, so an HDR strength has no byte in a
        // packed channel and must not bake as full white.
        let bake = MaterialBake::Packing(vec![scalar(EMISSIVE_STRENGTH, false)]);
        let error = bake_one(EMISSIVE_STRENGTH, strength_value_pool(4.0), &bake).unwrap_err();

        let message = error.to_string();
        assert!(message.contains(EMISSIVE_STRENGTH), "{message}");
        assert!(message.contains('4'), "{message}");
    }

    #[test]
    fn a_scalar_at_the_range_ends_still_bakes() {
        // The check rejects only what the channel cannot hold. The endpoints fit.
        let bake = MaterialBake::Packing(vec![scalar(EMISSIVE_STRENGTH, false)]);
        let pixels = bake_one(EMISSIVE_STRENGTH, strength_value_pool(1.0), &bake).unwrap();
        assert_eq!(pixels[0], 255);

        let pixels = bake_one(EMISSIVE_STRENGTH, strength_value_pool(0.0), &bake).unwrap();
        assert_eq!(pixels[0], 0);
    }

    #[test]
    fn a_color_packed_as_a_scalar_errors() {
        let color = VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap();
        let bake = MaterialBake::Packing(vec![scalar(METALLIC, false)]);

        assert!(bake_one(METALLIC, color, &bake).is_err());
    }

    #[test]
    fn a_scalar_baked_as_a_color_errors() {
        // The rgba bake reads `baseColor` whole, so a scalar value pool
        // under it has no color to decode.
        let number = VoxValuePool::float(vec![0.5]).unwrap();

        assert!(bake_one(BASE_COLOR, number, &MaterialBake::RgbaColor).is_err());
    }

    #[test]
    fn an_unbound_custom_key_errors() {
        let (state, object_id) = single_layer_state();
        let used = resolve_used_materials(&state, state.object(object_id).unwrap()).unwrap();
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();

        // No layer binds `subsurface` and the glTF spec gives it no default, so
        // there is nothing to bake.
        let bake = MaterialBake::Packing(vec![scalar("subsurface", false)]);
        assert!(bake_atlas_pixels(&used, &bake, width, height).is_err());
    }

    #[test]
    fn an_unbound_color_component_bakes_its_own_spec_default() {
        let (state, object_id) = single_layer_state();
        let used = resolve_used_materials(&state, state.object(object_id).unwrap()).unwrap();
        let (width, height) = atlas_dimensions(used.len(), AtlasShape::Fit).unwrap();

        // The palette binds no `emissiveColor`. Its spec default is black,
        // while `baseColor` defaults to white.
        let bake = MaterialBake::Packing(vec![MaterialChannel::Attribute {
            key: EMISSIVE_COLOR.to_owned(),
            component: Some(ColorChannel::R),
            invert: false,
        }]);
        let pixels = bake_atlas_pixels(&used, &bake, width, height).unwrap();
        assert_eq!(pixels[0], 0);

        // Unbound, `baseColor` would take white. This palette binds it red.
        let bake = MaterialBake::Packing(vec![MaterialChannel::Attribute {
            key: BASE_COLOR.to_owned(),
            component: Some(ColorChannel::R),
            invert: false,
        }]);
        let pixels = bake_atlas_pixels(&used, &bake, width, height).unwrap();
        assert_eq!(pixels[0], 255);
    }
}
