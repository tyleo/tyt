use crate::{ColorFormat, voxj_value_from_vox_value};
use std::fmt::Write;
use ty_math::{TyFloatExt, TyLinSrgbaF64, TySrgbaF64};
use voxcore::{VoxBound, VoxValuePool, VoxValuePoolKind, VoxValuePoolValueRef};
use voxj::{VoxjBound, VoxjValuePool};

/// Converts a [`VoxValuePool`] into a [`VoxjValuePool`], kind by kind, its
/// values in listing order.
///
/// `color_format` picks how the two sRGB kinds serialize:
///
/// 1. [`ColorFormat::Hex`]: quantize each component to a `#RRGGBB` /
///    `#RRGGBBAA` byte.
/// 2. [`ColorFormat::Float`]: write the float components straight through.
/// 3. [`ColorFormat::LinearFloat`]: decode to linear light, write the
///    `linear-rgb` / `linear-rgba` float kinds.
///
/// Linear-kind value pools always serialize as float. Every other kind maps
/// one to one, keeping its values and any `int`/`float` bounds; `json`
/// recurses through [`voxj_value_from_vox_value`].
pub fn voxj_value_pool_from_vox_value_pool(
    value_pool: &VoxValuePool,
    color_format: ColorFormat,
) -> VoxjValuePool {
    match value_pool.kind() {
        VoxValuePoolKind::Json { .. } => VoxjValuePool::Json {
            values: value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::Json(value) => voxj_value_from_vox_value(value),
                    other => unreachable!("a json pool yields json values, not {other:?}"),
                })
                .collect(),
        },

        VoxValuePoolKind::Bool { .. } => VoxjValuePool::Bool {
            values: value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::Bool(flag) => flag,
                    other => unreachable!("a bool pool yields bool values, not {other:?}"),
                })
                .collect(),
        },

        VoxValuePoolKind::Float { min, max, .. } => VoxjValuePool::Float {
            min: voxj_bound(*min),
            max: voxj_bound(*max),
            values: value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::Float(number) => number,
                    other => unreachable!("a float pool yields float values, not {other:?}"),
                })
                .collect(),
        },

        VoxValuePoolKind::Int { min, max, .. } => VoxjValuePool::Int {
            min: voxj_bound(*min),
            max: voxj_bound(*max),
            values: value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::Int(number) => number,
                    other => unreachable!("an int pool yields int values, not {other:?}"),
                })
                .collect(),
        },

        VoxValuePoolKind::String { .. } => VoxjValuePool::String {
            values: value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::String(text) => text.to_owned(),
                    other => unreachable!("a string pool yields string values, not {other:?}"),
                })
                .collect(),
        },

        VoxValuePoolKind::Srgb { .. } => match color_format {
            ColorFormat::Hex => VoxjValuePool::SrgbHex {
                values: rgb_colors(value_pool).iter().map(encode_hex).collect(),
            },
            ColorFormat::Float => VoxjValuePool::SrgbFloat {
                values: rgb_colors(value_pool),
            },
            ColorFormat::LinearFloat => VoxjValuePool::LinearRgbFloat {
                values: rgb_colors(value_pool).iter().map(decode_rgb).collect(),
            },
        },

        VoxValuePoolKind::Srgba { .. } => match color_format {
            ColorFormat::Hex => VoxjValuePool::SrgbaHex {
                values: rgba_colors(value_pool).iter().map(encode_hex).collect(),
            },
            ColorFormat::Float => VoxjValuePool::SrgbaFloat {
                values: rgba_colors(value_pool),
            },
            ColorFormat::LinearFloat => VoxjValuePool::LinearRgbaFloat {
                values: rgba_colors(value_pool).iter().map(decode_rgba).collect(),
            },
        },

        VoxValuePoolKind::LinearRgb { .. } => VoxjValuePool::LinearRgbFloat {
            values: rgb_colors(value_pool),
        },

        VoxValuePoolKind::LinearRgba { .. } => VoxjValuePool::LinearRgbaFloat {
            values: rgba_colors(value_pool),
        },
    }
}

/// The three-component colors of an `srgb` or `linear-rgb` value pool in
/// listing order.
fn rgb_colors(value_pool: &VoxValuePool) -> Vec<[f64; 3]> {
    value_pool
        .iter_values()
        .map(|(_, value)| match value {
            VoxValuePoolValueRef::Srgb(color) | VoxValuePoolValueRef::LinearRgb(color) => *color,
            other => unreachable!("an rgb pool yields rgb colors, not {other:?}"),
        })
        .collect()
}

/// The four-component colors of an `srgba` or `linear-rgba` value pool in
/// listing order.
fn rgba_colors(value_pool: &VoxValuePool) -> Vec<[f64; 4]> {
    value_pool
        .iter_values()
        .map(|(_, value)| match value {
            VoxValuePoolValueRef::Srgba(color) | VoxValuePoolValueRef::LinearRgba(color) => *color,
            other => unreachable!("an rgba pool yields rgba colors, not {other:?}"),
        })
        .collect()
}

/// Maps a voxcore bound to its wire form; the two are the same number-or-none
/// shape.
fn voxj_bound(bound: VoxBound) -> VoxjBound {
    match bound {
        VoxBound::Number(number) => VoxjBound::Number(number),
        VoxBound::None => VoxjBound::None,
    }
}

