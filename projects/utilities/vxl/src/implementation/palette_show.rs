use crate::{AttributeType, ColorComponent, Format, PaletteShowFormat, Result, implementation};
use std::{
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use voxcore::{VoxPalette, VoxValue};

/// Loads the voxel file at `input` and prints one palette's selected attribute.
/// The palette is picked by `index`, the attribute by `attribute`, optionally
/// narrowed to one color `component`. `r#type` overrides the type otherwise
/// inferred from the stored values, `format` selects the rendering, and `json`
/// emits the attribute as JSON instead.
#[allow(clippy::too_many_arguments)]
pub fn palette_show(
    input: &Path,
    from: Option<Format>,
    index: usize,
    attribute: &str,
    component: Option<ColorComponent>,
    r#type: Option<AttributeType>,
    format: PaletteShowFormat,
    json: bool,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;
    let palette_count = state.palette_count();
    let Some((_, palette)) = state.iter_palettes().nth(index) else {
        return Err(IOError::new(
            ErrorKind::InvalidInput,
            format!(
                "palette index {index} is out of range; the document has {palette_count} palette(s)"
            ),
        )
        .into());
    };
    let output =
        render_palette_attribute(palette, index, attribute, component, r#type, format, json)?;
    implementation::write_stdout(output.as_bytes())
}

/// Renders one palette attribute to the output string, the pure core of
/// [`palette_show`] once the palette is in hand.
fn render_palette_attribute(
    palette: &VoxPalette,
    index: usize,
    attribute: &str,
    component: Option<ColorComponent>,
    type_override: Option<AttributeType>,
    format: PaletteShowFormat,
    json: bool,
) -> Result<String> {
    let Some(attribute_id) = palette
        .iter_attributes()
        .find(|(_, name)| *name == attribute)
        .map(|(id, _)| id)
    else {
        return Err(IOError::new(
            ErrorKind::InvalidInput,
            format!(
                "palette {index} has no attribute `{attribute}`; available attributes: {}",
                available_keys(palette)
            ),
        )
        .into());
    };

    // A retained cell holds a value for every attribute, so the lookup with
    // this palette's own ids always resolves.
    let values: Vec<&VoxValue> = palette
        .iter_cells()
        .map(|cell| {
            palette
                .cell_value(cell, attribute_id)
                .expect("a cell holds a value for every attribute")
        })
        .collect();

    let resolved = type_override.unwrap_or_else(|| infer_type(&values));
    if let Some(component) = component
        && resolved == AttributeType::Scalar
    {
        return Err(IOError::new(
            ErrorKind::InvalidInput,
            format!(
                "attribute `{attribute}` is a scalar and has no `.{}` color component",
                component_letter(component)
            ),
        )
        .into());
    }

    let samples: Vec<Sample> = values
        .iter()
        .map(|&value| sample(value, resolved, component))
        .collect();
    let output = if json {
        render_json(index, attribute, component, &samples)
    } else {
        render_text(&samples, format)
    };
    Ok(output)
}

/// One rendered cell value: a whole color, a scalar in `0..1` (a scalar
/// attribute or an extracted color channel), or a raw fallback for a value that
/// is neither.
enum Sample {
    Color(Rgba),
    Scalar(f64),
    Raw(String),
}

/// A straight-alpha color decoded from a `#RRGGBBAA` string.
struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

/// The sample for one stored value under the resolved `r#type` and optional
/// `component`: a color decodes to a swatch or, with a component, to that
/// channel as a scalar; a scalar reads its number; anything else falls back to
/// its raw text.
fn sample(value: &VoxValue, r#type: AttributeType, component: Option<ColorComponent>) -> Sample {
    match r#type {
        AttributeType::Color => match value {
            VoxValue::Text(text) => match parse_rgba(text) {
                Some(rgba) => match component {
                    Some(component) => Sample::Scalar(component_value(&rgba, component)),
                    None => Sample::Color(rgba),
                },
                None => Sample::Raw(value_to_string(value)),
            },
            _ => Sample::Raw(value_to_string(value)),
        },
        AttributeType::Scalar => match value {
            VoxValue::Number(number) => Sample::Scalar(*number),
            _ => Sample::Raw(value_to_string(value)),
        },
    }
}

/// The type inferred from the first concrete value: a string is a color, a
/// number a scalar, defaulting to scalar when no value names a type.
fn infer_type(values: &[&VoxValue]) -> AttributeType {
    for value in values {
        match value {
            VoxValue::Text(_) => return AttributeType::Color,
            VoxValue::Number(_) => return AttributeType::Scalar,
            _ => {}
        }
    }
    AttributeType::Scalar
}

/// Decodes a `#RRGGBBAA` string, or `None` when it is not eight hex digits with
/// a leading `#`.
fn parse_rgba(text: &str) -> Option<Rgba> {
    let hex = text.strip_prefix('#')?;
    if hex.len() != 8 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some(Rgba {
        r: u8::from_str_radix(&hex[0..2], 16).ok()?,
        g: u8::from_str_radix(&hex[2..4], 16).ok()?,
        b: u8::from_str_radix(&hex[4..6], 16).ok()?,
        a: u8::from_str_radix(&hex[6..8], 16).ok()?,
    })
}

/// One color component as a `0..1` scalar, the same `byte / 255` the material
/// packing reads.
fn component_value(rgba: &Rgba, component: ColorComponent) -> f64 {
    let byte = match component {
        ColorComponent::R => rgba.r,
        ColorComponent::G => rgba.g,
        ColorComponent::B => rgba.b,
        ColorComponent::A => rgba.a,
    };
    byte as f64 / 255.0
}

/// The canonical uppercase `#RRGGBBAA` for a color.
fn hex(rgba: &Rgba) -> String {
    format!("#{:02X}{:02X}{:02X}{:02X}", rgba.r, rgba.g, rgba.b, rgba.a)
}

/// The lowercase letter naming a color component.
fn component_letter(component: ColorComponent) -> char {
    match component {
        ColorComponent::R => 'r',
        ColorComponent::G => 'g',
        ColorComponent::B => 'b',
        ColorComponent::A => 'a',
    }
}

/// The `--attribute` form, the key plus any `.component`, for messages and JSON.
fn attribute_label(attribute: &str, component: Option<ColorComponent>) -> String {
    match component {
        Some(component) => format!("{attribute}.{}", component_letter(component)),
        None => attribute.to_string(),
    }
}

/// The palette's attribute keys joined for a not-found message.
fn available_keys(palette: &VoxPalette) -> String {
    palette
        .iter_attributes()
        .map(|(_, name)| name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// A `VoxValue` as plain text, the raw fallback for a value that is neither a
/// color nor a number. Arrays and objects, which no recommended attribute uses,
/// render as `null`.
fn value_to_string(value: &VoxValue) -> String {
    match value {
        VoxValue::Number(number) => format!("{number}"),
        VoxValue::Text(text) => text.clone(),
        VoxValue::Bool(boolean) => format!("{boolean}"),
        VoxValue::Null | VoxValue::Array(_) | VoxValue::Object(_) => "null".to_string(),
    }
}

/// Renders the samples to lines, one per cell, in the chosen `format`.
fn render_text(samples: &[Sample], format: PaletteShowFormat) -> String {
    let mut output = String::new();
    for sample in samples {
        output.push_str(&render_sample(sample, format));
        output.push('\n');
    }
    output
}

/// One sample's line: `auto` swatches whole colors beside their hex but prints
/// scalars and channels as bare numbers, `swatch` prints a colored or grayscale
/// swatch alone, `swatch-value` follows that swatch with its value text, and
/// `value` prints the raw value with no swatch. A raw fallback, which has no
/// swatch, prints its text under every format.
fn render_sample(sample: &Sample, format: PaletteShowFormat) -> String {
    match (format, sample) {
        (PaletteShowFormat::Auto, Sample::Color(rgba)) => {
            format!("{} {}", color_swatch(rgba), hex(rgba))
        }
        (PaletteShowFormat::Auto, Sample::Scalar(value)) => format!("{value}"),
        (PaletteShowFormat::Auto, Sample::Raw(text)) => text.clone(),

        (PaletteShowFormat::Swatch, Sample::Color(rgba)) => color_swatch(rgba),
        (PaletteShowFormat::Swatch, Sample::Scalar(value)) => gray_swatch(*value),
        (PaletteShowFormat::Swatch, Sample::Raw(text)) => text.clone(),

        (PaletteShowFormat::SwatchValue, Sample::Color(rgba)) => {
            format!("{} {}", color_swatch(rgba), hex(rgba))
        }
        (PaletteShowFormat::SwatchValue, Sample::Scalar(value)) => {
            format!("{} {value}", gray_swatch(*value))
        }
        (PaletteShowFormat::SwatchValue, Sample::Raw(text)) => text.clone(),

        (PaletteShowFormat::Value, Sample::Color(rgba)) => hex(rgba),
        (PaletteShowFormat::Value, Sample::Scalar(value)) => format!("{value}"),
        (PaletteShowFormat::Value, Sample::Raw(text)) => text.clone(),
    }
}

/// A two-cell truecolor swatch of a color's rgb.
fn color_swatch(rgba: &Rgba) -> String {
    format!("\x1b[48;2;{};{};{}m  \x1b[0m", rgba.r, rgba.g, rgba.b)
}

/// A two-cell truecolor grayscale swatch of a `0..1` value.
fn gray_swatch(value: f64) -> String {
    let level = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("\x1b[48;2;{level};{level};{level}m  \x1b[0m")
}

/// The selected attribute as a one-line JSON object: its palette `index`, the
/// `--attribute` label, and the rendered `values`. The values carry their own
/// JSON type, a hex string for a color and a number otherwise, so no separate
/// type field is reported.
fn render_json(
    index: usize,
    attribute: &str,
    component: Option<ColorComponent>,
    samples: &[Sample],
) -> String {
    let label = attribute_label(attribute, component);
    let values = samples
        .iter()
        .map(sample_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"index\":{index},\"attribute\":{},\"values\":[{values}]}}\n",
        json_string(&label)
    )
}

/// One sample as a JSON value: a quoted hex string for a color, a bare number
/// for a scalar, a quoted string for a raw fallback.
fn sample_json(sample: &Sample) -> String {
    match sample {
        Sample::Color(rgba) => json_string(&hex(rgba)),
        Sample::Scalar(value) => format!("{value}"),
        Sample::Raw(text) => json_string(text),
    }
}

/// `text` as a quoted, escaped JSON string.
fn json_string(text: &str) -> String {
    let mut output = String::with_capacity(text.len() + 2);
    output.push('"');
    for character in text.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                output.push_str(&format!("\\u{:04x}", control as u32))
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

#[cfg(test)]
mod tests {
    use crate::{
        AttributeType, ColorComponent, PaletteShowFormat,
        implementation::palette_show::render_palette_attribute,
    };
    use voxcore::{VoxPalette, VoxValue};

    /// A palette with `rgba` and `metallic` attributes and two cells.
    fn sample_palette() -> VoxPalette {
        let mut palette = VoxPalette::default();
        palette.add_attribute("rgba".to_owned());
        palette.add_attribute("metallic".to_owned());
        palette
            .add_cell(vec![
                VoxValue::Text("#FF0000FF".to_owned()),
                VoxValue::Number(1.0),
            ])
            .unwrap();
        palette
            .add_cell(vec![
                VoxValue::Text("#00FF0080".to_owned()),
                VoxValue::Number(0.2),
            ])
            .unwrap();
        palette
    }

    #[test]
    fn value_format_prints_canonical_hex_for_a_color() {
        let palette = sample_palette();
        let output = render_palette_attribute(
            &palette,
            0,
            "rgba",
            None,
            None,
            PaletteShowFormat::Value,
            false,
        )
        .unwrap();
        assert_eq!(output, "#FF0000FF\n#00FF0080\n");
    }

    #[test]
    fn value_format_prints_bare_numbers_for_a_scalar() {
        let palette = sample_palette();
        let output = render_palette_attribute(
            &palette,
            0,
            "metallic",
            None,
            None,
            PaletteShowFormat::Value,
            false,
        )
        .unwrap();
        assert_eq!(output, "1\n0.2\n");
    }

    #[test]
    fn extracts_a_color_component_as_a_scalar() {
        let palette = sample_palette();
        let output = render_palette_attribute(
            &palette,
            0,
            "rgba",
            Some(ColorComponent::A),
            None,
            PaletteShowFormat::Value,
            false,
        )
        .unwrap();
        // Alpha bytes FF and 80 over 255.
        assert_eq!(output, "1\n0.5019607843137255\n");
    }

    #[test]
    fn swatch_format_prints_swatches_alone() {
        let palette = sample_palette();
        let output = render_palette_attribute(
            &palette,
            0,
            "rgba",
            None,
            None,
            PaletteShowFormat::Swatch,
            false,
        )
        .unwrap();
        assert_eq!(
            output,
            "\x1b[48;2;255;0;0m  \x1b[0m\n\x1b[48;2;0;255;0m  \x1b[0m\n"
        );
    }

    #[test]
    fn swatch_value_format_pairs_each_swatch_with_its_value() {
        let palette = sample_palette();
        let output = render_palette_attribute(
            &palette,
            0,
            "rgba",
            None,
            None,
            PaletteShowFormat::SwatchValue,
            false,
        )
        .unwrap();
        assert_eq!(
            output,
            "\x1b[48;2;255;0;0m  \x1b[0m #FF0000FF\n\x1b[48;2;0;255;0m  \x1b[0m #00FF0080\n"
        );
    }

    #[test]
    fn auto_swatches_colors_but_prints_scalars_and_channels_as_numbers() {
        let palette = sample_palette();
        // A scalar attribute is a bare number.
        let scalar = render_palette_attribute(
            &palette,
            0,
            "metallic",
            None,
            None,
            PaletteShowFormat::Auto,
            false,
        )
        .unwrap();
        assert_eq!(scalar, "1\n0.2\n");
        // An extracted channel is also a bare number.
        let channel = render_palette_attribute(
            &palette,
            0,
            "rgba",
            Some(ColorComponent::R),
            None,
            PaletteShowFormat::Auto,
            false,
        )
        .unwrap();
        assert_eq!(channel, "1\n0\n");
        // A whole color is a swatch beside its hex.
        let color = render_palette_attribute(
            &palette,
            0,
            "rgba",
            None,
            None,
            PaletteShowFormat::Auto,
            false,
        )
        .unwrap();
        assert_eq!(
            color,
            "\x1b[48;2;255;0;0m  \x1b[0m #FF0000FF\n\x1b[48;2;0;255;0m  \x1b[0m #00FF0080\n"
        );
    }

    #[test]
    fn rejects_a_component_on_a_scalar() {
        let palette = sample_palette();
        let error = render_palette_attribute(
            &palette,
            0,
            "metallic",
            Some(ColorComponent::R),
            None,
            PaletteShowFormat::SwatchValue,
            false,
        );
        assert!(error.is_err());
    }

    #[test]
    fn rejects_an_unknown_attribute() {
        let palette = sample_palette();
        let error = render_palette_attribute(
            &palette,
            0,
            "missing",
            None,
            None,
            PaletteShowFormat::SwatchValue,
            false,
        );
        assert!(error.is_err());
    }

    #[test]
    fn json_reports_index_label_and_values() {
        let palette = sample_palette();
        let output = render_palette_attribute(
            &palette,
            0,
            "rgba",
            None,
            None,
            PaletteShowFormat::SwatchValue,
            true,
        )
        .unwrap();
        assert_eq!(
            output,
            "{\"index\":0,\"attribute\":\"rgba\",\"values\":[\"#FF0000FF\",\"#00FF0080\"]}\n"
        );
    }

    #[test]
    fn json_emits_an_extracted_channel_as_a_number() {
        let palette = sample_palette();
        let output = render_palette_attribute(
            &palette,
            0,
            "rgba",
            Some(ColorComponent::A),
            None,
            PaletteShowFormat::SwatchValue,
            true,
        )
        .unwrap();
        assert_eq!(
            output,
            "{\"index\":0,\"attribute\":\"rgba.a\",\"values\":[1,0.5019607843137255]}\n"
        );
    }

    #[test]
    fn type_override_reads_a_number_attribute_as_a_color_fallback() {
        // Forcing `color` on a numeric attribute cannot decode a hex string, so
        // each value falls back to its raw text.
        let palette = sample_palette();
        let output = render_palette_attribute(
            &palette,
            0,
            "metallic",
            None,
            Some(AttributeType::Color),
            PaletteShowFormat::Value,
            false,
        )
        .unwrap();
        assert_eq!(output, "1\n0.2\n");
    }
}
