use crate::{ColorChannel, Error, MaterialBake, MaterialChannel, MaterialMap, Result};
use voxcore::{
    VoxEffectivePalette, VoxMain, VoxObject, VoxValuePool, VoxValuePoolKind,
    material::{BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, MaterialPropertyKind},
};

/// Checks every map's property reads against `object`'s effective palette
/// before anything bakes, each key's kind read through its winning layer.
pub(crate) fn check_material_maps<T>(
    state: &VoxMain<T>,
    object: &VoxObject,
    maps: &[MaterialMap],
) -> Result<()> {
    let effective = state.effective_palette(object)?;

    for map in maps {
        match &map.bake {
            MaterialBake::Packing(channels) => {
                for channel in channels {
                    let MaterialChannel::Property { key, component, .. } = channel else {
                        continue;
                    };

                    check_channel(&effective, key, *component)?;
                }
            }

            // The color bakes name no channels. Each reads a fixed property
            // whole, so it is checked against the kind it reads that property as.
            MaterialBake::RgbaColor => check_color(&effective, BASE_COLOR)?,

            MaterialBake::EmissiveColor => {
                check_color(&effective, EMISSIVE_COLOR)?;
                check_scalar(&effective, EMISSIVE_STRENGTH)?;
            }
        }
    }

    Ok(())
}

/// The kind a channel reads a property as, fixing whether it may pick a color
/// component.
enum ChannelKind {
    Color { alpha: bool },
    Scalar,
}

/// Errors unless `key` reads as a color, for a bake that takes it whole.
fn check_color(effective: &VoxEffectivePalette, key: &str) -> Result<()> {
    match channel_kind(effective, key)? {
        ChannelKind::Color { .. } => Ok(()),
        ChannelKind::Scalar => Err(Error::invalid(format!(
            "`{key}` is a scalar, and this bake reads it as a color"
        ))),
    }
}

/// Errors unless `key` reads as a scalar, for a bake that takes it as a plain
/// factor.
fn check_scalar(effective: &VoxEffectivePalette, key: &str) -> Result<()> {
    match channel_kind(effective, key)? {
        ChannelKind::Scalar => Ok(()),
        ChannelKind::Color { .. } => Err(Error::invalid(format!(
            "`{key}` is a color, and this bake reads it as a scalar"
        ))),
    }
}

/// Checks one property channel's component against the key's kind.
fn check_channel(
    effective: &VoxEffectivePalette,
    key: &str,
    component: Option<ColorChannel>,
) -> Result<()> {
    match channel_kind(effective, key)? {
        ChannelKind::Color { alpha } => match component {
            None => Err(Error::invalid(format!(
                "`{key}` is a color; a channel reads one component of it"
            ))),
            Some(ColorChannel::A) if !alpha => Err(Error::invalid(format!(
                "`{key}` is a color with no alpha; read r, g, or b"
            ))),
            _ => Ok(()),
        },
        ChannelKind::Scalar => {
            if component.is_some() {
                return Err(Error::invalid(format!(
                    "`{key}` is a scalar and has no component"
                )));
            }

            Ok(())
        }
    }
}

/// The kind of the property `key`. A glTF vocabulary name takes its
/// vocabulary kind whatever shape it is bound to. A custom key takes the
/// shape of its winning value pool, and errors when unbound: it has no spec
/// default and no shape to read.
fn channel_kind(effective: &VoxEffectivePalette, key: &str) -> Result<ChannelKind> {
    if let Some(kind) = MaterialPropertyKind::of(key) {
        return Ok(match kind {
            MaterialPropertyKind::ColorRgba => ChannelKind::Color { alpha: true },
            MaterialPropertyKind::ColorRgb => ChannelKind::Color { alpha: false },
            MaterialPropertyKind::Scalar => ChannelKind::Scalar,
        });
    }

    let Some(property_id) = effective.property_id_by_name(key) else {
        return Err(Error::invalid(format!(
            "`{key}` is not bound by any of the object's layers, so its type cannot be \
             inferred; bind it in a palette or read a glTF property"
        )));
    };

    let value_pool = effective
        .property(property_id)
        .expect("a resolved name identifies one of the effective palette's properties")
        .value_pool();

    value_pool_kind(value_pool, key)
}

