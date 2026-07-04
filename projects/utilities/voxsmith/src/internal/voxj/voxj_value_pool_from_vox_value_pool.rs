use crate::{ColorFormat, voxj_value_from_vox_value};
use std::fmt::Write;
use voxcore::{VoxBound, VoxValuePool};
use voxj::{VoxjBound, VoxjValuePool};

/// Converts a [`VoxValuePool`] into a [`VoxjValuePool`], kind by kind.
///
/// voxcore stores every color as float components, so `color_format` picks the
/// on-wire encoding for the two sRGB kinds: [`ColorFormat::Hex`] quantizes each
/// component to a `#RRGGBB` / `#RRGGBBAA` byte, [`ColorFormat::Float`] writes
/// the float components straight through. Linear colors have no hex form and
/// always serialize as float. Every other kind maps one to one, carrying its
/// values and, for `int`/`float`, its bounds across unchanged. `json` values
/// recurse through [`voxj_value_from_vox_value`].
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
        },

        VoxValuePool::Srgba { values } => match color_format {
            ColorFormat::Hex => VoxjValuePool::SrgbaHex {
                values: values.iter().map(encode_hex).collect(),
            },
            ColorFormat::Float => VoxjValuePool::SrgbaFloat {
                values: values.clone(),
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
        let byte = (component.clamp(0.0, 1.0) * 255.0).round() as u8;
        // Writing to a String is infallible.
        write!(hex, "{byte:02X}").unwrap();
    }
    hex
}
