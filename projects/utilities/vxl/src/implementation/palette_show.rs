use crate::{
    ColorComponent, Format, Result, Width,
    commands::{PaletteRef, PaletteShowFormat, PaletteShowLayout, PropertyRef, PropertySelector},
    implementation,
};
use branded_id::U32Id;
use serde_json::{Map, Value, json};
use std::{
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use ty_math::{TyFloatExt, TyLinSrgbaF64, TySrgbaF64};
use voxcore::{BVoxPoolValue, VoxMain, VoxPalette, VoxPropertyId, VoxValue, VoxValuePool};

/// Loads the voxel file at `input` and prints the value collections named by
/// `selectors`. Each selector resolves to one or more collections, a
/// property's values down a palette; `layout` arranges them and chooses the
/// serialization, and `width` wraps the `row` layouts.
pub fn palette_show(
    input: &Path,
    from: Option<Format>,
    selectors: &[PropertySelector],
    layout: PaletteShowLayout,
    width: Width,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;
    let collections = resolve_collections(&state, selectors)?;
    let output = render(&collections, layout, resolve_width(width));
    implementation::write_stdout(output.as_bytes())
}

/// The column budget a `Width` resolves to, or `None` for no wrapping: a fixed
/// count, the terminal width, or unlimited. A `Terminal` width with no terminal
/// on stdout, as when the output is piped, also resolves to no wrapping.
fn resolve_width(width: Width) -> Option<usize> {
    match width {
        Width::Unlimited => None,
        Width::Columns(columns) => Some(columns),
        Width::Terminal => terminal_columns(),
    }
}

/// The terminal's column count, read from stdout, or `None` when stdout is not
/// a terminal.
#[cfg(unix)]
fn terminal_columns() -> Option<usize> {
    use libc::{STDOUT_FILENO, TIOCGWINSZ, ioctl, winsize};
    use std::mem;
    // Safety: winsize is plain data; ioctl fills it for the stdout fd, and the
    // result is read only when the call reports success.
    unsafe {
        let mut size: winsize = mem::zeroed();
        if ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size) == 0 && size.ws_col > 0 {
            Some(size.ws_col as usize)
        } else {
            None
        }
    }
}

/// No terminal-width detection off unix; the `row` layouts do not wrap.
#[cfg(not(unix))]
fn terminal_columns() -> Option<usize> {
    None
}

/// One resolved value collection: a property's values down one palette,
/// labeled by its palette index and property, with the format that renders it.
struct Collection {
    /// The resolved palette index, even when the selector used `*`.
    palette: usize,
    /// The property key, without any color component.
    key: String,
    /// The color component read from the property, when one was given.
    component: Option<ColorComponent>,
    /// Whether the key names a scalar property, whose one sample is the
    /// palette-wide pinned value rather than material 0's.
    scalar: bool,
    /// How each value renders.
    format: PaletteShowFormat,
    /// One sample per palette material in material order, or the one pinned
    /// sample of a scalar property.
    samples: Vec<Sample>,
}

impl Collection {
    /// The `{palette}."{key}"` header for the text layouts, with the color
    /// component appended after the closing quote when one was read and a
    /// ` (scalar)` marker on a scalar property.
    fn header(&self) -> String {
        format!(
            "{}.{}{}{}",
            self.palette,
            implementation::quote_name(&self.key),
            self.component_suffix(),
            if self.scalar { " (scalar)" } else { "" }
        )
    }

    /// The raw `{key}` or `{key}.{component}` property label for the JSON
    /// records, where the serialization already quotes it.
    fn property(&self) -> String {
        format!("{}{}", self.key, self.component_suffix())
    }

    /// The `.{component}` suffix, or empty when no component was read.
    fn component_suffix(&self) -> String {
        match self.component {
            Some(component) => format!(".{}", component_letter(component)),
            None => String::new(),
        }
    }
}

/// One rendered material value: its display text, JSON form, and the swatch it
/// carries. Sampling reads a value through its bound pool's kind and
/// precomputes all three so every layout renders uniformly.
struct Sample {
    /// The value as text: a hex color, a color component, a number, a bool, a
    /// string, or a JSON value.
    text: String,
    /// The value in its native JSON type.
    json: Value,
    /// The swatch this value renders, or none for a value with no swatch.
    swatch: Swatch,
}

/// The swatch a [`Sample`] renders: a true-color block for a whole color, a
/// grayscale block for a scalar or color component, or none for a value with no
/// meaningful swatch, such as a bool or string.
enum Swatch {
    Color([u8; 3]),
    Gray(u8),
    None,
}

/// How a bound pool's kind renders in `palette show`: a color, in sRGB or
/// linear space and with three or four components; a plain number; or any other
/// value shown as text with no swatch.
#[derive(Clone, Copy)]
enum Kind {
    /// A color pool: `srgb` distinguishes sRGB from linear space, `components`
    /// is 3 or 4.
    Color { srgb: bool, components: usize },
    /// A `float` or `int` pool.
    Number,
    /// A `bool`, `string`, or `json` pool.
    Other,
}

/// Resolves the selectors against the document's palettes into collections in
/// render order: selector order, then palette order, then property order. A
/// `*` palette or `*` property expands to one collection per match; a named
/// palette or property that is absent is an error, while a `*` palette quietly
/// skips a palette that lacks a named property.
fn resolve_collections(state: &VoxMain, selectors: &[PropertySelector]) -> Result<Vec<Collection>> {
    let palettes: Vec<&VoxPalette> = state.iter_palettes().map(|(_, palette)| palette).collect();
    let mut collections = Vec::new();
    for selector in selectors {
        match selector.palette {
            PaletteRef::All => {
                for (index, palette) in palettes.iter().enumerate() {
                    expand_property(
                        state,
                        index,
                        palette,
                        &selector.property,
                        selector.format,
                        true,
                        &mut collections,
                    )?;
                }
            }
            PaletteRef::Index(index) => {
                let palette = palettes.get(index).ok_or_else(|| {
                    IOError::new(
                        ErrorKind::InvalidInput,
                        format!(
                            "palette index {index} is out of range; the document has {} palette(s)",
                            palettes.len()
                        ),
                    )
                })?;
                expand_property(
                    state,
                    index,
                    palette,
                    &selector.property,
                    selector.format,
                    false,
                    &mut collections,
                )?;
            }
        }
    }
    Ok(collections)
}

