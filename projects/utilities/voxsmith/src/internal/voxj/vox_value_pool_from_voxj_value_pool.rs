use crate::{Error, Result, vox_value_from_voxj_value};
use voxcore::{VoxBound, VoxValuePool};
use voxj::{VoxjBound, VoxjValuePool};

/// Converts a [`VoxjValuePool`] into a [`VoxValuePool`], kind by kind.
///
/// The wire's six color kinds collapse onto voxcore's four: the hex and float
/// encodings of a color space both canonicalize to that space's single voxcore
/// color kind, with hex strings decoded to float components in `[0, 1]` by
/// dividing each byte by 255. Every other kind maps one to one, carrying its
/// values and, for `int`/`float`, its bounds across unchanged. `json` values
/// recurse through [`vox_value_from_voxj_value`].
///
/// Errors only on a malformed hex color string; range and bound checks are left
/// to [`VoxMain::validate`](voxcore::VoxMain::validate).
pub fn vox_value_pool_from_voxj_value_pool(pool: &VoxjValuePool) -> Result<VoxValuePool> {
    Ok(match pool {
        VoxjValuePool::Json { values } => VoxValuePool::json(
            values
                .iter()
                .map(vox_value_from_voxj_value)
                .collect::<Result<_>>()?,
        ),

        VoxjValuePool::Bool { values } => VoxValuePool::boolean(values.clone()),

        VoxjValuePool::Float { min, max, values } => {
            VoxValuePool::float(vox_bound(*min), vox_bound(*max), values.clone())
        }

        VoxjValuePool::Int { min, max, values } => {
            VoxValuePool::int(vox_bound(*min), vox_bound(*max), values.clone())
        }

        VoxjValuePool::String { values } => VoxValuePool::string(values.clone()),

        VoxjValuePool::SrgbFloat { values } => VoxValuePool::srgb(values.clone()),

        VoxjValuePool::SrgbHex { values } => VoxValuePool::srgb(decode_hex_values(values)?),

        VoxjValuePool::SrgbaFloat { values } => VoxValuePool::srgba(values.clone()),

        VoxjValuePool::SrgbaHex { values } => VoxValuePool::srgba(decode_hex_values(values)?),

        VoxjValuePool::LinearRgbFloat { values } => VoxValuePool::linear_rgb(values.clone()),

        VoxjValuePool::LinearRgbaFloat { values } => VoxValuePool::linear_rgba(values.clone()),
    })
}

/// Maps a wire bound to its voxcore form; the two are the same number-or-none
/// shape.
fn vox_bound(bound: VoxjBound) -> VoxBound {
    match bound {
        VoxjBound::Number(number) => VoxBound::Number(number),
        VoxjBound::None => VoxBound::None,
    }
}

/// Decodes each `#RRGGBB` (`N = 3`) or `#RRGGBBAA` (`N = 4`) string to float
/// components in `[0, 1]`.
fn decode_hex_values<const N: usize>(values: &[String]) -> Result<Vec<[f64; N]>> {
    values.iter().map(|hex| decode_hex(hex)).collect()
}

/// Decodes one `#` plus exactly `2 * N` hex digits into `N` float components,
/// each the byte value divided by 255. Errors on any other shape.
fn decode_hex<const N: usize>(hex: &str) -> Result<[f64; N]> {
    let digits = hex
        .strip_prefix('#')
        .filter(|digits| digits.len() == N * 2 && digits.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| {
            Error::invalid(format!(
                "color \"{hex}\" is not a # followed by {} hex digits",
                N * 2
            ))
        })?;
    let mut components = [0.0; N];
    for (index, component) in components.iter_mut().enumerate() {
        // `digits` is all ASCII, so this two-byte slice is a valid str of two
        // hex digits and the parse never fails.
        let byte = u8::from_str_radix(&digits[index * 2..index * 2 + 2], 16).unwrap();
        *component = byte as f64 / 255.0;
    }
    Ok(components)
}