/// Classifies a bound custom property by its value pool's shape. A component
/// read on a float vector is the caller's color assertion.
fn value_pool_kind(value_pool: &VoxValuePool, key: &str) -> Result<ChannelKind> {
    match value_pool.kind() {
        VoxValuePoolKind::Bool(_) | VoxValuePoolKind::Float(_) | VoxValuePoolKind::Int(_) => {
            Ok(ChannelKind::Scalar)
        }
        VoxValuePoolKind::Vec3Float(_) => Ok(ChannelKind::Color { alpha: false }),
        VoxValuePoolKind::Vec4Float(_) => Ok(ChannelKind::Color { alpha: true }),
        VoxValuePoolKind::Json(_)
        | VoxValuePoolKind::String(_)
        | VoxValuePoolKind::Vec2Float(_)
        | VoxValuePoolKind::Vec2Int(_)
        | VoxValuePoolKind::Vec3Int(_)
        | VoxValuePoolKind::Vec4Int(_) => Err(Error::invalid(format!(
            "`{key}` is bound to a value pool with no texel value; a channel reads a float, \
             int, or bool scalar or a vec-3-float or vec-4-float color"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{check_channel, check_color, check_scalar};
    use crate::ColorChannel;
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{BVoxPalette, BVoxValuePoolValue, VoxMain, VoxObject, VoxPalette, VoxValuePool};

    /// The branded value id `index`.
    fn value_id(index: usize) -> U32Id<BVoxValuePoolValue> {
        U32Id::from_u32(index as u32)
    }

    /// A one-palette document binding `tint` to a four-component color, `glow`
    /// to a three-component color, and `gloss` to a float scalar.
    fn palette_state() -> (VoxMain, U32Id<BVoxPalette>) {
        let mut state = VoxMain::default();

        let tint_value_pool_id =
            state.retain_value_pool(VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap());
        let glow_value_pool_id =
            state.retain_value_pool(VoxValuePool::vec_3_float(vec![[0.0, 1.0, 0.0]]).unwrap());
        let gloss_value_pool_id = state.retain_value_pool(VoxValuePool::float(vec![0.5]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .retain_property("tint".to_owned(), tint_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette
            .retain_property("glow".to_owned(), glow_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette
            .retain_property("gloss".to_owned(), gloss_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette
            .retain_material(vec![value_id(0), value_id(0), value_id(0)])
            .unwrap();

        let palette_id = state.retain_palette(palette).unwrap();

        (state, palette_id)
    }

    /// Whether `key` with `component` checks against an object layering the
    /// given palettes in order.
    fn checks(
        state: &VoxMain,
        palette_ids: &[U32Id<BVoxPalette>],
        key: &str,
        component: Option<ColorChannel>,
    ) -> bool {
        let mut object = VoxObject::new("body".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        for &palette_id in palette_ids {
            object.retain_layer(palette_id, U32Id::from_u32(0));
        }
        let effective = state.effective_palette(&object).unwrap();
        check_channel(&effective, key, component).is_ok()
    }

    #[test]
    fn a_present_color_reads_by_component() {
        let (state, palette_id) = palette_state();
        // `tint` is a vec-4-float value pool: a component is required, and `A`
        // is allowed.
        assert!(!checks(&state, &[palette_id], "tint", None));
        assert!(checks(&state, &[palette_id], "tint", Some(ColorChannel::R)));
        assert!(checks(&state, &[palette_id], "tint", Some(ColorChannel::A)));
    }

    #[test]
    fn a_present_three_component_color_rejects_alpha() {
        let (state, palette_id) = palette_state();
        assert!(checks(&state, &[palette_id], "glow", Some(ColorChannel::B)));
        assert!(!checks(
            &state,
            &[palette_id],
            "glow",
            Some(ColorChannel::A)
        ));
    }

    #[test]
    fn a_present_scalar_rejects_a_component() {
        let (state, palette_id) = palette_state();
        assert!(checks(&state, &[palette_id], "gloss", None));
        assert!(!checks(
            &state,
            &[palette_id],
            "gloss",
            Some(ColorChannel::R)
        ));
    }

    #[test]
    fn an_absent_builtin_takes_its_spec_kind() {
        let (state, palette_id) = palette_state();
        // None are bound, so each checks by its glTF spec kind and bakes its
        // default: baseColor is a four-component color, occlusionStrength a
        // scalar, emissiveColor a three-component color.
        assert!(checks(
            &state,
            &[palette_id],
            "baseColor",
            Some(ColorChannel::A)
        ));
        assert!(!checks(&state, &[palette_id], "baseColor", None));
        assert!(checks(&state, &[palette_id], "occlusionStrength", None));
        assert!(!checks(
            &state,
            &[palette_id],
            "occlusionStrength",
            Some(ColorChannel::R)
        ));
        assert!(!checks(
            &state,
            &[palette_id],
            "emissiveColor",
            Some(ColorChannel::A)
        ));
    }

    #[test]
    fn a_bound_builtin_takes_its_vocabulary_kind() {
        // The vocabulary kind wins over the bound shape: `metallic` stays a
        // scalar on a vec-4-float value pool.
        let mut state = VoxMain::default();
        let value_pool_id =
            state.retain_value_pool(VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property("metallic".to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        let palette_id = state.retain_palette(palette).unwrap();

        assert!(checks(&state, &[palette_id], "metallic", None));
        assert!(!checks(
            &state,
            &[palette_id],
            "metallic",
            Some(ColorChannel::R)
        ));
    }

    #[test]
    fn an_absent_custom_property_is_an_error() {
        let (state, palette_id) = palette_state();
        // `subsurface` is neither bound nor a glTF property, so its type cannot
        // be inferred, whether or not a component is picked.
        assert!(!checks(&state, &[palette_id], "subsurface", None));
        assert!(!checks(
            &state,
            &[palette_id],
            "subsurface",
            Some(ColorChannel::R)
        ));
    }

    /// Whether `key` checks as a whole color, then as a scalar, against an
    /// object layering the given palettes in order.
    fn checks_whole(
        state: &VoxMain,
        palette_ids: &[U32Id<BVoxPalette>],
        key: &str,
    ) -> (bool, bool) {
        let mut object = VoxObject::new("body".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        for &palette_id in palette_ids {
            object.retain_layer(palette_id, U32Id::from_u32(0));
        }
        let effective = state.effective_palette(&object).unwrap();
        (
            check_color(&effective, key).is_ok(),
            check_scalar(&effective, key).is_ok(),
        )
    }

    #[test]
    fn a_whole_read_property_is_checked_against_the_kind_it_is_read_as() {
        // The rgba and emissive bakes name no channel, so each fixed property
        // is checked whole before it reaches the baker.
        let (state, palette_id) = palette_state();
        assert_eq!(checks_whole(&state, &[palette_id], "tint"), (true, false));
        assert_eq!(checks_whole(&state, &[palette_id], "gloss"), (false, true));
    }

    #[test]
    fn a_string_value_pool_has_no_texel_value() {
        let mut state = VoxMain::default();
        let tag_value_pool_id =
            state.retain_value_pool(VoxValuePool::string(vec!["low".to_owned()]));
        let mut palette = VoxPalette::default();
        palette
            .retain_property("tag".to_owned(), tag_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        let palette_id = state.retain_palette(palette).unwrap();

        assert!(!checks(&state, &[palette_id], "tag", None));
    }

    #[test]
    fn an_int_vector_value_pool_has_no_texel_value() {
        let mut state = VoxMain::default();
        let cell_value_pool_id =
            state.retain_value_pool(VoxValuePool::vec_3_int(vec![[1, 2, 3]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property("cell".to_owned(), cell_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        let palette_id = state.retain_palette(palette).unwrap();

        assert!(!checks(&state, &[palette_id], "cell", None));
        assert!(!checks(
            &state,
            &[palette_id],
            "cell",
            Some(ColorChannel::R)
        ));
    }

    #[test]
    fn a_key_takes_its_winning_layers_kind() {
        // Two palettes bind `finish` to different kinds: a float scalar and a
        // four-component color. The last layer's palette wins, so the layer
        // order flips which component rule applies.
        let mut state = VoxMain::default();
        let scalar_value_pool_id = state.retain_value_pool(VoxValuePool::float(vec![0.5]).unwrap());
        let color_value_pool_id =
            state.retain_value_pool(VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap());

        let mut scalar_palette = VoxPalette::default();
        scalar_palette
            .retain_property(
                "finish".to_owned(),
                scalar_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        scalar_palette.retain_material(vec![value_id(0)]).unwrap();
        let scalar_palette_id = state.retain_palette(scalar_palette).unwrap();

        let mut color_palette = VoxPalette::default();
        color_palette
            .retain_property("finish".to_owned(), color_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        color_palette.retain_material(vec![value_id(0)]).unwrap();
        let color_palette_id = state.retain_palette(color_palette).unwrap();

        // Color wins: a component is required and `A` allowed.
        assert!(!checks(
            &state,
            &[scalar_palette_id, color_palette_id],
            "finish",
            None
        ));
        assert!(checks(
            &state,
            &[scalar_palette_id, color_palette_id],
            "finish",
            Some(ColorChannel::A)
        ));

        // Scalar wins: no component allowed.
        assert!(checks(
            &state,
            &[color_palette_id, scalar_palette_id],
            "finish",
            None
        ));
        assert!(!checks(
            &state,
            &[color_palette_id, scalar_palette_id],
            "finish",
            Some(ColorChannel::R)
        ));
    }
}
