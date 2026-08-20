use crate::{
    Format, Result, VectorComponent, Width,
    commands::{
        PaletteRef, PaletteShowLabel, PaletteShowLayout, PaletteShowPresentation,
        PaletteShowReading, PaletteShowTableShape, PropertyRef, PropertySelector,
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
use ty_math::{TyLinSrgbF64, TyLinSrgbaF64, TySrgbF64, TySrgbaF64};
use voxcore::{
    BVoxValuePoolValue, VoxMain, VoxPalette, VoxValue, VoxValuePool, VoxValuePoolKind,
    VoxValuePoolValueRef,
};
use voxsmith::GltfPropertyKind;

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
    let value_collections = resolve_value_collections(&state, selectors)?;
    let grid = build_grid(value_collections);
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

/// One resolved value collection: a property's values down one palette.
struct ValueCollection {
    /// The resolved palette index, even when the selector used `*`.
    palette_index: usize,
    /// The property key, without any component.
    key: String,
    /// The vector component read from the property, when one was given.
    component: Option<VectorComponent>,
    /// What renders for each value.
    presentation: PaletteShowPresentation,
    /// One sample per palette material in material order.
    samples: Vec<TreeGridJsonValue>,
}

/// A selector's reading with `auto` resolved to the key's default.
#[derive(Clone, Copy)]
enum Reading {
    LinearFloat,
    Plain,
    SrgbFloat,
    SrgbHex,
}

/// Resolves the selectors against the document's palettes into value
/// collections in render order: selector order, then palette order, then
/// property order. A `*` palette or `*` property expands to one value
/// collection per match; a named palette or property that is absent is an
/// error, while a `*` palette quietly skips a palette that lacks a named
/// property.
fn resolve_value_collections(
    state: &VoxMain,
    selectors: &[PropertySelector],
) -> Result<Vec<ValueCollection>> {
    let palettes: Vec<&VoxPalette> = state.iter_palettes().map(|(_, palette)| palette).collect();
    let mut value_collections = Vec::new();
    for selector in selectors {
        match selector.palette {
            PaletteRef::All => {
                for (palette_index, palette) in palettes.iter().enumerate() {
                    value_collections.extend(expand_property(
                        state,
                        palette_index,
                        palette,
                        selector,
                        true,
                    )?);
                }
            }
            PaletteRef::Index(palette_index) => {
                let palette = palettes.get(palette_index).ok_or_else(|| {
                    IOError::new(
                        ErrorKind::InvalidInput,
                        format!(
                            "palette index {palette_index} is out of range; the document has {} palette(s)",
                            palettes.len()
                        ),
                    )
                })?;
                value_collections.extend(expand_property(
                    state,
                    palette_index,
                    palette,
                    selector,
                    false,
                )?);
            }
        }
    }
    Ok(value_collections)
}

/// Expands one selector's property against one palette into its value
/// collections. A `palette_is_wild` palette, one from a `*`, skips a named
/// property it lacks instead of erroring.
fn expand_property(
    state: &VoxMain,
    palette_index: usize,
    palette: &VoxPalette,
    selector: &PropertySelector,
    palette_is_wild: bool,
) -> Result<Vec<ValueCollection>> {
    match &selector.property {
        PropertyRef::All => implementation::property_names(palette)
            .into_iter()
            .map(|name| {
                build_value_collection(
                    state,
                    palette_index,
                    palette,
                    name,
                    None,
                    selector.presentation,
                    selector.reading,
                )
            })
            .collect(),
        PropertyRef::Key { key, component } => {
            if palette.property_id_by_name(key).is_none() {
                if palette_is_wild {
                    return Ok(Vec::new());
                }
                return Err(IOError::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "palette {palette_index} has no property `{key}`; available properties: {}",
                        available_keys(palette)
                    ),
                )
                .into());
            }
            Ok(vec![build_value_collection(
                state,
                palette_index,
                palette,
                key,
                *component,
                selector.presentation,
                selector.reading,
            )?])
        }
    }
}

