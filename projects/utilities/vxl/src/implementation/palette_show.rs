use crate::{
    AttributeRef, AttributeSelector, AttributeType, ColorComponent, Format, PaletteRef,
    PaletteShowFormat, PaletteShowLayout, Result, Width, implementation,
};
use serde_json::{Value, json};
use std::{
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use voxcore::{VoxMain, VoxPalette, VoxValue};

/// Loads the voxel file at `input` and prints the value collections named by
/// `selectors`. Each selector resolves to one or more collections, an
/// attribute's values down a palette; `layout` arranges them and chooses the
/// serialization, and `width` wraps the `row` layouts.
pub fn palette_show(
    input: &Path,
    from: Option<Format>,
    selectors: &[AttributeSelector],
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

/// One resolved value collection: an attribute's values down one palette,
/// labeled by its palette index and attribute, with the format that renders it.
struct Collection {
    /// The resolved palette index, even when the selector used `*`.
    palette: usize,
    /// The attribute key, with the color component appended when one is read.
    attribute: String,
    /// How each value renders.
    format: PaletteShowFormat,
    /// One sample per palette cell, in cell order.
    samples: Vec<Sample>,
}

impl Collection {
    /// The `{palette}.{attribute}` header, with the component already folded
    /// into `attribute`.
    fn header(&self) -> String {
        format!("{}.{}", self.palette, self.attribute)
    }
}

/// One rendered cell value: a whole color, a color component byte, a scalar
/// number, or a raw fallback for a value that matches neither its inferred type
/// nor a number.
enum Sample {
    Color(Rgba),
    Component(u8),
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

/// Resolves the selectors against the document's palettes into collections in
/// render order: selector order, then palette order, then attribute order. A
/// `*` palette or `*` attribute expands to one collection per match; a named
/// palette or attribute that is absent is an error, while a `*` palette quietly
/// skips a palette that lacks a named attribute.
fn resolve_collections(
    state: &VoxMain,
    selectors: &[AttributeSelector],
) -> Result<Vec<Collection>> {
    let palettes: Vec<&VoxPalette> = state.iter_palettes().map(|(_, palette)| palette).collect();
    let mut collections = Vec::new();
    for selector in selectors {
        match selector.palette {
            PaletteRef::All => {
                for (index, palette) in palettes.iter().enumerate() {
                    expand_attribute(
                        index,
                        palette,
                        &selector.attribute,
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
                expand_attribute(
                    index,
                    palette,
                    &selector.attribute,
                    selector.format,
                    false,
                    &mut collections,
                )?;
            }
        }
    }
    Ok(collections)
}

/// Expands one selector's attribute against one palette, pushing the resulting
/// collections. A `palette_is_wild` palette, one from a `*`, skips a named
/// attribute it lacks instead of erroring.
fn expand_attribute(
    index: usize,
    palette: &VoxPalette,
    attribute: &AttributeRef,
    format: PaletteShowFormat,
    palette_is_wild: bool,
    collections: &mut Vec<Collection>,
) -> Result<()> {
    match attribute {
        AttributeRef::All => {
            let names: Vec<String> = palette
                .iter_attributes()
                .map(|(_, name)| name.to_string())
                .collect();
            for name in names {
                collections.push(build_collection(index, palette, &name, None, format)?);
            }
        }
        AttributeRef::Key { key, component } => {
            let present = palette.iter_attributes().any(|(_, name)| name == key);
            if !present {
                if palette_is_wild {
                    return Ok(());
                }
                return Err(IOError::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "palette {index} has no attribute `{key}`; available attributes: {}",
                        available_keys(palette)
                    ),
                )
                .into());
            }
            collections.push(build_collection(index, palette, key, *component, format)?);
        }
    }
    Ok(())
}

/// Builds one collection from a present attribute: reads each cell's value,
/// infers the type, rejects a color component on a scalar, and samples each
/// value.
fn build_collection(
    index: usize,
    palette: &VoxPalette,
    key: &str,
    component: Option<ColorComponent>,
    format: PaletteShowFormat,
) -> Result<Collection> {
    let attribute_id = palette
        .iter_attributes()
        .find(|(_, name)| *name == key)
        .map(|(id, _)| id)
        .expect("caller verified the attribute is present");

    // A retained cell holds a value for every attribute, so the lookup with
    // this palette's own id always resolves.
    let values: Vec<&VoxValue> = palette
        .iter_cells()
        .map(|cell| {
            palette
                .cell_value(cell, attribute_id)
                .expect("a cell holds a value for every attribute")
        })
        .collect();

    let inferred = infer_type(&values);
    if let Some(component) = component
        && inferred == AttributeType::Scalar
    {
        return Err(IOError::new(
            ErrorKind::InvalidInput,
            format!(
                "attribute `{key}` is a scalar and has no `.{}` color component",
                component_letter(component)
            ),
        )
        .into());
    }

    let samples = values
        .iter()
        .map(|&value| sample(value, inferred, component))
        .collect();
    let attribute = match component {
        Some(component) => format!("{key}.{}", component_letter(component)),
        None => key.to_string(),
    };
    Ok(Collection {
        palette: index,
        attribute,
        format,
        samples,
    })
}

/// The sample for one value under the inferred `r#type` and `component`:
/// a color or its `component` byte, a scalar's number, else a raw fallback.
fn sample(value: &VoxValue, r#type: AttributeType, component: Option<ColorComponent>) -> Sample {
    match r#type {
        AttributeType::Color => match value {
            VoxValue::Text(text) => match parse_rgba(text) {
                Some(rgba) => match component {
                    Some(component) => Sample::Component(component_byte(&rgba, component)),
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

/// One color component as a `0..255` byte, the value as stored in the hex.
fn component_byte(rgba: &Rgba, component: ColorComponent) -> u8 {
    match component {
        ColorComponent::R => rgba.r,
        ColorComponent::G => rgba.g,
        ColorComponent::B => rgba.b,
        ColorComponent::A => rgba.a,
    }
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

/// One cell's text under its collection's `format`. A raw fallback prints its
/// text under every format, with no swatch.
fn render_cell(sample: &Sample, format: PaletteShowFormat) -> String {
    match (format, sample) {
        (PaletteShowFormat::Auto, Sample::Color(rgba)) => {
            format!("{} {}", color_swatch(rgba), hex(rgba))
        }
        (PaletteShowFormat::Auto, Sample::Component(byte)) => format!("{byte}"),
        (PaletteShowFormat::Auto, Sample::Scalar(value)) => format!("{value}"),
        (PaletteShowFormat::Auto, Sample::Raw(text)) => text.clone(),

        (PaletteShowFormat::Swatch, Sample::Color(rgba)) => color_swatch(rgba),
        (PaletteShowFormat::Swatch, Sample::Component(byte)) => gray_swatch(*byte),
        (PaletteShowFormat::Swatch, Sample::Scalar(value)) => gray_swatch(scalar_level(*value)),
        (PaletteShowFormat::Swatch, Sample::Raw(text)) => text.clone(),

        (PaletteShowFormat::SwatchValue, Sample::Color(rgba)) => {
            format!("{} {}", color_swatch(rgba), hex(rgba))
        }
        (PaletteShowFormat::SwatchValue, Sample::Component(byte)) => {
            format!("{} {byte}", gray_swatch(*byte))
        }
        (PaletteShowFormat::SwatchValue, Sample::Scalar(value)) => {
            format!("{} {value}", gray_swatch(scalar_level(*value)))
        }
        (PaletteShowFormat::SwatchValue, Sample::Raw(text)) => text.clone(),

        (PaletteShowFormat::Value, Sample::Color(rgba)) => hex(rgba),
        (PaletteShowFormat::Value, Sample::Component(byte)) => format!("{byte}"),
        (PaletteShowFormat::Value, Sample::Scalar(value)) => format!("{value}"),
        (PaletteShowFormat::Value, Sample::Raw(text)) => text.clone(),
    }
}

/// A two-cell truecolor swatch of a color's rgb.
fn color_swatch(rgba: &Rgba) -> String {
    format!("\x1b[48;2;{};{};{}m  \x1b[0m", rgba.r, rgba.g, rgba.b)
}

/// A two-cell truecolor grayscale swatch of a `0..255` gray level.
fn gray_swatch(level: u8) -> String {
    format!("\x1b[48;2;{level};{level};{level}m  \x1b[0m")
}

/// The gray level for a scalar, its `0..1` value mapped onto `0..255`.
fn scalar_level(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Whether a collection's cells abut into a strip: a `swatch` collection whose
/// every value is a real swatch. A value with no swatch, a raw fallback like a
/// bool, keeps the one-space separator so it does not run together.
fn abuts(collection: &Collection) -> bool {
    matches!(collection.format, PaletteShowFormat::Swatch)
        && !collection
            .samples
            .iter()
            .any(|sample| matches!(sample, Sample::Raw(_)))
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
/// cell indices, then one column per collection and one row per cell index. A
/// shorter palette leaves its column blank past its last cell.
fn render_markdown(collections: &[Collection]) -> String {
    if collections.is_empty() {
        return String::new();
    }
    let headers: Vec<String> = collections.iter().map(Collection::header).collect();
    let cells: Vec<Vec<String>> = collections.iter().map(rendered_cells).collect();
    let row_count = cells.iter().map(Vec::len).max().unwrap_or(0);
    // One row per cell index, led by the index and then each collection's cell
    // or a blank.
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

/// The collections as JSON, one record per collection in render order: its
/// resolved palette index, its attribute label, and its values in native JSON
/// types. The format and layout are ignored; a color is a hex string, a
/// component its byte, and a scalar a number. `pretty` selects indented output,
/// matching the shared report JSON.
fn render_json(collections: &[Collection], pretty: bool) -> String {
    let records: Vec<Value> = collections
        .iter()
        .map(|collection| {
            let values: Vec<Value> = collection.samples.iter().map(sample_value).collect();
            json!({
                "palette": collection.palette,
                "attribute": collection.attribute,
                "values": values,
            })
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

/// One sample as a JSON value: a hex string for a color, a component byte, a
/// number for a scalar, a string for a raw fallback. An integral scalar emits
/// as an integer so it reads as it does in the text layouts.
fn sample_value(sample: &Sample) -> Value {
    match sample {
        Sample::Color(rgba) => Value::String(hex(rgba)),
        Sample::Component(byte) => json!(byte),
        Sample::Scalar(number) if number.fract() == 0.0 && number.abs() < i64::MAX as f64 => {
            json!(*number as i64)
        }
        Sample::Scalar(number) => json!(number),
        Sample::Raw(text) => Value::String(text.clone()),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        AttributeSelector, PaletteShowLayout,
        implementation::palette_show::{render, resolve_collections},
    };
    use serde_json::Value;
    use voxcore::{VoxMain, VoxPalette, VoxValue};

    /// A document with two palettes: palette 0 has `rgba` and `metallic` with
    /// two cells, palette 1 has `rgba` with one cell.
    fn sample_state() -> VoxMain {
        let mut state = VoxMain::default();

        let mut first = VoxPalette::default();
        first.add_attribute("rgba".to_owned());
        first.add_attribute("metallic".to_owned());
        first
            .add_cell(vec![
                VoxValue::Text("#FF0000FF".to_owned()),
                VoxValue::Number(1.0),
            ])
            .unwrap();
        first
            .add_cell(vec![
                VoxValue::Text("#00FF0080".to_owned()),
                VoxValue::Number(0.2),
            ])
            .unwrap();
        state.add_palette(first);

        let mut second = VoxPalette::default();
        second.add_attribute("rgba".to_owned());
        second
            .add_cell(vec![VoxValue::Text("#0000FFFF".to_owned())])
            .unwrap();
        state.add_palette(second);

        state
    }

    fn selectors(fields: &[(&str, &str, &str)]) -> Vec<AttributeSelector> {
        fields
            .iter()
            .map(|(palette, attribute, format)| {
                AttributeSelector::parse(palette, attribute, format).unwrap()
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
        let output = show(&state, &[("0", "rgba", "value")], PaletteShowLayout::Row);
        assert_eq!(output, "0.rgba #FF0000FF #00FF0080\n");
    }

    #[test]
    fn extracts_a_color_component_as_a_byte() {
        let state = sample_state();
        let output = show(&state, &[("0", "rgba.a", "value")], PaletteShowLayout::Row);
        // Alpha bytes FF and 80 as 0..255 integers.
        assert_eq!(output, "0.rgba.a 255 128\n");
    }

    #[test]
    fn swatch_layout_abuts_swatches_into_a_strip() {
        let state = sample_state();
        let output = show(&state, &[("0", "rgba", "swatch")], PaletteShowLayout::Row);
        assert_eq!(
            output,
            "0.rgba \x1b[48;2;255;0;0m  \x1b[0m\x1b[48;2;0;255;0m  \x1b[0m\n"
        );
    }

    #[test]
    fn swatch_spaces_values_with_no_swatch() {
        let mut state = VoxMain::default();
        let mut palette = VoxPalette::default();
        palette.add_attribute("shadows".to_owned());
        palette.add_cell(vec![VoxValue::Bool(true)]).unwrap();
        palette.add_cell(vec![VoxValue::Bool(false)]).unwrap();
        state.add_palette(palette);

        // Bools have no swatch, so swatch format spaces them rather than
        // abutting them into `truefalse`.
        let output = show(
            &state,
            &[("0", "shadows", "swatch")],
            PaletteShowLayout::Row,
        );
        assert_eq!(output, "0.shadows true false\n");
    }

    #[test]
    fn row_pads_only_the_header_not_the_values() {
        let state = sample_state();
        // The headers pad to the longest so each row's first value aligns, but
        // the values are not column-aligned: `metallic` stays compact rather
        // than padding out to the wider `rgba` columns.
        let output = show(
            &state,
            &[("0", "rgba", "value"), ("0", "metallic", "value")],
            PaletteShowLayout::Row,
        );
        assert_eq!(
            output,
            "0.rgba     #FF0000FF #00FF0080\n\
             \n\
             0.metallic 1 0.2\n"
        );
    }

    #[test]
    fn row_wraps_cells_to_the_width() {
        let state = sample_state();
        let collections =
            resolve_collections(&state, &selectors(&[("0", "rgba", "value")])).unwrap();
        // Width 20 leaves 13 columns after the `0.rgba ` prefix: one 9-wide hex
        // fits per line, so the second wraps under the first.
        let output = render(&collections, PaletteShowLayout::Row, Some(20));
        assert_eq!(output, "0.rgba #FF0000FF\n       #00FF0080\n");
    }

    #[test]
    fn row_no_header_drops_the_header_column() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "rgba", "value")],
            PaletteShowLayout::RowNoHeader,
        );
        assert_eq!(output, "#FF0000FF #00FF0080\n");
    }

    #[test]
    fn default_selector_shows_every_palette_rgba() {
        let state = sample_state();
        let collections =
            resolve_collections(&state, &[AttributeSelector::default_rgba_swatch()]).unwrap();
        let output = render(&collections, PaletteShowLayout::Row, None);
        assert_eq!(
            output,
            "0.rgba \x1b[48;2;255;0;0m  \x1b[0m\x1b[48;2;0;255;0m  \x1b[0m\n\
             \n\
             1.rgba \x1b[48;2;0;0;255m  \x1b[0m\n"
        );
    }

    #[test]
    fn column_layout_stacks_collections_under_headers() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "rgba.a", "value"), ("1", "rgba.a", "value")],
            PaletteShowLayout::Column,
        );
        assert_eq!(output, "0.rgba.a 1.rgba.a\n255      255\n128\n");
    }

    #[test]
    fn column_no_header_drops_the_header_row() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "rgba.a", "value"), ("1", "rgba.a", "value")],
            PaletteShowLayout::ColumnNoHeader,
        );
        assert_eq!(output, "255 255\n128\n");
    }

    #[test]
    fn markdown_layout_fills_an_aligned_table() {
        let state = sample_state();
        let output = show(
            &state,
            &[("*", "rgba", "value")],
            PaletteShowLayout::Markdown,
        );
        assert_eq!(
            output,
            "| #   | 0.rgba    | 1.rgba    |\n\
             | --- | --------- | --------- |\n\
             | 0   | #FF0000FF | #0000FFFF |\n\
             | 1   | #00FF0080 |           |\n"
        );
    }

    #[test]
    fn compact_json_emits_records_in_render_order() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "rgba", "value"), ("0", "rgba.a", "value")],
            PaletteShowLayout::CompactJson,
        );
        assert_eq!(
            output,
            "[{\"palette\":0,\"attribute\":\"rgba\",\"values\":[\"#FF0000FF\",\"#00FF0080\"]},\
             {\"palette\":0,\"attribute\":\"rgba.a\",\"values\":[255,128]}]\n"
        );
    }

    #[test]
    fn pretty_json_is_indented_and_matches_compact() {
        let state = sample_state();
        let fields: &[(&str, &str, &str)] = &[("0", "rgba", "value"), ("0", "rgba.a", "value")];
        let pretty = show(&state, fields, PaletteShowLayout::PrettyJson);
        let compact = show(&state, fields, PaletteShowLayout::CompactJson);
        // Indented, and carrying the same data as the compact form.
        assert!(pretty.contains("\n  "));
        let pretty_value: Value = serde_json::from_str(&pretty).unwrap();
        let compact_value: Value = serde_json::from_str(&compact).unwrap();
        assert_eq!(pretty_value, compact_value);
    }

    #[test]
    fn star_attribute_expands_to_every_attribute() {
        let state = sample_state();
        let collections = resolve_collections(&state, &selectors(&[("0", "*", "value")])).unwrap();
        let headers: Vec<String> = collections.iter().map(|c| c.header()).collect();
        assert_eq!(headers, ["0.rgba", "0.metallic"]);
    }

    #[test]
    fn star_palette_skips_a_palette_lacking_a_named_attribute() {
        let state = sample_state();
        // Only palette 0 has `metallic`; palette 1 is skipped, not an error.
        let collections =
            resolve_collections(&state, &selectors(&[("*", "metallic", "value")])).unwrap();
        let headers: Vec<String> = collections.iter().map(|c| c.header()).collect();
        assert_eq!(headers, ["0.metallic"]);
    }

    #[test]
    fn named_palette_out_of_range_is_an_error() {
        let state = sample_state();
        assert!(resolve_collections(&state, &selectors(&[("5", "rgba", "value")])).is_err());
    }

    #[test]
    fn named_attribute_absent_on_a_named_palette_is_an_error() {
        let state = sample_state();
        assert!(resolve_collections(&state, &selectors(&[("0", "missing", "value")])).is_err());
    }

    #[test]
    fn a_component_on_a_scalar_is_an_error() {
        let state = sample_state();
        assert!(resolve_collections(&state, &selectors(&[("0", "metallic.r", "value")])).is_err());
    }

    #[test]
    fn auto_swatches_colors_but_prints_scalars_and_components_as_numbers() {
        let state = sample_state();
        let scalar = show(&state, &[("0", "metallic", "auto")], PaletteShowLayout::Row);
        assert_eq!(scalar, "0.metallic 1 0.2\n");
        let component = show(&state, &[("0", "rgba.r", "auto")], PaletteShowLayout::Row);
        assert_eq!(component, "0.rgba.r 255 0\n");
    }
}