/// Expands one selector's property against one palette, pushing the resulting
/// collections. A `palette_is_wild` palette, one from a `*`, skips a named
/// property it lacks instead of erroring.
fn expand_property(
    state: &VoxMain,
    index: usize,
    palette: &VoxPalette,
    property: &PropertyRef,
    format: PaletteShowFormat,
    palette_is_wild: bool,
    collections: &mut Vec<Collection>,
) -> Result<()> {
    match property {
        PropertyRef::All => {
            for name in implementation::property_names(palette) {
                collections.push(build_collection(state, index, palette, name, None, format)?);
            }
        }
        PropertyRef::Key { key, component } => {
            if palette.property_by_name(key).is_none() {
                if palette_is_wild {
                    return Ok(());
                }
                return Err(IOError::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "palette {index} has no property `{key}`; available properties: {}",
                        available_keys(palette)
                    ),
                )
                .into());
            }
            collections.push(build_collection(
                state, index, palette, key, *component, format,
            )?);
        }
    }
    Ok(())
}

/// Builds one collection from a present property: classifies the bound pool by
/// kind, rejects a color component on a non-color and `.a` on a three-component
/// color, then samples the property's values.
fn build_collection(
    state: &VoxMain,
    index: usize,
    palette: &VoxPalette,
    key: &str,
    component: Option<ColorComponent>,
    format: PaletteShowFormat,
) -> Result<Collection> {
    let property_id = palette
        .property_by_name(key)
        .expect("caller verified the property is present");
    let pool_id = match property_id {
        VoxPropertyId::Array(id) => {
            palette
                .array_property(id)
                .expect("an array-property id from this palette resolves")
                .pool
        }
        VoxPropertyId::Scalar(id) => {
            palette
                .scalar_property(id)
                .expect("a scalar-property id from this palette resolves")
                .pool
        }
    };
    let pool = state
        .value_pool(pool_id)
        .expect("a property references a pool the state holds");
    let kind = classify(pool);

    // A color component names one channel: it applies only to a color, and `.a`
    // only to a four-component color.
    if let Some(component) = component {
        match kind {
            Kind::Color { components, .. } => {
                if component == ColorComponent::A && components < 4 {
                    return Err(IOError::new(
                        ErrorKind::InvalidInput,
                        format!(
                            "property `{key}` is a three-component color and has no `.a` component"
                        ),
                    )
                    .into());
                }
            }
            Kind::Number | Kind::Other => {
                return Err(IOError::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "property `{key}` is not a color and has no `.{}` component",
                        component_letter(component)
                    ),
                )
                .into());
            }
        }
    }

    // A material holds one value id per array property, so the lookup with
    // this palette's own property id always resolves.
    let samples = match property_id {
        VoxPropertyId::Array(id) => palette
            .iter_materials()
            .map(|material| {
                let value_id = palette
                    .value_id(material, id)
                    .expect("a material holds a value for every array property");
                sample(pool, value_id, kind, component)
            })
            .collect(),
        VoxPropertyId::Scalar(id) => {
            let property = palette
                .scalar_property(id)
                .expect("a scalar-property id from this palette resolves");
            vec![sample(pool, property.value_id, kind, component)]
        }
    };

    Ok(Collection {
        palette: index,
        key: key.to_string(),
        component,
        scalar: matches!(property_id, VoxPropertyId::Scalar(_)),
        format,
        samples,
    })
}

/// How a pool's kind renders: a color with its space and component count, a
/// number, or any other value.
fn classify(pool: &VoxValuePool) -> Kind {
    match pool {
        VoxValuePool::Srgb { .. } => Kind::Color {
            srgb: true,
            components: 3,
        },
        VoxValuePool::Srgba { .. } => Kind::Color {
            srgb: true,
            components: 4,
        },
        VoxValuePool::LinearRgb { .. } => Kind::Color {
            srgb: false,
            components: 3,
        },
        VoxValuePool::LinearRgba { .. } => Kind::Color {
            srgb: false,
            components: 4,
        },
        VoxValuePool::Float { .. } | VoxValuePool::Int { .. } => Kind::Number,
        VoxValuePool::Bool { .. } | VoxValuePool::String { .. } | VoxValuePool::Json { .. } => {
            Kind::Other
        }
    }
}

/// The sample for the value at `value_id` under its pool `kind` and an
/// optional color `component`.
fn sample(
    pool: &VoxValuePool,
    value_id: U32Id<BVoxPoolValue>,
    kind: Kind,
    component: Option<ColorComponent>,
) -> Sample {
    match kind {
        Kind::Color { srgb, .. } => sample_color(pool, value_id, srgb, component),
        Kind::Number => sample_number(pool, value_id),
        Kind::Other => sample_other(pool, value_id),
    }
}

/// The sample for a color value: a whole color as a swatch plus a hex string
/// (sRGB) or float components (linear), or, with a `component`, one channel as a
/// byte (sRGB) or float (linear) with a grayscale swatch.
fn sample_color(
    pool: &VoxValuePool,
    value_id: U32Id<BVoxPoolValue>,
    srgb: bool,
    component: Option<ColorComponent>,
) -> Sample {
    let bytes = color_bytes(pool, value_id);
    match component {
        Some(component) => {
            let channel = component_index(component);
            if srgb {
                let byte = bytes[channel];
                Sample {
                    text: byte.to_string(),
                    json: json!(byte),
                    swatch: Swatch::Gray(byte),
                }
            } else {
                let value = color_floats(pool, value_id)[channel];
                Sample {
                    text: format_number(value),
                    json: number_json(value),
                    swatch: Swatch::Gray(scalar_level(value)),
                }
            }
        }
        None => {
            let swatch = Swatch::Color([bytes[0], bytes[1], bytes[2]]);
            if srgb {
                let text = srgb_hex(&bytes, alpha_component(pool));
                Sample {
                    json: Value::String(text.clone()),
                    text,
                    swatch,
                }
            } else {
                let floats = color_floats(pool, value_id);
                let text = floats
                    .iter()
                    .map(|value| format_number(*value))
                    .collect::<Vec<_>>()
                    .join(" ");
                let json = Value::Array(floats.iter().map(|value| number_json(*value)).collect());
                Sample { text, json, swatch }
            }
        }
    }
}

