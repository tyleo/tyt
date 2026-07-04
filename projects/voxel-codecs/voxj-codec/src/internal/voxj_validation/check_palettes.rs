use crate::{Check, Failures};
use std::collections::HashSet;
use voxj::{VoxjMain, VoxjValue};

/// Every palette is rectangular, declares each attribute key once, and stores
/// any `rgba` value as a `#RRGGBBAA` string.
pub fn check_palettes(main: &VoxjMain, failures: &mut Failures) {
    for (index, palette) in main.runtime_state.palettes.iter().enumerate() {
        if !failures.go() {
            return;
        }

        let mut seen = HashSet::with_capacity(palette.attributes.len());
        for attribute in &palette.attributes {
            if !seen.insert(attribute.as_str()) {
                failures.report(
                    Check::Palettes,
                    format!("palette {index} declares attribute key {attribute:?} more than once"),
                );
                if !failures.go() {
                    return;
                }
            }
        }

        for (cell, row) in palette.data.iter().enumerate() {
            if row.len() != palette.attributes.len() {
                failures.report(
                    Check::Palettes,
                    format!(
                        "palette {index} cell {cell} has {} values but the palette has {} attributes",
                        row.len(),
                        palette.attributes.len()
                    ),
                );
                if !failures.go() {
                    return;
                }
            }
        }

        if let Some(rgba) = palette.attributes.iter().position(|key| key == "rgba") {
            for (cell, row) in palette.data.iter().enumerate() {
                let Some(value) = row.get(rgba) else {
                    // Short row already reported above.
                    continue;
                };
                if !is_rgba(value) {
                    failures.report(
                        Check::Palettes,
                        format!(
                            "palette {index} cell {cell} rgba value {} is not #RRGGBBAA with uppercase hex",
                            describe_value(value)
                        ),
                    );
                    if !failures.go() {
                        return;
                    }
                }
            }
        }
    }
}

/// Whether `value` is a `#RRGGBBAA` string: a leading `#` then exactly eight
/// uppercase hex digits, matching `^#[0-9A-F]{8}$`.
fn is_rgba(value: &VoxjValue) -> bool {
    let VoxjValue::Text(text) = value else {
        return false;
    };
    let Some(hex) = text.strip_prefix('#') else {
        return false;
    };
    hex.len() == 8 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'A'..=b'F'))
}

/// A short description of a value for a failure message.
fn describe_value(value: &VoxjValue) -> String {
    match value {
        VoxjValue::Text(text) => format!("{text:?}"),
        VoxjValue::Number(number) => number.to_string(),
        VoxjValue::Bool(boolean) => boolean.to_string(),
        VoxjValue::Null => "null".to_owned(),
        VoxjValue::Array(_) => "an array".to_owned(),
        VoxjValue::Object(_) => "an object".to_owned(),
    }
}
