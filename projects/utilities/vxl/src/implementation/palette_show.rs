use crate::{
    ColorComponent, Format, Result, Width,
    commands::{
        PaletteRef, PaletteShowFormat, PaletteShowLabel, PaletteShowLayout, PaletteShowTableShape,
        PropertyRef, PropertySelector,
    },
    implementation,
};
use branded_id::U32Id;
use serde_json::{Value, json};
use std::{
    io::{Error as IOError, ErrorKind},
    num::NonZeroU8,
    path::Path,
    result::Result as StdResult,
};
use treegrid::{
    BTreeGridNode, TreeGrid, TreeGridCellFormat, TreeGridError, TreeGridJsonValue,
    TreeGridJsonValueCells, TreeGridLabel, TreeGridLabelKind, TreeGridOptions,
    TreeGridRenderColumns, TreeGridRenderHierarchy, TreeGridRenderJson, TreeGridRenderRows,
    TreeGridRenderTables, TreeGridSwatch, TreeGridTableShapeKind,
};
use ty_math::{TyLinSrgbaF64, TySrgbaF64};
use voxcore::{BVoxPoolValue, VoxMain, VoxPalette, VoxPropertyId, VoxValue, VoxValuePool};

/// Loads the voxel file at `input` and prints the value collections named by
/// `selectors`, each a property's values down a palette, populated into a
/// tree grid of palette, property, and component nodes and rendered under
/// `layout`.
#[allow(clippy::too_many_arguments)]
pub fn palette_show(
    input: &Path,
    from: Option<Format>,
    selectors: &[PropertySelector],
    layout: PaletteShowLayout,
    label: Option<PaletteShowLabel>,
    header_level: Option<NonZeroU8>,
    table_shape: Option<PaletteShowTableShape>,
    width: Width,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;
    let collections = resolve_collections(&state, selectors)?;
    let grid = build_grid(collections);
    let output = render(&grid, layout, label, header_level, table_shape, width)?;
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

/// No terminal-width detection off unix; the `rows` layout does not wrap.
#[cfg(not(unix))]
fn terminal_columns() -> Option<usize> {
    None
}

/// One resolved value collection: a property's values down one palette,
/// addressed by its palette index and property, with the format that renders
/// it.
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
    samples: Vec<TreeGridJsonValue>,
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
) -> TreeGridJsonValue {
    match kind {
        Kind::Color { srgb, .. } => sample_color(pool, value_id, srgb, component),
        Kind::Number => sample_number(pool, value_id),
        Kind::Other => sample_other(pool, value_id),
    }
}

/// The sample for a color value: a whole color as hex (sRGB) or space-joined
/// float components (linear) with a color swatch, or, with a `component`, one
/// channel as a byte (sRGB) or float (linear) with a grayscale swatch.
fn sample_color(
    pool: &VoxValuePool,
    value_id: U32Id<BVoxPoolValue>,
    srgb: bool,
    component: Option<ColorComponent>,
) -> TreeGridJsonValue {
    let bytes = color_bytes(pool, value_id);
    match component {
        Some(component) => {
            let channel = component_index(component);
            if srgb {
                TreeGridJsonValue::unorm8(bytes[channel])
            } else {
                TreeGridJsonValue::unorm(color_floats(pool, value_id)[channel])
            }
        }
        None if srgb => {
            if alpha_component(pool) {
                TreeGridJsonValue::srgba8(bytes)
            } else {
                TreeGridJsonValue::srgb8([bytes[0], bytes[1], bytes[2]])
            }
        }
        None => {
            // Linear colors keep their space-joined component text rather
            // than the crate's functional notation.
            let floats = color_floats(pool, value_id);
            let text = floats
                .iter()
                .map(|value| format_number(*value))
                .collect::<Vec<_>>()
                .join(" ");
            let json = Value::Array(floats.iter().map(|value| number_json(*value)).collect());
            TreeGridJsonValue::new(text)
                .with_json(json)
                .with_swatch(TreeGridSwatch::Color([bytes[0], bytes[1], bytes[2]]))
        }
    }
}

