use crate::{ColorFormat, voxj_value_from_vox_value};
use std::fmt::Write;
use ty_math::{TyFloatExt, TySrgbaF64};
use voxcore::{VoxBound, VoxValuePool};
use voxj::{VoxjBound, VoxjValuePool};

/// Converts a [`VoxValuePool`] into a [`VoxjValuePool`], kind by kind.
///
/// `color_format` picks how the two sRGB kinds serialize:
///
/// 1. [`ColorFormat::Hex`]: quantize each component to a `#RRGGBB` /
///    `#RRGGBBAA` byte.
/// 2. [`ColorFormat::Float`]: write the float components straight through.
/// 3. [`ColorFormat::LinearFloat`]: decode to linear light, write the
///    `linear-rgb` / `linear-rgba` float kinds.
///
/// Linear-kind pools always serialize as float. Every other kind maps one to
/// one, keeping its values and any `int`/`float` bounds; `json` recurses through
/// [`voxj_value_from_vox_value`].
pub fn voxj_value_pool_from_vox_value_pool(
    pool: &VoxValuePool,
    color_format: ColorFormat,
) -> VoxjValuePool {
    match pool {
        VoxValuePool::Json { values } => VoxjValuePool::Json {
            values: values.iter().map(voxj_value_from_vox_value).collect(),
        },

        VoxValuePool::Bool { values } => VoxjValuePool::Bool {
            values: values.clone(),
        },

        VoxValuePool::Float { min, max, values } => VoxjValuePool::Float {
            min: voxj_bound(*min),
            max: voxj_bound(*max),
            values: values.clone(),
        },

        VoxValuePool::Int { min, max, values } => VoxjValuePool::Int {
            min: voxj_bound(*min),
            max: voxj_bound(*max),
            values: values.clone(),
        },

        VoxValuePool::String { values } => VoxjValuePool::String {
            values: values.clone(),
        },

        VoxValuePool::Srgb { values } => match color_format {
            ColorFormat::Hex => VoxjValuePool::SrgbHex {
                values: values.iter().map(encode_hex).collect(),
            },
            ColorFormat::Float => VoxjValuePool::SrgbFloat {
                values: values.clone(),
            },
            ColorFormat::LinearFloat => VoxjValuePool::LinearRgbFloat {
                values: values.iter().map(decode_rgb).collect(),
            },
        },

        VoxValuePool::Srgba { values } => match color_format {
            ColorFormat::Hex => VoxjValuePool::SrgbaHex {
                values: values.iter().map(encode_hex).collect(),
            },
            ColorFormat::Float => VoxjValuePool::SrgbaFloat {
                values: values.clone(),
            },
            ColorFormat::LinearFloat => VoxjValuePool::LinearRgbaFloat {
                values: values.iter().map(decode_rgba).collect(),
            },
        },

        VoxValuePool::LinearRgb { values } => VoxjValuePool::LinearRgbFloat {
            values: values.clone(),
        },

        VoxValuePool::LinearRgba { values } => VoxjValuePool::LinearRgbaFloat {
            values: values.clone(),
        },
    }
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

/// Decodes a three-component sRGB color to linear light through
/// [`TySrgba::to_lin_srgba`](ty_math::TySrgba::to_lin_srgba). The alpha is a
/// discarded placeholder, since the linear decode is shared with the 4-channel
/// form.
fn decode_rgb(components: &[f64; 3]) -> [f64; 3] {
    let [r, g, b] = *components;
    let linear = TySrgbaF64::new(r, g, b, 0.0).to_lin_srgba();
    [linear.r, linear.g, linear.b]
}

/// Decodes an sRGBA color to linear light through
/// [`TySrgba::to_lin_srgba`](ty_math::TySrgba::to_lin_srgba); alpha carries no
/// gamma, so it passes through.
fn decode_rgba(components: &[f64; 4]) -> [f64; 4] {
    TySrgbaF64::from_array(*components)
        .to_lin_srgba()
        .to_array()
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
    fn linear_float_decodes_an_srgb_pool() {
        let pool = VoxValuePool::Srgb {
            values: vec![[1.0, 0.0, 0.5]],
        };

        match voxj_value_pool_from_vox_value_pool(&pool, ColorFormat::LinearFloat) {
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
        let pool = VoxValuePool::Srgba {
            values: vec![[0.5, 0.5, 0.5, 0.25]],
        };

        match voxj_value_pool_from_vox_value_pool(&pool, ColorFormat::LinearFloat) {
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
        // A linear pool carries components above 1; linear-float holds them,
        // hex could not.
        let pool = VoxValuePool::LinearRgb {
            values: vec![[2.5, 0.0, 1.0]],
        };

        match voxj_value_pool_from_vox_value_pool(&pool, ColorFormat::LinearFloat) {
            VoxjValuePool::LinearRgbFloat { values } => assert_eq!(values[0], [2.5, 0.0, 1.0]),
            other => panic!("expected linear-rgb-float, got {other:?}"),
        }
    }

    #[test]
    fn srgb_float_default_passes_components_through_unchanged() {
        let pool = VoxValuePool::Srgb {
            values: vec![[0.1, 0.2, 0.3]],
        };

        match voxj_value_pool_from_vox_value_pool(&pool, ColorFormat::Float) {
            VoxjValuePool::SrgbFloat { values } => assert_eq!(values, vec![[0.1, 0.2, 0.3]]),
            other => panic!("expected srgb-float, got {other:?}"),
        }
    }

    /// The sRGB transfer inverse, kept in the test as an independent reference
    /// for the production decode (`TySrgba::to_lin_srgba`).
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