/// The sample for a `float` or `int` value: its number, with a grayscale swatch
/// mapping its `0..1` range onto `0..255`.
fn sample_number(pool: &VoxValuePool, value_id: U32Id<BVoxPoolValue>) -> Sample {
    let index = value_id.to_usize_id();
    let value = match pool {
        VoxValuePool::Float { values, .. } => values[index],
        VoxValuePool::Int { values, .. } => values[index] as f64,
        // classify() routes only Float and Int here.
        _ => 0.0,
    };
    Sample {
        text: format_number(value),
        json: number_json(value),
        swatch: Swatch::Gray(scalar_level(value)),
    }
}

/// The sample for a `bool`, `string`, or `json` value: its text and native JSON
/// with no swatch.
fn sample_other(pool: &VoxValuePool, value_id: U32Id<BVoxPoolValue>) -> Sample {
    let index = value_id.to_usize_id();
    match pool {
        VoxValuePool::Bool { values } => {
            let value = values[index];
            Sample {
                text: value.to_string(),
                json: Value::Bool(value),
                swatch: Swatch::None,
            }
        }
        VoxValuePool::String { values } => {
            let value = values[index].clone();
            Sample {
                json: Value::String(value.clone()),
                text: value,
                swatch: Swatch::None,
            }
        }
        VoxValuePool::Json { values } => {
            let json = vox_value_to_json(&values[index]);
            Sample {
                text: json_text(&json),
                json,
                swatch: Swatch::None,
            }
        }
        // classify() routes only Bool, String, and Json here.
        _ => Sample {
            text: "null".to_string(),
            json: Value::Null,
            swatch: Swatch::None,
        },
    }
}

/// The sRGB `[r, g, b, a]` bytes for the color at `value_id`, mirroring the
/// shared pool color decode: sRGB components map straight to bytes, linear
/// components re-encode to sRGB, and a three-component color takes opaque
/// alpha.
fn color_bytes(pool: &VoxValuePool, value_id: U32Id<BVoxPoolValue>) -> [u8; 4] {
    let index = value_id.to_usize_id();
    match pool {
        VoxValuePool::Srgb { values } => {
            let [r, g, b] = values[index];
            TySrgbaF64::new(r, g, b, 1.0).to_u8().to_array()
        }
        VoxValuePool::Srgba { values } => {
            let [r, g, b, a] = values[index];
            TySrgbaF64::new(r, g, b, a).to_u8().to_array()
        }
        VoxValuePool::LinearRgb { values } => {
            let [r, g, b] = values[index];
            TyLinSrgbaF64::new(r, g, b, 1.0)
                .to_srgba()
                .to_u8()
                .to_array()
        }
        VoxValuePool::LinearRgba { values } => {
            let [r, g, b, a] = values[index];
            TyLinSrgbaF64::new(r, g, b, a).to_srgba().to_u8().to_array()
        }
        // classify() routes only color kinds here.
        _ => [0, 0, 0, 0],
    }
}

/// The natural-range float components of the color at `value_id`, three or
/// four long by kind.
fn color_floats(pool: &VoxValuePool, value_id: U32Id<BVoxPoolValue>) -> Vec<f64> {
    let index = value_id.to_usize_id();
    match pool {
        VoxValuePool::Srgb { values } | VoxValuePool::LinearRgb { values } => {
            values[index].to_vec()
        }
        VoxValuePool::Srgba { values } | VoxValuePool::LinearRgba { values } => {
            values[index].to_vec()
        }
        // classify() routes only color kinds here.
        _ => Vec::new(),
    }
}

/// Whether a color pool carries an alpha component.
fn alpha_component(pool: &VoxValuePool) -> bool {
    matches!(
        pool,
        VoxValuePool::Srgba { .. } | VoxValuePool::LinearRgba { .. }
    )
}

/// The canonical uppercase hex for a color, `#RRGGBBAA` with alpha or `#RRGGBB`
/// without.
fn srgb_hex(bytes: &[u8; 4], alpha: bool) -> String {
    if alpha {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            bytes[0], bytes[1], bytes[2], bytes[3]
        )
    } else {
        format!("#{:02X}{:02X}{:02X}", bytes[0], bytes[1], bytes[2])
    }
}