/// Builds one value collection from a property the caller verified present.
fn build_value_collection(
    state: &VoxMain,
    palette_index: usize,
    palette: &VoxPalette,
    key: &str,
    component: Option<VectorComponent>,
    presentation: PaletteShowPresentation,
    reading: PaletteShowReading,
) -> Result<ValueCollection> {
    let property_id = palette
        .property_id_by_name(key)
        .expect("caller verified the property is present");
    let value_pool_id = palette
        .property(property_id)
        .expect("a property id from this palette resolves")
        .value_pool_id;
    let value_pool = state
        .value_pool(value_pool_id)
        .expect("a property references a value pool the state holds");

    // A component is shape addressing: legal on any vector value pool whose
    // width exceeds its index, whatever the reading.
    if let Some(component) = component {
        match vector_width(value_pool) {
            None => {
                return Err(IOError::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "property `{key}` is not a vector and has no `.{}` component",
                        component.letter()
                    ),
                )
                .into());
            }
            Some(width) if component.index() >= width => {
                return Err(IOError::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "property `{key}` is {width} components wide and has no `.{}` component",
                        component.letter()
                    ),
                )
                .into());
            }
            Some(_) => {}
        }
    }

    let reading = resolve_reading(key, value_pool, reading)?;

    // A material holds one value id per property, so the lookup with
    // this palette's own property id always resolves.
    let samples = palette
        .iter_materials()
        .map(|material_id| {
            let value_id = palette
                .value_id(material_id, property_id)
                .expect("a material holds a value for every property");
            sample(key, value_pool, value_id, reading, component)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(ValueCollection {
        palette_index,
        key: key.to_string(),
        component,
        presentation,
        samples,
    })
}

/// The width of the value pool's vector values, or `None` for a non-vector
/// kind.
fn vector_width(value_pool: &VoxValuePool) -> Option<usize> {
    match value_pool.kind() {
        VoxValuePoolKind::Bool(_)
        | VoxValuePoolKind::Float(_)
        | VoxValuePoolKind::Int(_)
        | VoxValuePoolKind::Json(_)
        | VoxValuePoolKind::String(_) => None,
        VoxValuePoolKind::Vec2Float(_) | VoxValuePoolKind::Vec2Int(_) => Some(2),
        VoxValuePoolKind::Vec3Float(_) | VoxValuePoolKind::Vec3Int(_) => Some(3),
        VoxValuePoolKind::Vec4Float(_) | VoxValuePoolKind::Vec4Int(_) => Some(4),
    }
}

/// Whether the value pool holds a color-shaped value, three or four float
/// components.
fn color_shape(value_pool: &VoxValuePool) -> bool {
    matches!(
        value_pool.kind(),
        VoxValuePoolKind::Vec3Float(_) | VoxValuePoolKind::Vec4Float(_)
    )
}

/// Resolves a selector's reading for the property `key`. The three color
/// readings are the color assertion and require a color-shaped value pool.
/// `auto` interprets a known glTF property by its vocabulary kind: a color
/// name reads `srgb-hex` and errors off a color shape, and the reading's
/// range rule holds an out-of-range color to an explicit `linear-float` or
/// `plain`. Everything else assumes `plain`.
fn resolve_reading(
    key: &str,
    value_pool: &VoxValuePool,
    reading: PaletteShowReading,
) -> Result<Reading> {
    match reading {
        PaletteShowReading::Auto => match GltfPropertyKind::of(key) {
            Some(GltfPropertyKind::ColorRgb | GltfPropertyKind::ColorRgba) => {
                if !color_shape(value_pool) {
                    return Err(IOError::new(
                        ErrorKind::InvalidInput,
                        format!(
                            "property `{key}` names a glTF color and must bind a vec-3-float or \
                             vec-4-float value pool"
                        ),
                    )
                    .into());
                }
                Ok(Reading::SrgbHex)
            }
            Some(GltfPropertyKind::Scalar) | None => Ok(Reading::Plain),
        },
        PaletteShowReading::LinearFloat => {
            color_reading(key, value_pool, "linear-float", Reading::LinearFloat)
        }
        PaletteShowReading::Plain => Ok(Reading::Plain),
        PaletteShowReading::SrgbFloat => {
            color_reading(key, value_pool, "srgb-float", Reading::SrgbFloat)
        }
        PaletteShowReading::SrgbHex => color_reading(key, value_pool, "srgb-hex", Reading::SrgbHex),
    }
}

/// Admits a color reading on a color-shaped value pool and errors on every
/// other kind.
fn color_reading(
    key: &str,
    value_pool: &VoxValuePool,
    name: &str,
    reading: Reading,
) -> Result<Reading> {
    if color_shape(value_pool) {
        Ok(reading)
    } else {
        Err(IOError::new(
            ErrorKind::InvalidInput,
            format!(
                "property `{key}` does not bind a vec-3-float or vec-4-float value pool, which \
                 the `{name}` reading requires"
            ),
        )
        .into())
    }
}

/// The sample for the value at `value_id` under the resolved `reading` and an
/// optional vector `component`. An sRGB reading errors on a spelled component
/// outside `[0, 1]`.
fn sample(
    key: &str,
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
    reading: Reading,
    component: Option<VectorComponent>,
) -> Result<TreeGridJsonValue> {
    match component {
        Some(component) => sample_component(key, value_pool, value_id, reading, component),
        None => sample_whole(key, value_pool, value_id, reading),
    }
}

/// The sample for a whole value under `reading`. The color readings spell the
/// stored linear color. `plain` spells any value as it is.
fn sample_whole(
    key: &str,
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
    reading: Reading,
) -> Result<TreeGridJsonValue> {
    match reading {
        Reading::LinearFloat => {
            let floats = color_floats(value_pool, value_id);
            Ok(match floats[..] {
                [r, g, b] => TreeGridJsonValue::lin_srgb(TyLinSrgbF64::new(r, g, b)),
                [r, g, b, a] => TreeGridJsonValue::lin_srgba(TyLinSrgbaF64::new(r, g, b, a)),
                _ => unreachable!("a color reading resolves only on a color shape"),
            })
        }
        Reading::Plain => Ok(sample_plain(value_pool, value_id)),
        Reading::SrgbFloat => {
            let floats = color_floats(value_pool, value_id);
            require_unit(key, &floats)?;
            // The rgb components spell transfer-encoded. Alpha passes through.
            let encode = |component| round_six_decimals(encode_component(component));
            Ok(match floats[..] {
                [r, g, b] => {
                    TreeGridJsonValue::srgb(TySrgbF64::new(encode(r), encode(g), encode(b)))
                }
                [r, g, b, a] => {
                    TreeGridJsonValue::srgba(TySrgbaF64::new(encode(r), encode(g), encode(b), a))
                }
                _ => unreachable!("a color reading resolves only on a color shape"),
            })
        }
        Reading::SrgbHex => {
            let floats = color_floats(value_pool, value_id);
            require_unit(key, &floats)?;
            let bytes = color_bytes(value_pool, value_id);
            Ok(if floats.len() == 4 {
                TreeGridJsonValue::srgba8(bytes)
            } else {
                TreeGridJsonValue::srgb8([bytes[0], bytes[1], bytes[2]])
            })
        }
    }
}

/// The sample for one component under `reading`. A color reading's grayscale
/// swatch is the channel's sRGB appearance, the same byte the whole-vector
/// spelling carries. `plain` maps the stored number onto the raw ramp.
fn sample_component(
    key: &str,
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
    reading: Reading,
    component: VectorComponent,
) -> Result<TreeGridJsonValue> {
    let index = component.index();
    match reading {
        Reading::LinearFloat => {
            let stored = color_floats(value_pool, value_id)[index];
            let byte = color_bytes(value_pool, value_id)[index];
            Ok(TreeGridJsonValue::float(stored).with_swatch(TreeGridSwatch::Gray(byte)))
        }
        Reading::Plain => Ok(sample_plain_component(value_pool, value_id, index)),
        Reading::SrgbFloat => {
            let stored = color_floats(value_pool, value_id)[index];
            require_unit(key, &[stored])?;
            // The rgb components spell transfer-encoded. Alpha passes through.
            let spelled = if index < 3 {
                round_six_decimals(encode_component(stored))
            } else {
                stored
            };
            let byte = color_bytes(value_pool, value_id)[index];
            Ok(TreeGridJsonValue::float(spelled).with_swatch(TreeGridSwatch::Gray(byte)))
        }
        Reading::SrgbHex => {
            let stored = color_floats(value_pool, value_id)[index];
            require_unit(key, &[stored])?;
            let byte = color_bytes(value_pool, value_id)[index];
            Ok(TreeGridJsonValue::new(format!("{byte:02X}"))
                .with_swatch(TreeGridSwatch::Gray(byte)))
        }
    }
}

/// The `plain` sample for a whole value: the stored value as it is, with a
/// `float` or `int` number on the grayscale ramp.
fn sample_plain(
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
) -> TreeGridJsonValue {
    let value = value_pool
        .value(value_id)
        .expect("a material draws a retained value");
    match value {
        VoxValuePoolValueRef::Bool(flag) => TreeGridJsonValue::bool(flag),
        VoxValuePoolValueRef::Float(number) => TreeGridJsonValue::unorm(number),
        VoxValuePoolValueRef::Int(number) => TreeGridJsonValue::unorm(number as f64),
        VoxValuePoolValueRef::Json(value) => TreeGridJsonValue::json(vox_value_to_json(value)),
        VoxValuePoolValueRef::String(text) => TreeGridJsonValue::new(text.to_owned()),
        VoxValuePoolValueRef::Vec2Float(vector) => float_array_json(vector),
        VoxValuePoolValueRef::Vec2Int(vector) => int_array_json(vector),
        VoxValuePoolValueRef::Vec3Float(vector) => float_array_json(vector),
        VoxValuePoolValueRef::Vec3Int(vector) => int_array_json(vector),
        VoxValuePoolValueRef::Vec4Float(vector) => float_array_json(vector),
        VoxValuePoolValueRef::Vec4Int(vector) => int_array_json(vector),
    }
}

/// The `plain` sample for one vector component: the stored number on the
/// grayscale ramp.
fn sample_plain_component(
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
    index: usize,
) -> TreeGridJsonValue {
    let value = value_pool
        .value(value_id)
        .expect("a material draws a retained value");
    match value {
        VoxValuePoolValueRef::Vec2Float(vector) => TreeGridJsonValue::unorm(vector[index]),
        VoxValuePoolValueRef::Vec2Int(vector) => TreeGridJsonValue::unorm(vector[index] as f64),
        VoxValuePoolValueRef::Vec3Float(vector) => TreeGridJsonValue::unorm(vector[index]),
        VoxValuePoolValueRef::Vec3Int(vector) => TreeGridJsonValue::unorm(vector[index] as f64),
        VoxValuePoolValueRef::Vec4Float(vector) => TreeGridJsonValue::unorm(vector[index]),
        VoxValuePoolValueRef::Vec4Int(vector) => TreeGridJsonValue::unorm(vector[index] as f64),
        _ => unreachable!("a component was validated against a vector shape"),
    }
}

/// Requires every component an sRGB reading would spell to lie in `[0, 1]`:
/// the transfer is defined there, and a byte cannot spell values outside it.
fn require_unit(key: &str, components: &[f64]) -> Result<()> {
    if components
        .iter()
        .all(|component| (0.0..=1.0).contains(component))
    {
        Ok(())
    } else {
        Err(IOError::new(
            ErrorKind::InvalidInput,
            format!(
                "property `{key}` holds a component outside [0, 1], which an sRGB reading \
                 cannot spell; read it as `linear-float` or `plain`"
            ),
        )
        .into())
    }
}

/// Transfer-encodes one linear rgb component to its sRGB spelling.
fn encode_component(component: f64) -> f64 {
    TySrgbF64::from_linear(TyLinSrgbF64::new(component, 0.0, 0.0)).red
}

/// Rounds an encoded component to six decimal places, the display precision
/// of the `srgb-float` reading.
fn round_six_decimals(value: f64) -> f64 {
    (value * 1e6).round() / 1e6
}

/// A float vector as a JSON array, each component spelled as a number.
fn float_array_json(vector: &[f64]) -> TreeGridJsonValue {
    TreeGridJsonValue::json(Value::Array(
        vector
            .iter()
            .map(|&component| number_json(component))
            .collect(),
    ))
}

/// An int vector as a JSON array.
fn int_array_json(vector: &[i64]) -> TreeGridJsonValue {
    TreeGridJsonValue::json(Value::Array(
        vector.iter().map(|&component| json!(component)).collect(),
    ))
}

/// The `[r, g, b, a]` bytes for the stored linear color at `value_id`,
/// encoded to sRGB at display. The alpha only quantizes, and a
/// three-component color takes opaque alpha.
fn color_bytes(value_pool: &VoxValuePool, value_id: U32Id<BVoxValuePoolValue>) -> [u8; 4] {
    let [r, g, b, a] = match color_floats(value_pool, value_id)[..] {
        [r, g, b] => [r, g, b, 1.0],
        [r, g, b, a] => [r, g, b, a],
        _ => unreachable!("a color reading resolves only on a color shape"),
    };
    <[u8; 4]>::from(TySrgbaF64::from_linear(TyLinSrgbaF64::new(r, g, b, a)).into_format::<u8, u8>())
}

/// The stored linear float components of the color at `value_id`, three or
/// four long by shape.
fn color_floats(value_pool: &VoxValuePool, value_id: U32Id<BVoxValuePoolValue>) -> Vec<f64> {
    let value = value_pool
        .value(value_id)
        .expect("a material draws a retained value");
    match value {
        VoxValuePoolValueRef::Vec3Float(color) => color.to_vec(),
        VoxValuePoolValueRef::Vec4Float(color) => color.to_vec(),
        _ => unreachable!("a color reading resolves only on a color shape"),
    }
}

/// A number as JSON: an integer when it is integral and fits `i64`, else a
/// float, so it reads as it does in the text layouts. JSON spells no infinity
/// and serde_json writes one as `null`, so an infinite value carries the
/// sentinel the wire spells it with.
fn number_json(value: f64) -> Value {
    if value == f64::INFINITY {
        Value::String("inf".to_owned())
    } else if value == f64::NEG_INFINITY {
        Value::String("-inf".to_owned())
    } else if value.fract() == 0.0 && value.abs() < i64::MAX as f64 {
        json!(value as i64)
    } else {
        json!(value)
    }
}

/// A [`VoxValue`] from a `json` value pool as a [`serde_json::Value`].
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

/// Populates a tree grid from the value collections in order: palette root,
/// property child, component leaf, with each value collection's samples on
/// its deepest node. A value collection reuses the immediately preceding
/// value collection's palette and property nodes when they match, so a
/// contiguous run shares its ancestors and pre-order keeps the selector
/// order in every layout.
fn build_grid(value_collections: Vec<ValueCollection>) -> TreeGrid<TreeGridJsonValueCells> {
    let mut grid = TreeGrid::with_cells(TreeGridJsonValueCells);
    let mut palette_node: Option<(usize, U32Id<BTreeGridNode>)> = None;
    let mut property_node: Option<(String, U32Id<BTreeGridNode>)> = None;
    for value_collection in value_collections {
        let palette_node_id = match palette_node {
            Some((palette_index, node_id)) if palette_index == value_collection.palette_index => {
                node_id
            }
            _ => {
                let node_id = grid.add_root(TreeGridLabel::bare(
                    value_collection.palette_index.to_string(),
                ));
                palette_node = Some((value_collection.palette_index, node_id));
                property_node = None;
                node_id
            }
        };
        let data_node_id = match value_collection.component {
            Some(component) => {
                let property_node_id = match &property_node {
                    Some((key, node_id)) if *key == value_collection.key => *node_id,
                    _ => grid.add_child(
                        palette_node_id,
                        TreeGridLabel::quoted(value_collection.key.as_str()),
                    ),
                };
                property_node = Some((value_collection.key, property_node_id));
                let letter = component.letter().to_string();
                grid.add_child(property_node_id, TreeGridLabel::bare(letter))
            }
            None => {
                // A data node is always fresh, so a property selected twice
                // keeps one value collection per selector.
                let node_id = grid.add_child(
                    palette_node_id,
                    TreeGridLabel::quoted(value_collection.key.as_str()),
                );
                property_node = Some((value_collection.key, node_id));
                node_id
            }
        };
        let node = grid.node_mut(data_node_id);
        node.format = cell_format(value_collection.presentation);
        node.values = value_collection.samples;
    }
    grid
}

/// The node cell format a `--property` presentation maps to. `auto` leaves
/// the format unset so the grid's cell policy decides per value.
fn cell_format(presentation: PaletteShowPresentation) -> Option<TreeGridCellFormat> {
    match presentation {
        PaletteShowPresentation::Auto => None,
        PaletteShowPresentation::Swatch => Some(TreeGridCellFormat::Visual),
        PaletteShowPresentation::SwatchValue => Some(TreeGridCellFormat::VisualText),
        PaletteShowPresentation::Value => Some(TreeGridCellFormat::Text),
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
        implementation::palette_show::{build_grid, render, resolve_value_collections},
    };
    use branded_id::U32Id;
    use serde_json::Value;
    use std::num::NonZeroU8;
    use treegrid::{TreeGrid, TreeGridJsonValueCells};
    use ty_math::TySrgbaU8;
    use voxcore::{BVoxValuePool, BVoxValuePoolValue, VoxMain, VoxPalette, VoxValue, VoxValuePool};

    /// The branded value id `index`.
    fn value_id(index: usize) -> U32Id<BVoxValuePoolValue> {
        U32Id::from_u32(index as u32)
    }

    /// A `vec-4-float` value pool of the given 8-bit sRGB colors, each decoded
    /// to linear light, the way the importers store colors.
    fn lin_srgba_f64_value_pool(state: &mut VoxMain, colors: &[[u8; 4]]) -> U32Id<BVoxValuePool> {
        let values = colors
            .iter()
            .map(|&[red, green, blue, alpha]| {
                let linear = TySrgbaU8::new(red, green, blue, alpha)
                    .into_format::<f64, f64>()
                    .into_linear();
                [linear.red, linear.green, linear.blue, linear.alpha]
            })
            .collect();
        state.retain_value_pool(VoxValuePool::vec_4_float(values).unwrap())
    }

    /// A document with two palettes: palette 0 has `baseColor` and
    /// `metallic` with two materials, palette 1 has `baseColor` with
    /// one material.
    fn sample_state() -> VoxMain {
        let mut state = VoxMain::default();

        let colors_zero_value_pool_id =
            lin_srgba_f64_value_pool(&mut state, &[[255, 0, 0, 255], [0, 255, 0, 128]]);
        let metallic_value_pool_id =
            state.retain_value_pool(VoxValuePool::float(vec![1.0, 0.2]).unwrap());
        let colors_one_value_pool_id = lin_srgba_f64_value_pool(&mut state, &[[0, 0, 255, 255]]);

        let mut first = VoxPalette::default();
        first
            .retain_property(
                "baseColor".to_owned(),
                colors_zero_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        first
            .retain_property(
                "metallic".to_owned(),
                metallic_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        first
            .retain_material(vec![value_id(0), value_id(0)])
            .unwrap();
        first
            .retain_material(vec![value_id(1), value_id(1)])
            .unwrap();
        state.retain_palette(first).unwrap();

        let mut second = VoxPalette::default();
        second
            .retain_property(
                "baseColor".to_owned(),
                colors_one_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        second.retain_material(vec![value_id(0)]).unwrap();
        state.retain_palette(second).unwrap();

        state
    }

    fn selectors(fields: &[(&str, &str, &str, &str)]) -> Vec<PropertySelector> {
        fields
            .iter()
            .map(|(palette, property, presentation, reading)| {
                PropertySelector::parse(palette, property, presentation, reading).unwrap()
            })
            .collect()
    }

    /// The populated grid for the selectors, resolved against `state`.
    fn grid_for(
        state: &VoxMain,
        fields: &[(&str, &str, &str, &str)],
    ) -> TreeGrid<TreeGridJsonValueCells> {
        build_grid(resolve_value_collections(state, &selectors(fields)).unwrap())
    }

    /// Renders the selectors under `layout` with default label options and no
    /// wrapping.
    fn show(
        state: &VoxMain,
        fields: &[(&str, &str, &str, &str)],
        layout: PaletteShowLayout,
    ) -> String {
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
    fn value_presentation_prints_canonical_hex_with_a_label() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "baseColor", "value", "auto")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(output, "0.\"baseColor\" #FF0000FF #00FF0080\n");
    }

    #[test]
    fn a_color_component_under_auto_spells_its_hex_pair() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "baseColor.a", "value", "auto")],
            PaletteShowLayout::Rows,
        );
        // A vocabulary color name reads srgb-hex, so the alpha bytes FF and
        // 80 spell their hex pairs.
        assert_eq!(output, "0.\"baseColor\".a FF 80\n");
    }

    #[test]
    fn swatch_presentation_abuts_swatches_into_a_strip() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "baseColor", "swatch", "auto")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(
            output,
            "0.\"baseColor\" \x1b[48;2;255;0;0m  \x1b[0m\x1b[48;2;0;255;0m  \x1b[0m\n"
        );
    }

    #[test]
    fn swatch_spaces_values_with_no_swatch() {
        let mut state = VoxMain::default();
        let shadows_value_pool_id =
            state.retain_value_pool(VoxValuePool::boolean(vec![true, false]));
        let mut palette = VoxPalette::default();
        palette
            .retain_property(
                "shadows".to_owned(),
                shadows_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        palette.retain_material(vec![value_id(1)]).unwrap();
        state.retain_palette(palette).unwrap();

        // Bools have no swatch, so swatch format spaces them rather than
        // abutting them into `truefalse`.
        let output = show(
            &state,
            &[("0", "shadows", "swatch", "auto")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(output, "0.\"shadows\" true false\n");
    }

    #[test]
    fn rows_pad_only_the_label_not_the_values() {
        let state = sample_state();
        // The labels pad to the longest so each row's first value aligns, but
        // the values are not column-aligned: `metallic` stays compact
        // rather than padding out to the wider `baseColor` columns.
        let output = show(
            &state,
            &[
                ("0", "baseColor", "value", "auto"),
                ("0", "metallic", "value", "auto"),
            ],
            PaletteShowLayout::Rows,
        );
        assert_eq!(
            output,
            "0.\"baseColor\" #FF0000FF #00FF0080\n\
             \n\
             0.\"metallic\"  1 0.2\n"
        );
    }

    #[test]
    fn rows_wrap_cells_to_the_width() {
        let state = sample_state();
        let grid = grid_for(&state, &[("0", "baseColor", "value", "auto")]);
        // Width 30 leaves 16 columns after the `0."baseColor" ` prefix:
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
            "0.\"baseColor\" #FF0000FF\n              #00FF0080\n"
        );
    }

    #[test]
    fn rows_with_label_none_drop_the_label_column() {
        let state = sample_state();
        let grid = grid_for(&state, &[("0", "baseColor", "value", "auto")]);
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
        let value_collections =
            resolve_value_collections(&state, &[PropertySelector::default_all_auto()]).unwrap();
        let output = render(
            &build_grid(value_collections),
            PaletteShowLayout::Rows,
            None,
            None,
            None,
            Width::Unlimited,
        )
        .unwrap();
        assert_eq!(
            output,
            "0.\"baseColor\" \x1b[48;2;255;0;0m  \x1b[0m #FF0000FF \x1b[48;2;0;255;0m  \x1b[0m #00FF0080\n\
             \n\
             0.\"metallic\"  1 0.2\n\
             \n\
             1.\"baseColor\" \x1b[48;2;0;0;255m  \x1b[0m #0000FFFF\n"
        );
    }

    #[test]
    fn value_collections_render_in_selector_order() {
        let state = sample_state();
        // A palette revisited later starts a fresh root rather than merging
        // backward, so pre-order keeps the selector order.
        let output = show(
            &state,
            &[
                ("1", "baseColor", "value", "auto"),
                ("0", "baseColor", "value", "auto"),
            ],
            PaletteShowLayout::Rows,
        );
        assert_eq!(
            output,
            "1.\"baseColor\" #0000FFFF\n\
             \n\
             0.\"baseColor\" #FF0000FF #00FF0080\n"
        );
    }

    #[test]
    fn a_repeated_property_keeps_one_row_per_selector() {
        let state = sample_state();
        let output = show(
            &state,
            &[
                ("0", "baseColor", "value", "auto"),
                ("0", "baseColor", "swatch", "auto"),
            ],
            PaletteShowLayout::Rows,
        );
        assert_eq!(
            output,
            "0.\"baseColor\" #FF0000FF #00FF0080\n\
             \n\
             0.\"baseColor\" \x1b[48;2;255;0;0m  \x1b[0m\x1b[48;2;0;255;0m  \x1b[0m\n"
        );
    }

    #[test]
    fn columns_stack_value_collections_under_labels() {
        let state = sample_state();
        let output = show(
            &state,
            &[
                ("0", "baseColor.a", "value", "auto"),
                ("1", "baseColor.a", "value", "auto"),
            ],
            PaletteShowLayout::Columns,
        );
        assert_eq!(
            output,
            "0.\"baseColor\".a 1.\"baseColor\".a\nFF              FF\n80\n"
        );
    }

    #[test]
    fn columns_with_label_none_drop_the_label_row() {
        let state = sample_state();
        let grid = grid_for(
            &state,
            &[
                ("0", "baseColor.a", "value", "auto"),
                ("1", "baseColor.a", "value", "auto"),
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
        assert_eq!(output, "FF FF\n80\n");
    }

    #[test]
    fn hierarchy_layout_renders_the_palette_tree() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "*", "value", "auto")],
            PaletteShowLayout::Hierarchy,
        );
        assert_eq!(
            output,
            "└ 0\n  ├ \"baseColor\": #FF0000FF #00FF0080\n  └ \"metallic\": 1 0.2\n"
        );
    }

    #[test]
    fn header_labels_group_rows_under_palette_headings() {
        let state = sample_state();
        let grid = grid_for(&state, &[("*", "baseColor", "value", "auto")]);
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
            "# 0\n\n\"baseColor\" #FF0000FF #00FF0080\n\n# 1\n\n\"baseColor\" #0000FFFF\n"
        );
    }

    #[test]
    fn a_header_level_shifts_the_headings() {
        let state = sample_state();
        let grid = grid_for(&state, &[("*", "baseColor", "value", "auto")]);
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
            &[("*", "baseColor", "value", "auto")],
            PaletteShowLayout::Tables,
        );
        assert_eq!(
            output,
            "# 0\n\
             \n\
             | #   | \"baseColor\" |\n\
             | --- | ----------- |\n\
             | 0   | #FF0000FF   |\n\
             | 1   | #00FF0080   |\n\
             \n\
             # 1\n\
             \n\
             | #   | \"baseColor\" |\n\
             | --- | ----------- |\n\
             | 0   | #0000FFFF   |\n"
        );
    }

    #[test]
    fn flat_tables_fill_one_aligned_comparison_table() {
        let state = sample_state();
        let grid = grid_for(&state, &[("*", "baseColor", "value", "auto")]);
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
            "| #   | 0.\"baseColor\" | 1.\"baseColor\" |\n\
             | --- | ------------- | ------------- |\n\
             | 0   | #FF0000FF     | #0000FFFF     |\n\
             | 1   | #00FF0080     |               |\n"
        );
    }

    #[test]
    fn records_tables_list_one_property_per_row() {
        let state = sample_state();
        let grid = grid_for(&state, &[("*", "baseColor", "value", "auto")]);
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
             | label       | value               |\n\
             | ----------- | ------------------- |\n\
             | \"baseColor\" | #FF0000FF #00FF0080 |\n\
             \n\
             # 1\n\
             \n\
             | label       | value     |\n\
             | ----------- | --------- |\n\
             | \"baseColor\" | #0000FFFF |\n"
        );
    }

    #[test]
    fn records_tables_add_a_column_per_component_path() {
        let state = sample_state();
        let grid = grid_for(
            &state,
            &[
                ("0", "baseColor", "value", "auto"),
                ("0", "baseColor.a", "value", "auto"),
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
             | label       | value               | a     |\n\
             | ----------- | ------------------- | ----- |\n\
             | \"baseColor\" | #FF0000FF #00FF0080 | FF 80 |\n"
        );
    }

    #[test]
    fn a_label_mode_on_the_hierarchy_layout_is_invalid_input() {
        let state = sample_state();
        let grid = grid_for(&state, &[("0", "baseColor", "value", "auto")]);
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
                ("0", "baseColor", "value", "auto"),
                ("0", "baseColor.a", "value", "auto"),
            ],
            PaletteShowLayout::JsonCompact,
        );
        assert_eq!(
            output,
            "[{\"label\":\"0\",\"children\":[{\"label\":\"baseColor\",\
             \"values\":[\"#FF0000FF\",\"#00FF0080\"],\"children\":[\
             {\"label\":\"a\",\"values\":[\"FF\",\"80\"]}]}]}]\n"
        );
    }

    #[test]
    fn pretty_json_is_indented_and_matches_compact() {
        let state = sample_state();
        let fields: &[(&str, &str, &str, &str)] = &[
            ("0", "baseColor", "value", "auto"),
            ("0", "baseColor.a", "value", "auto"),
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
        let value_collections =
            resolve_value_collections(&state, &selectors(&[("0", "*", "value", "auto")])).unwrap();
        let keys: Vec<&str> = value_collections.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(keys, ["baseColor", "metallic"]);
    }

    #[test]
    fn star_palette_skips_a_palette_lacking_a_named_property() {
        let state = sample_state();
        // Only palette 0 has `metallic`; palette 1 is skipped, not an error.
        let value_collections =
            resolve_value_collections(&state, &selectors(&[("*", "metallic", "value", "auto")]))
                .unwrap();
        let labels: Vec<(usize, &str)> = value_collections
            .iter()
            .map(|c| (c.palette_index, c.key.as_str()))
            .collect();
        assert_eq!(labels, [(0, "metallic")]);
    }

    #[test]
    fn named_palette_out_of_range_is_an_error() {
        let state = sample_state();
        assert!(
            resolve_value_collections(&state, &selectors(&[("5", "baseColor", "value", "auto")]))
                .is_err()
        );
    }

    #[test]
    fn named_property_absent_on_a_named_palette_is_an_error() {
        let state = sample_state();
        assert!(
            resolve_value_collections(&state, &selectors(&[("0", "missing", "value", "auto")]))
                .is_err()
        );
    }

    #[test]
    fn a_component_on_a_scalar_is_an_error() {
        let state = sample_state();
        assert!(
            resolve_value_collections(&state, &selectors(&[("0", "metallic.r", "value", "auto")]))
                .is_err()
        );
    }

    #[test]
    fn auto_presentation_prints_scalars_and_components_as_bare_text() {
        let state = sample_state();
        let scalar = show(
            &state,
            &[("0", "metallic", "auto", "auto")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(scalar, "0.\"metallic\" 1 0.2\n");
        let component = show(
            &state,
            &[("0", "baseColor.r", "auto", "auto")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(component, "0.\"baseColor\".r FF 00\n");
    }

    /// One palette binding `emissiveColor`, three components per the glTF
    /// vocabulary, to a `vec-3-float` value pool holding a red.
    fn three_component_state() -> VoxMain {
        let mut state = VoxMain::default();
        let emissive_value_pool_id =
            state.retain_value_pool(VoxValuePool::vec_3_float(vec![[1.0, 0.0, 0.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property(
                "emissiveColor".to_owned(),
                emissive_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        state.retain_palette(palette).unwrap();
        state
    }

    #[test]
    fn a_three_component_color_renders_hex_without_alpha() {
        let state = three_component_state();
        let output = show(
            &state,
            &[("0", "emissiveColor", "value", "auto")],
            PaletteShowLayout::Rows,
        );
        // Six hex digits, no alpha pair.
        assert_eq!(output, "0.\"emissiveColor\" #FF0000\n");
    }

    #[test]
    fn a_three_component_color_has_no_alpha_component() {
        let state = three_component_state();
        // `.a` is out of the three-wide shape, but `.r` reads its hex pair.
        assert!(
            resolve_value_collections(
                &state,
                &selectors(&[("0", "emissiveColor.a", "value", "auto")]),
            )
            .is_err()
        );
        let red = show(
            &state,
            &[("0", "emissiveColor.r", "value", "auto")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(red, "0.\"emissiveColor\".r FF\n");
    }

    #[test]
    fn an_hdr_vocabulary_color_errors_under_auto() {
        let mut state = VoxMain::default();
        // The vocabulary bounds a glTF color at [0, 1], so auto errors on the
        // HDR red and an explicit reading spells the exact stored values.
        let emissive_value_pool_id =
            state.retain_value_pool(VoxValuePool::vec_4_float(vec![[2.0, 1.0, 0.5, 1.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property(
                "emissiveColor".to_owned(),
                emissive_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        state.retain_palette(palette).unwrap();

        assert!(
            resolve_value_collections(
                &state,
                &selectors(&[("0", "emissiveColor", "value", "auto")]),
            )
            .is_err()
        );
        let output = show(
            &state,
            &[("0", "emissiveColor", "value", "linear-float")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(output, "0.\"emissiveColor\" lin_srgba(2, 1, 0.5, 1)\n");
    }

    #[test]
    fn a_vocabulary_color_off_its_shape_errors_under_auto() {
        let mut state = VoxMain::default();
        // `baseColor` names a glTF color, so a binding no color reading
        // spells is an error under auto, whatever the shape holds.
        let base_color_value_pool_id =
            state.retain_value_pool(VoxValuePool::vec_3_int(vec![[1, 0, 0]]).unwrap());
        let strength_value_pool_id =
            state.retain_value_pool(VoxValuePool::float(vec![1.0]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property(
                "baseColor".to_owned(),
                base_color_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette
            .retain_property(
                "emissiveColor".to_owned(),
                strength_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette
            .retain_material(vec![value_id(0), value_id(0)])
            .unwrap();
        state.retain_palette(palette).unwrap();

        for key in ["baseColor", "emissiveColor"] {
            assert!(
                resolve_value_collections(&state, &selectors(&[("0", key, "value", "auto")]))
                    .is_err()
            );
        }
    }

    /// One palette binding the custom `tint`, a key outside the glTF
    /// vocabulary, to a `vec-3-float` value pool holding a red.
    fn custom_vector_state() -> VoxMain {
        let mut state = VoxMain::default();
        let tint_value_pool_id =
            state.retain_value_pool(VoxValuePool::vec_3_float(vec![[1.0, 0.0, 0.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property("tint".to_owned(), tint_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        state.retain_palette(palette).unwrap();
        state
    }

    #[test]
    fn a_custom_float_vector_defaults_to_plain_numbers() {
        let state = custom_vector_state();
        // A custom vec-3-float could hold a color or a normal, so it renders
        // plain numbers. A component reads its stored float, and an index
        // past the width errors.
        let output = show(
            &state,
            &[("0", "tint", "value", "auto")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(output, "0.\"tint\" [1,0,0]\n");
        let component = show(
            &state,
            &[("0", "tint.r", "value", "auto")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(component, "0.\"tint\".r 1\n");
        assert!(
            resolve_value_collections(&state, &selectors(&[("0", "tint.w", "value", "auto")]))
                .is_err()
        );
    }

    #[test]
    fn an_srgb_hex_reading_asserts_color_for_a_custom_key() {
        let state = custom_vector_state();
        let output = show(
            &state,
            &[
                ("0", "tint", "value", "srgb-hex"),
                ("0", "tint.r", "value", "srgb-hex"),
            ],
            PaletteShowLayout::Rows,
        );
        assert_eq!(
            output,
            "0.\"tint\"   #FF0000\n\
             \n\
             0.\"tint\".r FF\n"
        );
    }

    #[test]
    fn a_color_reading_on_a_non_color_shape_is_an_error() {
        let state = sample_state();
        // `metallic` binds a float value pool, which no color reading spells.
        for reading in ["linear-float", "srgb-float", "srgb-hex"] {
            assert!(
                resolve_value_collections(
                    &state,
                    &selectors(&[("0", "metallic", "value", reading)])
                )
                .is_err()
            );
        }
    }

    /// One palette binding the custom `tint` to a `vec-4-float` value pool
    /// holding `[1, 0, 0.25, 0.5]`, the design page's worked example.
    fn custom_tint_vec_4_state() -> VoxMain {
        let mut state = VoxMain::default();
        let tint_value_pool_id = state
            .retain_value_pool(VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.25, 0.5]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property("tint".to_owned(), tint_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        state.retain_palette(palette).unwrap();
        state
    }

    #[test]
    fn the_readings_spell_one_color_per_the_design_examples() {
        let state = custom_tint_vec_4_state();
        let row = |fields| show(&state, &[fields], PaletteShowLayout::Rows);
        // The sRGB readings encode the stored 0.25 to 0.537099, byte 0x89;
        // the linear reading keeps the stored numbers; alpha never
        // transfer-encodes. A component respells under the same reading, and
        // either alias set addresses it.
        assert_eq!(
            row(("0", "tint", "value", "auto")),
            "0.\"tint\" [1,0,0.25,0.5]\n"
        );
        assert_eq!(
            row(("0", "tint", "value", "srgb-hex")),
            "0.\"tint\" #FF008980\n"
        );
        assert_eq!(
            row(("0", "tint", "value", "srgb-float")),
            "0.\"tint\" srgba(1, 0, 0.537099, 0.5)\n"
        );
        assert_eq!(
            row(("0", "tint", "value", "linear-float")),
            "0.\"tint\" lin_srgba(1, 0, 0.25, 0.5)\n"
        );
        assert_eq!(
            row(("0", "tint", "swatch-value", "srgb-hex")),
            "0.\"tint\" \x1b[48;2;255;0;137m  \x1b[0m #FF008980\n"
        );
        assert_eq!(
            row(("0", "tint.b", "value", "srgb-hex")),
            "0.\"tint\".b 89\n"
        );
        assert_eq!(row(("0", "tint.z", "value", "auto")), "0.\"tint\".z 0.25\n");
        assert_eq!(
            row(("0", "tint.a", "value", "srgb-float")),
            "0.\"tint\".a 0.5\n"
        );
    }

    #[test]
    fn an_srgb_reading_errors_outside_the_unit_range() {
        let mut state = VoxMain::default();
        // The sRGB readings never clamp: the HDR red errors under both, and
        // the auto fallback stays available through linear-float.
        let tint_value_pool_id =
            state.retain_value_pool(VoxValuePool::vec_4_float(vec![[2.0, 1.0, 0.5, 1.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property("tint".to_owned(), tint_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        state.retain_palette(palette).unwrap();

        for reading in ["srgb-hex", "srgb-float"] {
            assert!(
                resolve_value_collections(&state, &selectors(&[("0", "tint", "value", reading)]))
                    .is_err()
            );
        }
        let output = show(
            &state,
            &[("0", "tint", "value", "linear-float")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(output, "0.\"tint\" lin_srgba(2, 1, 0.5, 1)\n");
    }

    #[test]
    fn a_component_reads_any_vector_shape() {
        let mut state = VoxMain::default();
        let position_value_pool_id =
            state.retain_value_pool(VoxValuePool::vec_3_int(vec![[3, 7, 2]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property(
                "position".to_owned(),
                position_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        state.retain_palette(palette).unwrap();

        // An int vector's component reads its stored int; the width still
        // bounds the index, and no color reading spells an int vector.
        let output = show(
            &state,
            &[("0", "position.x", "value", "auto")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(output, "0.\"position\".x 3\n");
        assert!(
            resolve_value_collections(&state, &selectors(&[("0", "position.w", "value", "auto")]))
                .is_err()
        );
        assert!(
            resolve_value_collections(
                &state,
                &selectors(&[("0", "position.x", "value", "srgb-hex")]),
            )
            .is_err()
        );
    }

    #[test]
    fn an_int_value_pool_renders_integers() {
        let mut state = VoxMain::default();
        let count_value_pool_id = state.retain_value_pool(VoxValuePool::int(vec![3, 7]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property("count".to_owned(), count_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        palette.retain_material(vec![value_id(1)]).unwrap();
        state.retain_palette(palette).unwrap();

        let output = show(
            &state,
            &[("0", "count", "value", "auto")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(output, "0.\"count\" 3 7\n");
    }

    #[test]
    fn a_json_value_pool_renders_arrays_rather_than_null() {
        let mut state = VoxMain::default();
        let extra_value_pool_id = state.retain_value_pool(VoxValuePool::json(vec![
            VoxValue::Array(vec![VoxValue::Number(1.0), VoxValue::Number(2.0)]),
        ]));
        let mut palette = VoxPalette::default();
        palette
            .retain_property("extra".to_owned(), extra_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        state.retain_palette(palette).unwrap();

        // The array survives into both the text and JSON layouts.
        let text = show(
            &state,
            &[("0", "extra", "value", "auto")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(text, "0.\"extra\" [1,2]\n");
        let json = show(
            &state,
            &[("0", "extra", "value", "auto")],
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
        let value_pool_id = state.retain_value_pool(VoxValuePool::boolean(vec![true]));
        let mut palette = VoxPalette::default();
        // A binding with no property name, reached through the `*` property.
        palette
            .retain_property(String::new(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        state.retain_palette(palette).unwrap();

        // An empty name prints quoted as `""` rather than vanishing after the
        // `0.` prefix.
        let row = show(
            &state,
            &[("0", "*", "value", "auto")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(row, "0.\"\" true\n");
        // JSON keeps the raw name; its own string quoting is enough there.
        let json = show(
            &state,
            &[("0", "*", "value", "auto")],
            PaletteShowLayout::JsonCompact,
        );
        assert_eq!(
            json,
            "[{\"label\":\"0\",\"children\":[{\"label\":\"\",\"values\":[true]}]}]\n"
        );
    }

    #[test]
    fn a_shared_value_pool_cell_repeats_per_material() {
        // Both material rows draw the strength value pool's one value, so the
        // column shows it twice.
        let mut state = VoxMain::default();
        let strengths_value_pool_id =
            state.retain_value_pool(VoxValuePool::float(vec![2.0]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property(
                "emissiveStrength".to_owned(),
                strengths_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        state.retain_palette(palette).unwrap();

        let output = show(
            &state,
            &[("0", "emissiveStrength", "value", "auto")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(output, "0.\"emissiveStrength\" 2 2\n");
    }
}