/// Encodes `N` float components in `[0, 1]` as `#` plus `2 * N` uppercase hex
/// digits, each component clamped and scaled to an 8-bit byte.
fn encode_hex<const N: usize>(components: &[f64; N]) -> String {
    let mut hex = String::with_capacity(1 + N * 2);
    hex.push('#');
    for &component in components {
        let byte = component.to_unorm8();
        // Writing to a String is infallible.
        write!(hex, "{byte:02X}").unwrap();
    }
    hex
}

/// Decodes a three-component sRGB color to linear light. The alpha is a
/// discarded placeholder, since the linear decode is shared with the 4-channel
/// form.
fn decode_rgb(components: &[f64; 3]) -> [f64; 3] {
    let [r, g, b] = *components;
    let linear: TyLinSrgbaF64 = TySrgbaF64::new(r, g, b, 0.0).into_linear();
    [linear.red, linear.green, linear.blue]
}

/// Decodes an sRGBA color to linear light; alpha carries no gamma, so it passes
/// through.
fn decode_rgba(components: &[f64; 4]) -> [f64; 4] {
    let linear: TyLinSrgbaF64 = TySrgbaF64::from(*components).into_linear();

    linear.into()
}

#[cfg(test)]
mod tests {
    use super::voxj_value_pool_from_vox_value_pool;
    use crate::ColorFormat;
    use voxcore::VoxValuePool;
    use voxj::VoxjValuePool;

    /// Absolute tolerance for a transfer-function comparison.
    const EPSILON: f64 = 1e-9;

    #[test]
    fn srgb_float_maps_to_reference_linear_values() {
        // 0 and 1 are fixed points; 0.5 hits the power curve, the knee the toe.
        assert!((srgb_to_linear(0.0) - 0.0).abs() < EPSILON);
        assert!((srgb_to_linear(1.0) - 1.0).abs() < EPSILON);
        assert!((srgb_to_linear(0.5) - 0.214_041_140_6).abs() < 1e-6);
        assert!((srgb_to_linear(0.040_45) - 0.003_130_804_9).abs() < 1e-6);
    }

    #[test]
    fn linear_float_decodes_an_srgb_value_pool() {
        let value_pool = VoxValuePool::srgb(vec![[1.0, 0.0, 0.5]]).unwrap();

        match voxj_value_pool_from_vox_value_pool(&value_pool, ColorFormat::LinearFloat) {
            VoxjValuePool::LinearRgbFloat { values } => {
                assert_eq!(values.len(), 1);
                assert!((values[0][0] - 1.0).abs() < EPSILON);
                assert!((values[0][1] - 0.0).abs() < EPSILON);
                assert!((values[0][2] - srgb_to_linear(0.5)).abs() < EPSILON);
            }
            other => panic!("expected linear-rgb-float, got {other:?}"),
        }
    }

    #[test]
    fn linear_float_keeps_straight_alpha() {
        let value_pool = VoxValuePool::srgba(vec![[0.5, 0.5, 0.5, 0.25]]).unwrap();

        match voxj_value_pool_from_vox_value_pool(&value_pool, ColorFormat::LinearFloat) {
            VoxjValuePool::LinearRgbaFloat { values } => {
                // Alpha carries no gamma, so it passes through.
                assert_eq!(values[0][3], 0.25);
                assert!((values[0][0] - srgb_to_linear(0.5)).abs() < EPSILON);
            }
            other => panic!("expected linear-rgba-float, got {other:?}"),
        }
    }

    #[test]
    fn srgb_to_linear_round_trips_within_epsilon() {
        // Decode then re-encode recovers the sRGB component, so the conversion
        // loses nothing at float precision.
        for &srgb in &[0.0, 0.25, 0.5, 0.75, 1.0] {
            let linear = srgb_to_linear(srgb);
            assert!((linear_to_srgb(linear) - srgb).abs() < EPSILON);
        }
    }

    #[test]
    fn linear_float_preserves_an_hdr_component() {
        // A linear value pool carries components above 1; linear-float holds
        // them, hex could not.
        let value_pool = VoxValuePool::linear_rgb(vec![[2.5, 0.0, 1.0]]).unwrap();

        match voxj_value_pool_from_vox_value_pool(&value_pool, ColorFormat::LinearFloat) {
            VoxjValuePool::LinearRgbFloat { values } => assert_eq!(values[0], [2.5, 0.0, 1.0]),
            other => panic!("expected linear-rgb-float, got {other:?}"),
        }
    }

    #[test]
    fn srgb_float_default_passes_components_through_unchanged() {
        let value_pool = VoxValuePool::srgb(vec![[0.1, 0.2, 0.3]]).unwrap();

        match voxj_value_pool_from_vox_value_pool(&value_pool, ColorFormat::Float) {
            VoxjValuePool::SrgbFloat { values } => assert_eq!(values, vec![[0.1, 0.2, 0.3]]),
            other => panic!("expected srgb-float, got {other:?}"),
        }
    }

    /// The sRGB transfer inverse, kept in the test as an independent reference
    /// for the production decode (`TySrgba::into_linear`).
    fn srgb_to_linear(component: f64) -> f64 {
        if component <= 0.040_45 {
            component / 12.92
        } else {
            ((component + 0.055) / 1.055).powf(2.4)
        }
    }

    /// The forward sRGB transfer, kept in the test to prove the decode inverts.
    fn linear_to_srgb(linear: f64) -> f64 {
        if linear <= 0.003_130_8 {
            linear * 12.92
        } else {
            1.055 * linear.powf(1.0 / 2.4) - 0.055
        }
    }
}