/// The `0..3` index of a color component into an `[r, g, b, a]` array.
fn component_index(component: ColorComponent) -> usize {
    match component {
        ColorComponent::R => 0,
        ColorComponent::G => 1,
        ColorComponent::B => 2,
        ColorComponent::A => 3,
    }
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

/// A number as text, an integral value with no fractional part, matching the
/// number layouts.
fn format_number(value: f64) -> String {
    format!("{value}")
}

/// A number as JSON: an integer when it is integral and fits `i64`, else a
/// float, so it reads as it does in the text layouts.
fn number_json(value: f64) -> Value {
    if value.fract() == 0.0 && value.abs() < i64::MAX as f64 {
        json!(value as i64)
    } else {
        json!(value)
    }
}

/// A [`VoxValue`] from a `json` pool as a [`serde_json::Value`].
fn vox_value_to_json(value: &VoxValue) -> Value {
    match value {
        VoxValue::Bool(boolean) => Value::Bool(*boolean),
        VoxValue::Number(number) => number_json(*number),
        VoxValue::Text(text) => Value::String(text.clone()),
        VoxValue::Null => Value::Null,
        VoxValue::Array(items) => Value::Array(items.iter().map(vox_value_to_json).collect()),
        VoxValue::Object(map) => Value::Object(
            map.0
                .iter()
                .map(|(key, value)| (key.clone(), vox_value_to_json(value)))
                .collect(),
        ),
    }
}

/// A JSON value as its compact text, the string a `json`-pool value renders to.
fn json_text(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
}

/// The palette's property keys joined for a not-found message.
fn available_keys(palette: &VoxPalette) -> String {
    implementation::property_names(palette).join(", ")
}

/// Renders the collections under `layout`, wrapping the `row` layouts to the
/// `width` columns when given.
fn render(collections: &[Collection], layout: PaletteShowLayout, width: Option<usize>) -> String {
    match layout {
        PaletteShowLayout::Row => render_row(collections, true, width),
        PaletteShowLayout::RowNoHeader => render_row(collections, false, width),
        PaletteShowLayout::Column => render_column(collections, true),
        PaletteShowLayout::ColumnNoHeader => render_column(collections, false),
        PaletteShowLayout::Markdown => render_markdown(collections),
        PaletteShowLayout::PrettyJson => render_json(collections, true),
        PaletteShowLayout::CompactJson => render_json(collections, false),
    }
}

/// One material's text under its collection's `format`. A value with no swatch
/// prints its text under every format.
fn render_cell(sample: &Sample, format: PaletteShowFormat) -> String {
    match format {
        PaletteShowFormat::Auto => match &sample.swatch {
            Swatch::Color(rgb) => format!("{} {}", color_swatch(rgb), sample.text),
            Swatch::Gray(_) | Swatch::None => sample.text.clone(),
        },
        PaletteShowFormat::Swatch => match &sample.swatch {
            Swatch::Color(rgb) => color_swatch(rgb),
            Swatch::Gray(level) => gray_swatch(*level),
            Swatch::None => sample.text.clone(),
        },
        PaletteShowFormat::SwatchValue => match &sample.swatch {
            Swatch::Color(rgb) => format!("{} {}", color_swatch(rgb), sample.text),
            Swatch::Gray(level) => format!("{} {}", gray_swatch(*level), sample.text),
            Swatch::None => sample.text.clone(),
        },
        PaletteShowFormat::Value => sample.text.clone(),
    }
}

/// A two-cell truecolor swatch of an rgb color.
fn color_swatch(rgb: &[u8; 3]) -> String {
    format!("\x1b[48;2;{};{};{}m  \x1b[0m", rgb[0], rgb[1], rgb[2])
}

/// A two-cell truecolor grayscale swatch of a `0..255` gray level.
fn gray_swatch(level: u8) -> String {
    format!("\x1b[48;2;{level};{level};{level}m  \x1b[0m")
}

/// The gray level for a scalar, its `0..1` value mapped onto `0..255`.
fn scalar_level(value: f64) -> u8 {
    value.to_unorm8()
}

/// Whether a collection's cells abut into a strip: a `swatch` collection whose
/// every value carries a swatch. A value with no swatch, such as a bool, keeps
/// the one-space separator so it does not run together.
fn abuts(collection: &Collection) -> bool {
    matches!(collection.format, PaletteShowFormat::Swatch)
        && collection
            .samples
            .iter()
            .all(|sample| !matches!(sample.swatch, Swatch::None))
}

/// Each collection on one row, the rows separated by a blank line. A swatch
/// collection's cells abut into a strip; other formats put one space between
/// cells. With `with_header`, only the header is padded, to the longest, so the
/// first value of every row aligns; the values themselves are not padded. A
/// `width` wraps a row that overflows it onto continuation lines indented under
/// its first value.
fn render_row(collections: &[Collection], with_header: bool, width: Option<usize>) -> String {
    let headers: Vec<String> = collections.iter().map(Collection::header).collect();
    let cells: Vec<Vec<String>> = collections.iter().map(rendered_cells).collect();
    let header_width = headers
        .iter()
        .map(|h| implementation::visible_width(h))
        .max()
        .unwrap_or(0);
    let indent = if with_header { header_width + 1 } else { 0 };

    let mut blocks: Vec<String> = Vec::new();
    for ((collection, header), row) in collections.iter().zip(&headers).zip(&cells) {
        // Swatches abut into a strip; other formats, and swatch cells that fell
        // back to raw text, keep one space so values stay legible.
        let separator = if abuts(collection) { "" } else { " " };
        let segments = match width {
            // Leave room for at least one cell beside the header indent.
            Some(width) => wrap_cells(row, separator, width.saturating_sub(indent).max(1)),
            None => vec![row.join(separator)],
        };
        blocks.push(assemble_row(
            header,
            header_width,
            indent,
            with_header,
            &segments,
        ));
    }
    if blocks.is_empty() {
        String::new()
    } else {
        format!("{}\n", blocks.join("\n\n"))
    }
}

/// Splits `cells` into segments whose visible width stays within `budget`,
/// joining each segment's cells with `separator`. A cell wider than `budget`
/// takes a segment of its own.
fn wrap_cells(cells: &[String], separator: &str, budget: usize) -> Vec<String> {
    let separator_width = separator.chars().count();
    let mut segments: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut width = 0;
    for cell in cells {
        let cell_width = implementation::visible_width(cell);
        if !current.is_empty() && width + separator_width + cell_width > budget {
            segments.push(current.join(separator));
            current.clear();
            width = 0;
        }
        if !current.is_empty() {
            width += separator_width;
        }
        width += cell_width;
        current.push(cell);
    }
    if !current.is_empty() {
        segments.push(current.join(separator));
    }
    segments
}

/// Assembles one collection's row from its wrapped `segments`: the first line
/// carries the padded header, the continuation lines indent under the first
/// value, and every line is right-trimmed. An empty row keeps just its header.
fn assemble_row(
    header: &str,
    header_width: usize,
    indent: usize,
    with_header: bool,
    segments: &[String],
) -> String {
    if segments.is_empty() {
        return if with_header {
            implementation::pad_right(header, header_width)
                .trim_end()
                .to_string()
        } else {
            String::new()
        };
    }
    segments
        .iter()
        .enumerate()
        .map(|(line, segment)| {
            let prefix = match (line, with_header) {
                (0, true) => format!("{} ", implementation::pad_right(header, header_width)),
                (0, false) => String::new(),
                _ => " ".repeat(indent),
            };
            format!("{prefix}{segment}").trim_end().to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Each collection as its own column, padded to a common visible width so the
/// values read straight down. With `with_header`, each column is topped by its
/// header, which also widens the column to fit it.
fn render_column(collections: &[Collection], with_header: bool) -> String {
    let headers: Vec<String> = collections.iter().map(Collection::header).collect();
    let cells: Vec<Vec<String>> = collections.iter().map(rendered_cells).collect();
    let widths: Vec<usize> = headers
        .iter()
        .zip(&cells)
        .map(|(header, column)| {
            let cell = column
                .iter()
                .map(|c| implementation::visible_width(c))
                .max()
                .unwrap_or(0);
            if with_header {
                implementation::visible_width(header).max(cell)
            } else {
                cell
            }
        })
        .collect();
    let row_count = cells.iter().map(Vec::len).max().unwrap_or(0);

    let mut output = String::new();
    if with_header {
        output.push_str(join_padded(&headers, &widths).trim_end());
        output.push('\n');
    }
    for row in 0..row_count {
        let line: Vec<String> = cells
            .iter()
            .map(|column| column.get(row).cloned().unwrap_or_default())
            .collect();
        output.push_str(join_padded(&line, &widths).trim_end());
        output.push('\n');
    }
    output
}

/// The collections as an aligned markdown table led by a `#` column of 0-based
/// material indices, then one column per collection and one row per material
/// index. A shorter palette leaves its column blank past its last material.
fn render_markdown(collections: &[Collection]) -> String {
    if collections.is_empty() {
        return String::new();
    }
    let headers: Vec<String> = collections.iter().map(Collection::header).collect();
    let cells: Vec<Vec<String>> = collections.iter().map(rendered_cells).collect();
    let row_count = cells.iter().map(Vec::len).max().unwrap_or(0);
    // One row per material index, led by the index and then each collection's
    // cell or a blank.
    let rows: Vec<Vec<String>> = (0..row_count)
        .map(|row| {
            let mut cells_row = vec![row.to_string()];
            cells_row.extend(
                cells
                    .iter()
                    .map(|column| column.get(row).cloned().unwrap_or_default()),
            );
            cells_row
        })
        .collect();
    let mut header_refs: Vec<&str> = vec!["#"];
    header_refs.extend(headers.iter().map(String::as_str));
    implementation::markdown_table(&header_refs, &rows)
}

/// Joins one value per column, each padded to its column width, with single
/// spaces between columns.
fn join_padded(values: &[String], widths: &[usize]) -> String {
    values
        .iter()
        .zip(widths)
        .map(|(value, width)| implementation::pad_right(value, *width))
        .collect::<Vec<_>>()
        .join(" ")
}

/// One collection's cells rendered to text under its format.
fn rendered_cells(collection: &Collection) -> Vec<String> {
    collection
        .samples
        .iter()
        .map(|sample| render_cell(sample, collection.format))
        .collect()
}

/// The collections as JSON, pretty or compact: one record per collection in
/// render order. Values emit in their native JSON types; the per-collection
/// format is ignored.
fn render_json(collections: &[Collection], pretty: bool) -> String {
    let records: Vec<Value> = collections
        .iter()
        .map(|collection| {
            let values: Vec<Value> = collection
                .samples
                .iter()
                .map(|sample| sample.json.clone())
                .collect();
            // Field by field so `scalar` appears only on a scalar property.
            let mut record = Map::new();
            record.insert("palette".to_string(), json!(collection.palette));
            record.insert("property".to_string(), json!(collection.property()));
            if collection.scalar {
                record.insert("scalar".to_string(), json!(true));
            }
            record.insert("values".to_string(), Value::Array(values));
            Value::Object(record)
        })
        .collect();
    let payload = Value::Array(records);
    let text = if pretty {
        serde_json::to_string_pretty(&payload)
    } else {
        serde_json::to_string(&payload)
    }
    .expect("palette values serialize to JSON");
    format!("{text}\n")
}

#[cfg(test)]
mod tests {
    use crate::{
        commands::{PaletteShowLayout, PropertySelector},
        implementation::palette_show::{render, resolve_collections},
    };
    use branded_id::{IdVec, U32Id};
    use serde_json::Value;
    use voxcore::{
        BVoxPoolValue, BVoxValuePool, VoxBound, VoxMain, VoxPalette, VoxValue, VoxValuePool,
    };

    /// The branded value id `index`.
    fn value(index: usize) -> U32Id<BVoxPoolValue> {
        U32Id::from_u32(index as u32)
    }

    /// An `Srgba` pool of the given 8-bit colors, each byte divided by 255, the
    /// way the converters store colors.
    fn srgba_pool(state: &mut VoxMain, colors: &[[u8; 4]]) -> U32Id<BVoxValuePool> {
        let values = colors
            .iter()
            .map(|color| {
                [
                    color[0] as f64 / 255.0,
                    color[1] as f64 / 255.0,
                    color[2] as f64 / 255.0,
                    color[3] as f64 / 255.0,
                ]
            })
            .collect();
        state.add_value_pool(VoxValuePool::Srgba { values })
    }

    /// A document with two palettes: palette 0 has `baseColorFactor` and
    /// `metallicFactor` with two materials, palette 1 has `baseColorFactor` with
    /// one material.
    fn sample_state() -> VoxMain {
        let mut state = VoxMain::default();

        let colors_zero = srgba_pool(&mut state, &[[255, 0, 0, 255], [0, 255, 0, 128]]);
        let metallic = state.add_value_pool(VoxValuePool::Float {
            min: VoxBound::Number(0.0),
            max: VoxBound::Number(1.0),
            values: IdVec::from_vec(vec![1.0, 0.2]),
        });
        let colors_one = srgba_pool(&mut state, &[[0, 0, 255, 255]]);

        let mut first = VoxPalette::default();
        first.add_array_property("baseColorFactor".to_owned(), colors_zero);
        first.add_array_property("metallicFactor".to_owned(), metallic);
        first.add_material(vec![value(0), value(0)]).unwrap();
        first.add_material(vec![value(1), value(1)]).unwrap();
        state.add_palette(first);

        let mut second = VoxPalette::default();
        second.add_array_property("baseColorFactor".to_owned(), colors_one);
        second.add_material(vec![value(0)]).unwrap();
        state.add_palette(second);

        state
    }

    fn selectors(fields: &[(&str, &str, &str)]) -> Vec<PropertySelector> {
        fields
            .iter()
            .map(|(palette, property, format)| {
                PropertySelector::parse(palette, property, format).unwrap()
            })
            .collect()
    }

    fn show(state: &VoxMain, fields: &[(&str, &str, &str)], layout: PaletteShowLayout) -> String {
        let collections = resolve_collections(state, &selectors(fields)).unwrap();
        render(&collections, layout, None)
    }

    #[test]
    fn value_layout_prints_canonical_hex_with_a_header() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "baseColorFactor", "value")],
            PaletteShowLayout::Row,
        );
        assert_eq!(output, "0.\"baseColorFactor\" #FF0000FF #00FF0080\n");
    }

    #[test]
    fn extracts_a_color_component_as_a_byte() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "baseColorFactor.a", "value")],
            PaletteShowLayout::Row,
        );
        // Alpha bytes FF and 80 as 0..255 integers.
        assert_eq!(output, "0.\"baseColorFactor\".a 255 128\n");
    }

    #[test]
    fn swatch_layout_abuts_swatches_into_a_strip() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "baseColorFactor", "swatch")],
            PaletteShowLayout::Row,
        );
        assert_eq!(
            output,
            "0.\"baseColorFactor\" \x1b[48;2;255;0;0m  \x1b[0m\x1b[48;2;0;255;0m  \x1b[0m\n"
        );
    }

    #[test]
    fn swatch_spaces_values_with_no_swatch() {
        let mut state = VoxMain::default();
        let shadows = state.add_value_pool(VoxValuePool::Bool {
            values: IdVec::from_vec(vec![true, false]),
        });
        let mut palette = VoxPalette::default();
        palette.add_array_property("shadows".to_owned(), shadows);
        palette.add_material(vec![value(0)]).unwrap();
        palette.add_material(vec![value(1)]).unwrap();
        state.add_palette(palette);

        // Bools have no swatch, so swatch format spaces them rather than
        // abutting them into `truefalse`.
        let output = show(
            &state,
            &[("0", "shadows", "swatch")],
            PaletteShowLayout::Row,
        );
        assert_eq!(output, "0.\"shadows\" true false\n");
    }

    #[test]
    fn row_pads_only_the_header_not_the_values() {
        let state = sample_state();
        // The headers pad to the longest so each row's first value aligns, but
        // the values are not column-aligned: `metallicFactor` stays compact
        // rather than padding out to the wider `baseColorFactor` columns.
        let output = show(
            &state,
            &[
                ("0", "baseColorFactor", "value"),
                ("0", "metallicFactor", "value"),
            ],
            PaletteShowLayout::Row,
        );
        assert_eq!(
            output,
            "0.\"baseColorFactor\" #FF0000FF #00FF0080\n\
             \n\
             0.\"metallicFactor\"  1 0.2\n"
        );
    }

    #[test]
    fn row_wraps_cells_to_the_width() {
        let state = sample_state();
        let collections =
            resolve_collections(&state, &selectors(&[("0", "baseColorFactor", "value")])).unwrap();
        // Width 30 leaves 10 columns after the `0."baseColorFactor" ` prefix: one
        // 9-wide hex fits per line, so the second wraps under the first.
        let output = render(&collections, PaletteShowLayout::Row, Some(30));
        assert_eq!(
            output,
            "0.\"baseColorFactor\" #FF0000FF\n                    #00FF0080\n"
        );
    }

    #[test]
    fn row_no_header_drops_the_header_column() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "baseColorFactor", "value")],
            PaletteShowLayout::RowNoHeader,
        );
        assert_eq!(output, "#FF0000FF #00FF0080\n");
    }

    #[test]
    fn default_selector_shows_every_palette_and_property() {
        let state = sample_state();
        let collections =
            resolve_collections(&state, &[PropertySelector::default_all_auto()]).unwrap();
        let output = render(&collections, PaletteShowLayout::Row, None);
        assert_eq!(
            output,
            "0.\"baseColorFactor\" \x1b[48;2;255;0;0m  \x1b[0m #FF0000FF \x1b[48;2;0;255;0m  \x1b[0m #00FF0080\n\
             \n\
             0.\"metallicFactor\"  1 0.2\n\
             \n\
             1.\"baseColorFactor\" \x1b[48;2;0;0;255m  \x1b[0m #0000FFFF\n"
        );
    }

    #[test]
    fn column_layout_stacks_collections_under_headers() {
        let state = sample_state();
        let output = show(
            &state,
            &[
                ("0", "baseColorFactor.a", "value"),
                ("1", "baseColorFactor.a", "value"),
            ],
            PaletteShowLayout::Column,
        );
        assert_eq!(
            output,
            "0.\"baseColorFactor\".a 1.\"baseColorFactor\".a\n255                   255\n128\n"
        );
    }

    #[test]
    fn column_no_header_drops_the_header_row() {
        let state = sample_state();
        let output = show(
            &state,
            &[
                ("0", "baseColorFactor.a", "value"),
                ("1", "baseColorFactor.a", "value"),
            ],
            PaletteShowLayout::ColumnNoHeader,
        );
        assert_eq!(output, "255 255\n128\n");
    }

    #[test]
    fn markdown_layout_fills_an_aligned_table() {
        let state = sample_state();
        let output = show(
            &state,
            &[("*", "baseColorFactor", "value")],
            PaletteShowLayout::Markdown,
        );
        assert_eq!(
            output,
            "| #   | 0.\"baseColorFactor\" | 1.\"baseColorFactor\" |\n\
             | --- | ------------------- | ------------------- |\n\
             | 0   | #FF0000FF           | #0000FFFF           |\n\
             | 1   | #00FF0080           |                     |\n"
        );
    }

    #[test]
    fn compact_json_emits_records_in_render_order() {
        let state = sample_state();
        let output = show(
            &state,
            &[
                ("0", "baseColorFactor", "value"),
                ("0", "baseColorFactor.a", "value"),
            ],
            PaletteShowLayout::CompactJson,
        );
        assert_eq!(
            output,
            "[{\"palette\":0,\"property\":\"baseColorFactor\",\"values\":[\"#FF0000FF\",\"#00FF0080\"]},\
             {\"palette\":0,\"property\":\"baseColorFactor.a\",\"values\":[255,128]}]\n"
        );
    }

    #[test]
    fn pretty_json_is_indented_and_matches_compact() {
        let state = sample_state();
        let fields: &[(&str, &str, &str)] = &[
            ("0", "baseColorFactor", "value"),
            ("0", "baseColorFactor.a", "value"),
        ];
        let pretty = show(&state, fields, PaletteShowLayout::PrettyJson);
        let compact = show(&state, fields, PaletteShowLayout::CompactJson);
        // Indented, and carrying the same data as the compact form.
        assert!(pretty.contains("\n  "));
        let pretty_value: Value = serde_json::from_str(&pretty).unwrap();
        let compact_value: Value = serde_json::from_str(&compact).unwrap();
        assert_eq!(pretty_value, compact_value);
    }

    #[test]
    fn star_property_expands_to_every_property() {
        let state = sample_state();
        let collections = resolve_collections(&state, &selectors(&[("0", "*", "value")])).unwrap();
        let headers: Vec<String> = collections.iter().map(|c| c.header()).collect();
        assert_eq!(headers, ["0.\"baseColorFactor\"", "0.\"metallicFactor\""]);
    }

    #[test]
    fn star_palette_skips_a_palette_lacking_a_named_property() {
        let state = sample_state();
        // Only palette 0 has `metallicFactor`; palette 1 is skipped, not an error.
        let collections =
            resolve_collections(&state, &selectors(&[("*", "metallicFactor", "value")])).unwrap();
        let headers: Vec<String> = collections.iter().map(|c| c.header()).collect();
        assert_eq!(headers, ["0.\"metallicFactor\""]);
    }

    #[test]
    fn named_palette_out_of_range_is_an_error() {
        let state = sample_state();
        assert!(
            resolve_collections(&state, &selectors(&[("5", "baseColorFactor", "value")])).is_err()
        );
    }

    #[test]
    fn named_property_absent_on_a_named_palette_is_an_error() {
        let state = sample_state();
        assert!(resolve_collections(&state, &selectors(&[("0", "missing", "value")])).is_err());
    }

    #[test]
    fn a_component_on_a_scalar_is_an_error() {
        let state = sample_state();
        assert!(
            resolve_collections(&state, &selectors(&[("0", "metallicFactor.r", "value")])).is_err()
        );
    }

    #[test]
    fn auto_swatches_colors_but_prints_scalars_and_components_as_numbers() {
        let state = sample_state();
        let scalar = show(
            &state,
            &[("0", "metallicFactor", "auto")],
            PaletteShowLayout::Row,
        );
        assert_eq!(scalar, "0.\"metallicFactor\" 1 0.2\n");
        let component = show(
            &state,
            &[("0", "baseColorFactor.r", "auto")],
            PaletteShowLayout::Row,
        );
        assert_eq!(component, "0.\"baseColorFactor\".r 255 0\n");
    }

    /// One palette binding `tint` to a three-component `Srgb` pool holding a red.
    fn three_component_state() -> VoxMain {
        let mut state = VoxMain::default();
        let pool = state.add_value_pool(VoxValuePool::Srgb {
            values: IdVec::from_vec(vec![[1.0, 0.0, 0.0]]),
        });
        let mut palette = VoxPalette::default();
        palette.add_array_property("tint".to_owned(), pool);
        palette.add_material(vec![value(0)]).unwrap();
        state.add_palette(palette);
        state
    }

    #[test]
    fn a_three_component_color_renders_hex_without_alpha() {
        let state = three_component_state();
        let output = show(&state, &[("0", "tint", "value")], PaletteShowLayout::Row);
        // Six hex digits, no alpha pair.
        assert_eq!(output, "0.\"tint\" #FF0000\n");
    }

    #[test]
    fn a_three_component_color_has_no_alpha_component() {
        let state = three_component_state();
        // `.a` is out of range on a three-component color, but `.r` reads.
        assert!(resolve_collections(&state, &selectors(&[("0", "tint.a", "value")])).is_err());
        let red = show(&state, &[("0", "tint.r", "value")], PaletteShowLayout::Row);
        assert_eq!(red, "0.\"tint\".r 255\n");
    }

    #[test]
    fn a_linear_color_renders_float_components() {
        let mut state = VoxMain::default();
        // A linear pool carries HDR components above 1 that no hex can hold.
        let pool = state.add_value_pool(VoxValuePool::LinearRgba {
            values: IdVec::from_vec(vec![[2.0, 1.0, 0.5, 1.0]]),
        });
        let mut palette = VoxPalette::default();
        palette.add_array_property("emissiveFactor".to_owned(), pool);
        palette.add_material(vec![value(0)]).unwrap();
        state.add_palette(palette);

        let output = show(
            &state,
            &[("0", "emissiveFactor", "value")],
            PaletteShowLayout::Row,
        );
        assert_eq!(output, "0.\"emissiveFactor\" 2 1 0.5 1\n");
    }

    #[test]
    fn an_int_pool_renders_integers() {
        let mut state = VoxMain::default();
        let pool = state.add_value_pool(VoxValuePool::Int {
            min: VoxBound::Number(0.0),
            max: VoxBound::None,
            values: IdVec::from_vec(vec![3, 7]),
        });
        let mut palette = VoxPalette::default();
        palette.add_array_property("count".to_owned(), pool);
        palette.add_material(vec![value(0)]).unwrap();
        palette.add_material(vec![value(1)]).unwrap();
        state.add_palette(palette);

        let output = show(&state, &[("0", "count", "value")], PaletteShowLayout::Row);
        assert_eq!(output, "0.\"count\" 3 7\n");
    }

    #[test]
    fn a_json_pool_renders_arrays_rather_than_null() {
        let mut state = VoxMain::default();
        let pool = state.add_value_pool(VoxValuePool::Json {
            values: IdVec::from_vec(vec![VoxValue::Array(vec![
                VoxValue::Number(1.0),
                VoxValue::Number(2.0),
            ])]),
        });
        let mut palette = VoxPalette::default();
        palette.add_array_property("extra".to_owned(), pool);
        palette.add_material(vec![value(0)]).unwrap();
        state.add_palette(palette);

        // The array survives into both the text and JSON layouts.
        let text = show(&state, &[("0", "extra", "value")], PaletteShowLayout::Row);
        assert_eq!(text, "0.\"extra\" [1,2]\n");
        let json = show(
            &state,
            &[("0", "extra", "value")],
            PaletteShowLayout::CompactJson,
        );
        assert_eq!(
            json,
            "[{\"palette\":0,\"property\":\"extra\",\"values\":[[1,2]]}]\n"
        );
    }

    #[test]
    fn an_empty_property_name_is_quoted_in_the_header_but_raw_in_json() {
        let mut state = VoxMain::default();
        let pool = state.add_value_pool(VoxValuePool::Bool {
            values: IdVec::from_vec(vec![true]),
        });
        let mut palette = VoxPalette::default();
        // A binding with no property name, reached through the `*` property.
        palette.add_array_property(String::new(), pool);
        palette.add_material(vec![value(0)]).unwrap();
        state.add_palette(palette);

        // An empty name prints quoted as `""` rather than vanishing after the
        // `0.` prefix.
        let row = show(&state, &[("0", "*", "value")], PaletteShowLayout::Row);
        assert_eq!(row, "0.\"\" true\n");
        // JSON keeps the raw name; its own string quoting is enough there.
        let json = show(
            &state,
            &[("0", "*", "value")],
            PaletteShowLayout::CompactJson,
        );
        assert_eq!(
            json,
            "[{\"palette\":0,\"property\":\"\",\"values\":[true]}]\n"
        );
    }

    /// One palette pinning a scalar `emissiveStrength` and a scalar color
    /// `tint` beside a one-material `baseColorFactor` column.
    fn scalar_state() -> VoxMain {
        let mut state = VoxMain::default();
        let colors = srgba_pool(&mut state, &[[255, 0, 0, 255]]);
        let strengths = state.add_value_pool(VoxValuePool::Float {
            min: VoxBound::Number(0.0),
            max: VoxBound::None,
            values: IdVec::from_vec(vec![2.0]),
        });
        let tints = srgba_pool(&mut state, &[[0, 255, 0, 128]]);

        let mut palette = VoxPalette::default();
        palette.add_array_property("baseColorFactor".to_owned(), colors);
        palette.add_scalar_property("emissiveStrength".to_owned(), strengths, value(0));
        palette.add_scalar_property("tint".to_owned(), tints, value(0));
        palette.add_material(vec![value(0)]).unwrap();
        state.add_palette(palette);
        state
    }

    #[test]
    fn a_scalar_property_shows_its_one_pinned_value() {
        let state = scalar_state();
        let output = show(
            &state,
            &[("0", "emissiveStrength", "value")],
            PaletteShowLayout::Row,
        );
        assert_eq!(output, "0.\"emissiveStrength\" (scalar) 2\n");
    }

    #[test]
    fn star_property_expands_scalar_properties_after_array_ones() {
        let state = scalar_state();
        let collections = resolve_collections(&state, &selectors(&[("0", "*", "value")])).unwrap();
        let headers: Vec<String> = collections.iter().map(|c| c.header()).collect();
        assert_eq!(
            headers,
            [
                "0.\"baseColorFactor\"",
                "0.\"emissiveStrength\" (scalar)",
                "0.\"tint\" (scalar)"
            ]
        );
    }

    #[test]
    fn a_scalar_color_reads_like_a_material_value() {
        let state = scalar_state();
        let whole = show(&state, &[("0", "tint", "value")], PaletteShowLayout::Row);
        assert_eq!(whole, "0.\"tint\" (scalar) #00FF0080\n");
        let component = show(&state, &[("0", "tint.a", "value")], PaletteShowLayout::Row);
        assert_eq!(component, "0.\"tint\".a (scalar) 128\n");
    }

    #[test]
    fn a_component_on_a_scalar_number_property_is_an_error() {
        let state = scalar_state();
        let result =
            resolve_collections(&state, &selectors(&[("0", "emissiveStrength.r", "value")]));
        assert!(result.is_err());
    }

    #[test]
    fn scalar_json_records_mark_the_arity() {
        let state = scalar_state();
        let output = show(
            &state,
            &[
                ("0", "baseColorFactor", "value"),
                ("0", "emissiveStrength", "value"),
            ],
            PaletteShowLayout::CompactJson,
        );
        assert_eq!(
            output,
            "[{\"palette\":0,\"property\":\"baseColorFactor\",\"values\":[\"#FF0000FF\"]},\
             {\"palette\":0,\"property\":\"emissiveStrength\",\"scalar\":true,\"values\":[2]}]\n"
        );
    }

    #[test]
    fn a_scalar_only_palette_shows_with_no_materials() {
        // No materials at all: the palette is never sampled, but its pinned
        // value still shows.
        let mut state = VoxMain::default();
        let strengths = state.add_value_pool(VoxValuePool::Float {
            min: VoxBound::Number(0.0),
            max: VoxBound::None,
            values: IdVec::from_vec(vec![0.5]),
        });
        let mut palette = VoxPalette::default();
        palette.add_scalar_property("emissiveStrength".to_owned(), strengths, value(0));
        state.add_palette(palette);

        let output = show(&state, &[("0", "*", "value")], PaletteShowLayout::Row);
        assert_eq!(output, "0.\"emissiveStrength\" (scalar) 0.5\n");
    }
}