/// The sample for a `float` or `int` value: its number, with a grayscale swatch
/// mapping its `0..1` range onto `0..255`.
fn sample_number(pool: &VoxValuePool, value_id: U32Id<BVoxPoolValue>) -> TreeGridJsonValue {
    let index = value_id.to_usize_id();
    let value = match pool {
        VoxValuePool::Float { values, .. } => values[index],
        VoxValuePool::Int { values, .. } => values[index] as f64,
        // classify() routes only Float and Int here.
        _ => 0.0,
    };
    TreeGridJsonValue::unorm(value)
}

/// The sample for a `bool`, `string`, or `json` value: its text and native JSON
/// with no swatch.
fn sample_other(pool: &VoxValuePool, value_id: U32Id<BVoxPoolValue>) -> TreeGridJsonValue {
    let index = value_id.to_usize_id();
    match pool {
        VoxValuePool::Bool { values } => TreeGridJsonValue::bool(values[index]),
        VoxValuePool::String { values } => TreeGridJsonValue::new(values[index].clone()),
        VoxValuePool::Json { values } => TreeGridJsonValue::json(vox_value_to_json(&values[index])),
        // classify() routes only Bool, String, and Json here.
        _ => TreeGridJsonValue::json(Value::Null),
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
            <[u8; 4]>::from(TySrgbaF64::new(r, g, b, 1.0).into_format::<u8, u8>())
        }
        VoxValuePool::Srgba { values } => {
            let [r, g, b, a] = values[index];
            <[u8; 4]>::from(TySrgbaF64::new(r, g, b, a).into_format::<u8, u8>())
        }
        VoxValuePool::LinearRgb { values } => {
            let [r, g, b] = values[index];
            <[u8; 4]>::from(
                TySrgbaF64::from_linear(TyLinSrgbaF64::new(r, g, b, 1.0)).into_format::<u8, u8>(),
            )
        }
        VoxValuePool::LinearRgba { values } => {
            let [r, g, b, a] = values[index];
            <[u8; 4]>::from(
                TySrgbaF64::from_linear(TyLinSrgbaF64::new(r, g, b, a)).into_format::<u8, u8>(),
            )
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

/// The palette's property keys joined for a not-found message.
fn available_keys(palette: &VoxPalette) -> String {
    implementation::property_names(palette).join(", ")
}

/// Populates a tree grid from the collections in order: palette root,
/// property child, component leaf, with each collection's samples on its
/// deepest node. A collection reuses the immediately preceding collection's
/// palette and property nodes when they match, so a contiguous run shares
/// its ancestors and pre-order keeps the selector order in every layout.
fn build_grid(collections: Vec<Collection>) -> TreeGrid<TreeGridJsonValueCells> {
    let mut grid = TreeGrid::with_cells(TreeGridJsonValueCells);
    let mut palette_node: Option<(usize, U32Id<BTreeGridNode>)> = None;
    let mut property_node: Option<(String, U32Id<BTreeGridNode>)> = None;
    for collection in collections {
        let palette = match palette_node {
            Some((index, id)) if index == collection.palette => id,
            _ => {
                let id = grid.add_root(TreeGridLabel::bare(collection.palette.to_string()));
                palette_node = Some((collection.palette, id));
                property_node = None;
                id
            }
        };
        let data = match collection.component {
            Some(component) => {
                let property = match &property_node {
                    Some((key, id)) if *key == collection.key => *id,
                    _ => grid.add_child(palette, TreeGridLabel::quoted(collection.key.as_str())),
                };
                property_node = Some((collection.key, property));
                let letter = component_letter(component).to_string();
                grid.add_child(property, TreeGridLabel::bare(letter))
            }
            None => {
                // A data node is always fresh, so a property selected twice
                // keeps one collection per selector.
                let id = grid.add_child(palette, TreeGridLabel::quoted(collection.key.as_str()));
                property_node = Some((collection.key, id));
                id
            }
        };
        let node = grid.node_mut(data);
        if collection.scalar {
            node.annotation = Some("(scalar)".to_owned());
        }
        node.format = cell_format(collection.format);
        node.values = collection.samples;
    }
    grid
}

/// The node cell format a `--property` format maps to; `auto` leaves the
/// format unset so the grid's cell policy decides per value.
fn cell_format(format: PaletteShowFormat) -> Option<TreeGridCellFormat> {
    match format {
        PaletteShowFormat::Auto => None,
        PaletteShowFormat::Swatch => Some(TreeGridCellFormat::Visual),
        PaletteShowFormat::SwatchValue => Some(TreeGridCellFormat::VisualText),
        PaletteShowFormat::Value => Some(TreeGridCellFormat::Text),
    }
}

/// Renders the grid under `layout`, mapping the flag values into the crate's
/// loose options; an option the chosen render does not consume is invalid
/// input. Only the `rows` render consumes a width, so only it resolves the
/// `width` flag.
fn render(
    grid: &TreeGrid<TreeGridJsonValueCells>,
    layout: PaletteShowLayout,
    label: Option<PaletteShowLabel>,
    header_level: Option<NonZeroU8>,
    table_shape: Option<PaletteShowTableShape>,
    width: Width,
) -> Result<String> {
    let mut options = TreeGridOptions::default();
    if let Some(label) = label {
        options = options.with_label(match label {
            PaletteShowLabel::None => TreeGridLabelKind::None,
            PaletteShowLabel::Concat => TreeGridLabelKind::Concat,
            PaletteShowLabel::Header => TreeGridLabelKind::Header,
        });
    }
    if let Some(level) = header_level {
        options = options.with_header_level(level);
    }
    if let Some(shape) = table_shape {
        options = options.with_table_shape(match shape {
            PaletteShowTableShape::Nested => TreeGridTableShapeKind::Nested,
            PaletteShowTableShape::Flat => TreeGridTableShapeKind::Flat,
            PaletteShowTableShape::Records => TreeGridTableShapeKind::Records,
        });
    }
    Ok(match layout {
        PaletteShowLayout::Hierarchy => {
            grid.render_hierarchy(&resolve_options(options.resolve_hierarchy())?)
        }
        PaletteShowLayout::Rows => {
            if let Some(columns) = resolve_width(width) {
                options = options.with_width(columns);
            }
            grid.render_rows(&resolve_options(options.resolve_rows())?)
        }
        PaletteShowLayout::Columns => {
            grid.render_columns(&resolve_options(options.resolve_columns())?)
        }
        PaletteShowLayout::Tables => {
            grid.render_tables(&resolve_options(options.resolve_tables())?)
        }
        PaletteShowLayout::JsonPretty => {
            resolve_options(options.resolve_json())?;
            grid.render_json_pretty()
        }
        PaletteShowLayout::JsonCompact => {
            resolve_options(options.resolve_json())?;
            grid.render_json_compact()
        }
    })
}

/// Maps an invalid option combination into an invalid-input error.
fn resolve_options<T>(resolved: StdResult<T, TreeGridError>) -> Result<T> {
    resolved.map_err(|error| IOError::new(ErrorKind::InvalidInput, error.to_string()).into())
}

#[cfg(test)]
mod tests {
    use crate::{
        Width,
        commands::{PaletteShowLabel, PaletteShowLayout, PaletteShowTableShape, PropertySelector},
        implementation::palette_show::{build_grid, render, resolve_collections},
    };
    use branded_id::{IdVec, U32Id};
    use serde_json::Value;
    use std::num::NonZeroU8;
    use treegrid::{TreeGrid, TreeGridJsonValueCells};
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

    /// The populated grid for the selectors, resolved against `state`.
    fn grid_for(
        state: &VoxMain,
        fields: &[(&str, &str, &str)],
    ) -> TreeGrid<TreeGridJsonValueCells> {
        build_grid(resolve_collections(state, &selectors(fields)).unwrap())
    }

    /// Renders the selectors under `layout` with default label options and no
    /// wrapping.
    fn show(state: &VoxMain, fields: &[(&str, &str, &str)], layout: PaletteShowLayout) -> String {
        render(
            &grid_for(state, fields),
            layout,
            None,
            None,
            None,
            Width::Unlimited,
        )
        .unwrap()
    }

    #[test]
    fn value_format_prints_canonical_hex_with_a_label() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "baseColorFactor", "value")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(output, "0.\"baseColorFactor\" #FF0000FF #00FF0080\n");
    }

    #[test]
    fn extracts_a_color_component_as_a_byte() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "baseColorFactor.a", "value")],
            PaletteShowLayout::Rows,
        );
        // Alpha bytes FF and 80 as 0..255 integers.
        assert_eq!(output, "0.\"baseColorFactor\".a 255 128\n");
    }

    #[test]
    fn swatch_format_abuts_swatches_into_a_strip() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "baseColorFactor", "swatch")],
            PaletteShowLayout::Rows,
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
            PaletteShowLayout::Rows,
        );
        assert_eq!(output, "0.\"shadows\" true false\n");
    }

    #[test]
    fn rows_pad_only_the_label_not_the_values() {
        let state = sample_state();
        // The labels pad to the longest so each row's first value aligns, but
        // the values are not column-aligned: `metallicFactor` stays compact
        // rather than padding out to the wider `baseColorFactor` columns.
        let output = show(
            &state,
            &[
                ("0", "baseColorFactor", "value"),
                ("0", "metallicFactor", "value"),
            ],
            PaletteShowLayout::Rows,
        );
        assert_eq!(
            output,
            "0.\"baseColorFactor\" #FF0000FF #00FF0080\n\
             \n\
             0.\"metallicFactor\"  1 0.2\n"
        );
    }

    #[test]
    fn rows_wrap_cells_to_the_width() {
        let state = sample_state();
        let grid = grid_for(&state, &[("0", "baseColorFactor", "value")]);
        // Width 30 leaves 10 columns after the `0."baseColorFactor" ` prefix:
        // one 9-wide hex fits per line, so the second wraps under the first.
        let output = render(
            &grid,
            PaletteShowLayout::Rows,
            None,
            None,
            None,
            Width::Columns(30),
        )
        .unwrap();
        assert_eq!(
            output,
            "0.\"baseColorFactor\" #FF0000FF\n                    #00FF0080\n"
        );
    }

    #[test]
    fn rows_with_label_none_drop_the_label_column() {
        let state = sample_state();
        let grid = grid_for(&state, &[("0", "baseColorFactor", "value")]);
        let output = render(
            &grid,
            PaletteShowLayout::Rows,
            Some(PaletteShowLabel::None),
            None,
            None,
            Width::Unlimited,
        )
        .unwrap();
        assert_eq!(output, "#FF0000FF #00FF0080\n");
    }

    #[test]
    fn default_selector_shows_every_palette_and_property() {
        let state = sample_state();
        let collections =
            resolve_collections(&state, &[PropertySelector::default_all_auto()]).unwrap();
        let output = render(
            &build_grid(collections),
            PaletteShowLayout::Rows,
            None,
            None,
            None,
            Width::Unlimited,
        )
        .unwrap();
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
    fn collections_render_in_selector_order() {
        let state = sample_state();
        // A palette revisited later starts a fresh root rather than merging
        // backward, so pre-order keeps the selector order.
        let output = show(
            &state,
            &[
                ("1", "baseColorFactor", "value"),
                ("0", "baseColorFactor", "value"),
            ],
            PaletteShowLayout::Rows,
        );
        assert_eq!(
            output,
            "1.\"baseColorFactor\" #0000FFFF\n\
             \n\
             0.\"baseColorFactor\" #FF0000FF #00FF0080\n"
        );
    }

    #[test]
    fn a_repeated_property_keeps_one_row_per_selector() {
        let state = sample_state();
        let output = show(
            &state,
            &[
                ("0", "baseColorFactor", "value"),
                ("0", "baseColorFactor", "swatch"),
            ],
            PaletteShowLayout::Rows,
        );
        assert_eq!(
            output,
            "0.\"baseColorFactor\" #FF0000FF #00FF0080\n\
             \n\
             0.\"baseColorFactor\" \x1b[48;2;255;0;0m  \x1b[0m\x1b[48;2;0;255;0m  \x1b[0m\n"
        );
    }

    #[test]
    fn columns_stack_collections_under_labels() {
        let state = sample_state();
        let output = show(
            &state,
            &[
                ("0", "baseColorFactor.a", "value"),
                ("1", "baseColorFactor.a", "value"),
            ],
            PaletteShowLayout::Columns,
        );
        assert_eq!(
            output,
            "0.\"baseColorFactor\".a 1.\"baseColorFactor\".a\n255                   255\n128\n"
        );
    }

    #[test]
    fn columns_with_label_none_drop_the_label_row() {
        let state = sample_state();
        let grid = grid_for(
            &state,
            &[
                ("0", "baseColorFactor.a", "value"),
                ("1", "baseColorFactor.a", "value"),
            ],
        );
        let output = render(
            &grid,
            PaletteShowLayout::Columns,
            Some(PaletteShowLabel::None),
            None,
            None,
            Width::Unlimited,
        )
        .unwrap();
        assert_eq!(output, "255 255\n128\n");
    }

    #[test]
    fn hierarchy_layout_renders_the_palette_tree() {
        let state = sample_state();
        let output = show(&state, &[("0", "*", "value")], PaletteShowLayout::Hierarchy);
        assert_eq!(
            output,
            "└ 0\n  ├ \"baseColorFactor\": #FF0000FF #00FF0080\n  └ \"metallicFactor\": 1 0.2\n"
        );
    }

    #[test]
    fn header_labels_group_rows_under_palette_headings() {
        let state = sample_state();
        let grid = grid_for(&state, &[("*", "baseColorFactor", "value")]);
        let output = render(
            &grid,
            PaletteShowLayout::Rows,
            Some(PaletteShowLabel::Header),
            None,
            None,
            Width::Unlimited,
        )
        .unwrap();
        assert_eq!(
            output,
            "# 0\n\n\"baseColorFactor\" #FF0000FF #00FF0080\n\n# 1\n\n\"baseColorFactor\" #0000FFFF\n"
        );
    }

    #[test]
    fn a_header_level_shifts_the_headings() {
        let state = sample_state();
        let grid = grid_for(&state, &[("*", "baseColorFactor", "value")]);
        let output = render(
            &grid,
            PaletteShowLayout::Rows,
            Some(PaletteShowLabel::Header),
            NonZeroU8::new(2),
            None,
            Width::Unlimited,
        )
        .unwrap();
        assert!(output.starts_with("## 0\n"));
    }

    #[test]
    fn nested_tables_group_one_table_per_palette() {
        let state = sample_state();
        let output = show(
            &state,
            &[("*", "baseColorFactor", "value")],
            PaletteShowLayout::Tables,
        );
        assert_eq!(
            output,
            "# 0\n\
             \n\
             | #   | \"baseColorFactor\" |\n\
             | --- | ----------------- |\n\
             | 0   | #FF0000FF         |\n\
             | 1   | #00FF0080         |\n\
             \n\
             # 1\n\
             \n\
             | #   | \"baseColorFactor\" |\n\
             | --- | ----------------- |\n\
             | 0   | #0000FFFF         |\n"
        );
    }

    #[test]
    fn flat_tables_fill_one_aligned_comparison_table() {
        let state = sample_state();
        let grid = grid_for(&state, &[("*", "baseColorFactor", "value")]);
        let output = render(
            &grid,
            PaletteShowLayout::Tables,
            None,
            None,
            Some(PaletteShowTableShape::Flat),
            Width::Unlimited,
        )
        .unwrap();
        assert_eq!(
            output,
            "| #   | 0.\"baseColorFactor\" | 1.\"baseColorFactor\" |\n\
             | --- | ------------------- | ------------------- |\n\
             | 0   | #FF0000FF           | #0000FFFF           |\n\
             | 1   | #00FF0080           |                     |\n"
        );
    }

    #[test]
    fn records_tables_list_one_property_per_row() {
        let state = sample_state();
        let grid = grid_for(&state, &[("*", "baseColorFactor", "value")]);
        let output = render(
            &grid,
            PaletteShowLayout::Tables,
            None,
            None,
            Some(PaletteShowTableShape::Records),
            Width::Unlimited,
        )
        .unwrap();
        assert_eq!(
            output,
            "# 0\n\
             \n\
             | label             | value               |\n\
             | ----------------- | ------------------- |\n\
             | \"baseColorFactor\" | #FF0000FF #00FF0080 |\n\
             \n\
             # 1\n\
             \n\
             | label             | value     |\n\
             | ----------------- | --------- |\n\
             | \"baseColorFactor\" | #0000FFFF |\n"
        );
    }

    #[test]
    fn records_tables_add_a_column_per_component_path() {
        let state = sample_state();
        let grid = grid_for(
            &state,
            &[
                ("0", "baseColorFactor", "value"),
                ("0", "baseColorFactor.a", "value"),
            ],
        );
        let output = render(
            &grid,
            PaletteShowLayout::Tables,
            None,
            None,
            Some(PaletteShowTableShape::Records),
            Width::Unlimited,
        )
        .unwrap();
        assert_eq!(
            output,
            "# 0\n\
             \n\
             | label             | value               | a       |\n\
             | ----------------- | ------------------- | ------- |\n\
             | \"baseColorFactor\" | #FF0000FF #00FF0080 | 255 128 |\n"
        );
    }

    #[test]
    fn a_label_mode_on_the_hierarchy_layout_is_invalid_input() {
        let state = sample_state();
        let grid = grid_for(&state, &[("0", "baseColorFactor", "value")]);
        let result = render(
            &grid,
            PaletteShowLayout::Hierarchy,
            Some(PaletteShowLabel::Concat),
            None,
            None,
            Width::Unlimited,
        );
        assert!(result.is_err());
    }

    #[test]
    fn compact_json_nests_component_records_under_the_property() {
        let state = sample_state();
        let output = show(
            &state,
            &[
                ("0", "baseColorFactor", "value"),
                ("0", "baseColorFactor.a", "value"),
            ],
            PaletteShowLayout::JsonCompact,
        );
        assert_eq!(
            output,
            "[{\"label\":\"0\",\"children\":[{\"label\":\"baseColorFactor\",\
             \"values\":[\"#FF0000FF\",\"#00FF0080\"],\"children\":[\
             {\"label\":\"a\",\"values\":[255,128]}]}]}]\n"
        );
    }

    #[test]
    fn pretty_json_is_indented_and_matches_compact() {
        let state = sample_state();
        let fields: &[(&str, &str, &str)] = &[
            ("0", "baseColorFactor", "value"),
            ("0", "baseColorFactor.a", "value"),
        ];
        let pretty = show(&state, fields, PaletteShowLayout::JsonPretty);
        let compact = show(&state, fields, PaletteShowLayout::JsonCompact);
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
        let keys: Vec<&str> = collections.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(keys, ["baseColorFactor", "metallicFactor"]);
    }

    #[test]
    fn star_palette_skips_a_palette_lacking_a_named_property() {
        let state = sample_state();
        // Only palette 0 has `metallicFactor`; palette 1 is skipped, not an error.
        let collections =
            resolve_collections(&state, &selectors(&[("*", "metallicFactor", "value")])).unwrap();
        let labels: Vec<(usize, &str)> = collections
            .iter()
            .map(|c| (c.palette, c.key.as_str()))
            .collect();
        assert_eq!(labels, [(0, "metallicFactor")]);
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
            PaletteShowLayout::Rows,
        );
        assert_eq!(scalar, "0.\"metallicFactor\" 1 0.2\n");
        let component = show(
            &state,
            &[("0", "baseColorFactor.r", "auto")],
            PaletteShowLayout::Rows,
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
        let output = show(&state, &[("0", "tint", "value")], PaletteShowLayout::Rows);
        // Six hex digits, no alpha pair.
        assert_eq!(output, "0.\"tint\" #FF0000\n");
    }

    #[test]
    fn a_three_component_color_has_no_alpha_component() {
        let state = three_component_state();
        // `.a` is out of range on a three-component color, but `.r` reads.
        assert!(resolve_collections(&state, &selectors(&[("0", "tint.a", "value")])).is_err());
        let red = show(&state, &[("0", "tint.r", "value")], PaletteShowLayout::Rows);
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
            PaletteShowLayout::Rows,
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

        let output = show(&state, &[("0", "count", "value")], PaletteShowLayout::Rows);
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
        let text = show(&state, &[("0", "extra", "value")], PaletteShowLayout::Rows);
        assert_eq!(text, "0.\"extra\" [1,2]\n");
        let json = show(
            &state,
            &[("0", "extra", "value")],
            PaletteShowLayout::JsonCompact,
        );
        assert_eq!(
            json,
            "[{\"label\":\"0\",\"children\":[{\"label\":\"extra\",\"values\":[[1,2]]}]}]\n"
        );
    }

    #[test]
    fn an_empty_property_name_is_quoted_in_the_label_but_raw_in_json() {
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
        let row = show(&state, &[("0", "*", "value")], PaletteShowLayout::Rows);
        assert_eq!(row, "0.\"\" true\n");
        // JSON keeps the raw name; its own string quoting is enough there.
        let json = show(
            &state,
            &[("0", "*", "value")],
            PaletteShowLayout::JsonCompact,
        );
        assert_eq!(
            json,
            "[{\"label\":\"0\",\"children\":[{\"label\":\"\",\"values\":[true]}]}]\n"
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
            PaletteShowLayout::Rows,
        );
        assert_eq!(output, "0.\"emissiveStrength\" (scalar) 2\n");
    }

    #[test]
    fn star_property_expands_scalar_properties_after_array_ones() {
        let state = scalar_state();
        let collections = resolve_collections(&state, &selectors(&[("0", "*", "value")])).unwrap();
        let keys: Vec<(&str, bool)> = collections
            .iter()
            .map(|c| (c.key.as_str(), c.scalar))
            .collect();
        assert_eq!(
            keys,
            [
                ("baseColorFactor", false),
                ("emissiveStrength", true),
                ("tint", true)
            ]
        );
    }

    #[test]
    fn a_scalar_color_reads_like_a_material_value() {
        let state = scalar_state();
        let whole = show(&state, &[("0", "tint", "value")], PaletteShowLayout::Rows);
        assert_eq!(whole, "0.\"tint\" (scalar) #00FF0080\n");
        let component = show(&state, &[("0", "tint.a", "value")], PaletteShowLayout::Rows);
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
    fn scalar_json_records_carry_the_annotation() {
        let state = scalar_state();
        let output = show(
            &state,
            &[
                ("0", "baseColorFactor", "value"),
                ("0", "emissiveStrength", "value"),
            ],
            PaletteShowLayout::JsonCompact,
        );
        assert_eq!(
            output,
            "[{\"label\":\"0\",\"children\":[\
             {\"label\":\"baseColorFactor\",\"values\":[\"#FF0000FF\"]},\
             {\"label\":\"emissiveStrength\",\"annotation\":\"(scalar)\",\"values\":[2]}]}]\n"
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

        let output = show(&state, &[("0", "*", "value")], PaletteShowLayout::Rows);
        assert_eq!(output, "0.\"emissiveStrength\" (scalar) 0.5\n");
    }
}
