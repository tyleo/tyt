use crate::{Check, Failures};
use voxj::{VoxjBound, VoxjMain, VoxjValuePool};

/// Every value pool is well-formed per spec rule 9: `values` is non-empty; each
/// int or float value lies within the pool's `min`/`max`; an int pool's numeric
/// bounds are integer-valued; `min <= max` when both are finite; hex colors
/// match their kind's pattern; and float color components lie in their color
/// space's range.
///
/// A value's JSON shape is guaranteed at parse by the typed pool model: a bound
/// is present only on `int` and `float`, `null` is rejected outside a `json`
/// pool, and a color array has exactly its kind's length. So this check covers
/// only the content rules parse cannot see: emptiness, numeric bounds, hex
/// text, and color-component ranges. The `json`, `bool`, and `string` kinds add
/// no content rule past non-emptiness.
pub fn check_value_pools(main: &VoxjMain, failures: &mut Failures) {
    for (index, pool) in main.runtime_state.value_pools.iter().enumerate() {
        if !failures.go() {
            return;
        }

        if pool.values_len() == 0 {
            failures.report(
                Check::ValuePools,
                format!("value pool {index} has no values"),
            );
            if !failures.go() {
                return;
            }
        }

        match pool {
            VoxjValuePool::Float { min, max, values } => {
                check_numeric(index, *min, *max, false, values.iter().copied(), failures);
            }
            VoxjValuePool::Int { min, max, values } => {
                let values = values.iter().map(|&value| value as f64);
                check_numeric(index, *min, *max, true, values, failures);
            }
            VoxjValuePool::SrgbHex { values } => check_hex(index, values, 6, failures),
            VoxjValuePool::SrgbaHex { values } => check_hex(index, values, 8, failures),
            VoxjValuePool::SrgbFloat { values } => check_colors(index, values, false, failures),
            VoxjValuePool::SrgbaFloat { values } => check_colors(index, values, false, failures),
            VoxjValuePool::LinearRgbFloat { values } => check_colors(index, values, true, failures),
            VoxjValuePool::LinearRgbaFloat { values } => {
                check_colors(index, values, true, failures)
            }
            VoxjValuePool::Json { .. }
            | VoxjValuePool::Bool { .. }
            | VoxjValuePool::String { .. } => {}
        }
    }
}

/// An int or float pool's bounds are well-formed and every value lies within
/// them. `integer` marks an int pool, whose numeric bounds must be
/// integer-valued; both kinds present their values as `f64`, so a large int
/// bound compares within `f64` precision, ample for the format's small ranges.
fn check_numeric(
    index: usize,
    min: VoxjBound,
    max: VoxjBound,
    integer: bool,
    values: impl Iterator<Item = f64>,
    failures: &mut Failures,
) {
    if integer {
        for (side, bound) in [("min", min), ("max", max)] {
            if let VoxjBound::Number(number) = bound
                && number.fract() != 0.0
            {
                failures.report(
                    Check::ValuePools,
                    format!(
                        "value pool {index} is an int pool but its {side} bound \
                         {number} is not integer-valued"
                    ),
                );
                if !failures.go() {
                    return;
                }
            }
        }
    }

    if let (VoxjBound::Number(low), VoxjBound::Number(high)) = (min, max)
        && low > high
    {
        failures.report(
            Check::ValuePools,
            format!("value pool {index} has min {low} greater than max {high}"),
        );
        if !failures.go() {
            return;
        }
    }

    for value in values {
        let below = matches!(min, VoxjBound::Number(low) if value < low);
        let above = matches!(max, VoxjBound::Number(high) if value > high);
        if below || above {
            failures.report(
                Check::ValuePools,
                format!(
                    "value pool {index} value {value} lies outside [{}, {}]",
                    describe(min),
                    describe(max)
                ),
            );
            if !failures.go() {
                return;
            }
        }
    }
}

/// Renders a bound for a message: the number, or `none` when unbounded.
fn describe(bound: VoxjBound) -> String {
    match bound {
        VoxjBound::Number(number) => number.to_string(),
        VoxjBound::None => "none".to_owned(),
    }
}

/// Every hex color is `#` followed by exactly `digits` uppercase hex characters.
fn check_hex(index: usize, values: &[String], digits: usize, failures: &mut Failures) {
    for (value_index, value) in values.iter().enumerate() {
        if !is_hex_color(value, digits) {
            failures.report(
                Check::ValuePools,
                format!(
                    "value pool {index} value {value_index} {value:?} is not \
                     {digits} uppercase hex digits after '#'"
                ),
            );
            if !failures.go() {
                return;
            }
        }
    }
}

/// `#` then exactly `digits` uppercase hex characters, matching `0-9A-F`.
fn is_hex_color(value: &str, digits: usize) -> bool {
    value.strip_prefix('#').is_some_and(|rest| {
        rest.len() == digits
            && rest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte))
    })
}

/// Every float color component lies in its space's range: sRGB in `[0, 1]`,
/// linear `>= 0` for HDR. The array length is fixed by the kind, so only the
/// range is open here.
fn check_colors<const N: usize>(
    index: usize,
    values: &[[f64; N]],
    linear: bool,
    failures: &mut Failures,
) {
    for (value_index, color) in values.iter().enumerate() {
        for (component, &channel) in color.iter().enumerate() {
            let in_range = if linear {
                channel >= 0.0
            } else {
                (0.0..=1.0).contains(&channel)
            };
            if !in_range {
                failures.report(
                    Check::ValuePools,
                    format!(
                        "value pool {index} color {value_index} component {component} \
                         is {channel}, outside {}",
                        if linear {
                            ">= 0 (linear)"
                        } else {
                            "[0, 1] (sRGB)"
                        }
                    ),
                );
                if !failures.go() {
                    return;
                }
            }
        }
    }
}
