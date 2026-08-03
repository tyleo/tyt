use crate::{
    ColorComponent, Format, Result, Width,
    commands::{
        PaletteRef, PaletteShowFormat, PaletteShowLabel, PaletteShowLayout, PaletteShowTableShape,
        PaletteShowType, PropertyRef, PropertySelector,
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
    TreeGridRenderTables, TreeGridTableShapeKind,
};
use ty_math::{TyLinSrgbF64, TyLinSrgbaF64, TySrgbaF64};
use voxcore::{
    BVoxValuePoolValue, VoxMain, VoxPalette, VoxValue, VoxValuePool, VoxValuePoolKind,
    VoxValuePoolValueRef,
};
use voxsmith::GltfAttributeKind;

/// Loads the voxel file at `input` and prints the value collections named by
/// `selectors`, each a property's values down a palette, populated into a
/// tree grid of palette, property, and component nodes and rendered under
/// `layout`.
#[allow(clippy::too_many_arguments)]
pub fn palette_show(
    input: &Path,
    from: Option<Format>,
    selectors: &[PropertySelector],
    r#type: Option<PaletteShowType>,
    layout: PaletteShowLayout,
    label: Option<PaletteShowLabel>,
    header_level: Option<NonZeroU8>,
    table_shape: Option<PaletteShowTableShape>,
    width: Width,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;
    let collections = resolve_collections(&state, selectors, r#type)?;
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
    palette_index: usize,
    /// The property key, without any color component.
    key: String,
    /// The color component read from the property, when one was given.
    component: Option<ColorComponent>,
    /// How each value renders.
    format: PaletteShowFormat,
    /// One sample per palette material in material order.
    samples: Vec<TreeGridJsonValue>,
}

/// How a property's values render in `palette show`: a color with three or
/// four components; a plain number; or any other value shown as text with no
/// swatch.
#[derive(Clone, Copy)]
enum Kind {
    /// A float-vector value pool read as a color: `components` is 3 or 4, and
    /// `fits_srgb8` says every component lies in `[0, 1]`, so the collection
    /// displays as 8-bit sRGB; an HDR value pool displays exact linear
    /// functional notation instead.
    Color { components: usize, fits_srgb8: bool },
    /// A `float` or `int` value pool.
    Number,
    /// Any other value pool.
    Other,
}

/// Resolves the selectors against the document's palettes into collections in
/// render order: selector order, then palette order, then property order. A
/// `*` palette or `*` property expands to one collection per match; a named
/// palette or property that is absent is an error, while a `*` palette quietly
/// skips a palette that lacks a named property.
fn resolve_collections(
    state: &VoxMain,
    selectors: &[PropertySelector],
    r#type: Option<PaletteShowType>,
) -> Result<Vec<Collection>> {
    let palettes: Vec<&VoxPalette> = state.iter_palettes().map(|(_, palette)| palette).collect();
    let mut collections = Vec::new();
    for selector in selectors {
        match selector.palette {
            PaletteRef::All => {
                for (palette_index, palette) in palettes.iter().enumerate() {
                    expand_property(
                        state,
                        palette_index,
                        palette,
                        &selector.property,
                        r#type,
                        selector.format,
                        true,
                        &mut collections,
                    )?;
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
                expand_property(
                    state,
                    palette_index,
                    palette,
                    &selector.property,
                    r#type,
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
#[allow(clippy::too_many_arguments)]
fn expand_property(
    state: &VoxMain,
    palette_index: usize,
    palette: &VoxPalette,
    property: &PropertyRef,
    r#type: Option<PaletteShowType>,
    format: PaletteShowFormat,
    palette_is_wild: bool,
    collections: &mut Vec<Collection>,
) -> Result<()> {
    match property {
        PropertyRef::All => {
            for name in implementation::property_names(palette) {
                collections.push(build_collection(
                    state,
                    palette_index,
                    palette,
                    name,
                    None,
                    r#type,
                    format,
                )?);
            }
        }
        PropertyRef::Key { key, component } => {
            if palette.property_id_by_name(key).is_none() {
                if palette_is_wild {
                    return Ok(());
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
            collections.push(build_collection(
                state,
                palette_index,
                palette,
                key,
                *component,
                r#type,
                format,
            )?);
        }
    }
    Ok(())
}

/// Builds one collection from a present property: classifies it by name and
/// bound shape, rejects a color component on a non-color and `.a` on a
/// three-component color, then samples the property's values.
fn build_collection(
    state: &VoxMain,
    palette_index: usize,
    palette: &VoxPalette,
    key: &str,
    component: Option<ColorComponent>,
    r#type: Option<PaletteShowType>,
    format: PaletteShowFormat,
) -> Result<Collection> {
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
    let kind = classify(key, value_pool, r#type);

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

    // A material holds one value id per property, so the lookup with
    // this palette's own property id always resolves.
    let samples = palette
        .iter_materials()
        .map(|material_id| {
            let value_id = palette
                .value_id(material_id, property_id)
                .expect("a material holds a value for every property");
            sample(value_pool, value_id, kind, component)
        })
        .collect();

    Ok(Collection {
        palette_index,
        key: key.to_string(),
        component,
        format,
        samples,
    })
}

/// How the property `key`'s values render. No value pool kind says color: the
/// glTF vocabulary says it for a name it holds, and `--type color` says it for
/// a custom key, whose float vector is otherwise a color or a normal with
/// nothing to tell them apart. The bound shape supplies the component count.
fn classify(key: &str, value_pool: &VoxValuePool, r#type: Option<PaletteShowType>) -> Kind {
    let color = match GltfAttributeKind::of(key) {
        Some(GltfAttributeKind::ColorRgb | GltfAttributeKind::ColorRgba) => true,
        Some(GltfAttributeKind::Scalar) => false,
        None => matches!(r#type, Some(PaletteShowType::Color)),
    };
    match value_pool.kind() {
        VoxValuePoolKind::Vec3Float(_) if color => Kind::Color {
            components: 3,
            fits_srgb8: fits_srgb8(value_pool),
        },
        VoxValuePoolKind::Vec4Float(_) if color => Kind::Color {
            components: 4,
            fits_srgb8: fits_srgb8(value_pool),
        },
        VoxValuePoolKind::Float(_) | VoxValuePoolKind::Int(_) => Kind::Number,
        VoxValuePoolKind::Bool(_)
        | VoxValuePoolKind::Json(_)
        | VoxValuePoolKind::String(_)
        | VoxValuePoolKind::Vec2Float(_)
        | VoxValuePoolKind::Vec2Int(_)
        | VoxValuePoolKind::Vec3Float(_)
        | VoxValuePoolKind::Vec3Int(_)
        | VoxValuePoolKind::Vec4Float(_)
        | VoxValuePoolKind::Vec4Int(_) => Kind::Other,
    }
}

/// Whether every component of every color in the value pool lies in `[0, 1]`,
/// so the collection displays as 8-bit sRGB; an HDR value pool displays exact
/// linear functional notation instead, which no hex can hold.
fn fits_srgb8(value_pool: &VoxValuePool) -> bool {
    value_pool.iter_values().all(|(_, value)| match value {
        VoxValuePoolValueRef::Vec3Float(color) => color
            .iter()
            .all(|component| (0.0..=1.0).contains(component)),
        VoxValuePoolValueRef::Vec4Float(color) => color
            .iter()
            .all(|component| (0.0..=1.0).contains(component)),
        // classify() reads only float vectors as colors.
        _ => false,
    })
}

/// The sample for the value at `value_id` under its property's `kind` and an
/// optional color `component`.
fn sample(
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
    kind: Kind,
    component: Option<ColorComponent>,
) -> TreeGridJsonValue {
    match kind {
        Kind::Color {
            components,
            fits_srgb8,
        } => sample_color(value_pool, value_id, components, fits_srgb8, component),
        Kind::Number => sample_number(value_pool, value_id),
        Kind::Other => sample_other(value_pool, value_id),
    }
}

/// The sample for a stored linear color:
///
/// 1. A whole color in `[0, 1]`: sRGB hex with a color swatch.
/// 2. A whole HDR color: exact `lrgb(...)` / `lrgba(...)` functional notation
///    with a color swatch.
/// 3. One `component` channel: an sRGB byte (in `[0, 1]`) or a linear float
///    (HDR) with a grayscale swatch.
fn sample_color(
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
    components: usize,
    fits_srgb8: bool,
    component: Option<ColorComponent>,
) -> TreeGridJsonValue {
    let bytes = color_bytes(value_pool, value_id);
    match component {
        Some(component) => {
            let channel_index = component_index(component);
            if fits_srgb8 {
                TreeGridJsonValue::unorm8(bytes[channel_index])
            } else {
                TreeGridJsonValue::unorm(color_floats(value_pool, value_id)[channel_index])
            }
        }
        None if fits_srgb8 => {
            if components == 4 {
                TreeGridJsonValue::srgba8(bytes)
            } else {
                TreeGridJsonValue::srgb8([bytes[0], bytes[1], bytes[2]])
            }
        }
        None => {
            let floats = color_floats(value_pool, value_id);
            if components == 4 {
                TreeGridJsonValue::lin_rgba(TyLinSrgbaF64::new(
                    floats[0], floats[1], floats[2], floats[3],
                ))
            } else {
                TreeGridJsonValue::lin_rgb(TyLinSrgbF64::new(floats[0], floats[1], floats[2]))
            }
        }
    }
}

/// The sample for a `float` or `int` value: its number, with a grayscale swatch
/// mapping its `0..1` range onto `0..255`.
fn sample_number(
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
) -> TreeGridJsonValue {
    let value = match value_pool.value(value_id) {
        Some(VoxValuePoolValueRef::Float(number)) => number,
        Some(VoxValuePoolValueRef::Int(number)) => number as f64,
        // classify() routes only Float and Int here, and a validated document
        // draws only retained values.
        _ => 0.0,
    };
    TreeGridJsonValue::unorm(value)
}

/// The sample for any other value: its text and native JSON with no swatch. A
/// float vector no name or `--type` reads as a color and an int vector render
/// as their number arrays.
fn sample_other(
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
) -> TreeGridJsonValue {
    match value_pool.value(value_id) {
        Some(VoxValuePoolValueRef::Bool(flag)) => TreeGridJsonValue::bool(flag),
        Some(VoxValuePoolValueRef::String(text)) => TreeGridJsonValue::new(text.to_owned()),
        Some(VoxValuePoolValueRef::Json(value)) => {
            TreeGridJsonValue::json(vox_value_to_json(value))
        }
        Some(VoxValuePoolValueRef::Vec2Float(vector)) => float_array_json(vector),
        Some(VoxValuePoolValueRef::Vec3Float(vector)) => float_array_json(vector),
        Some(VoxValuePoolValueRef::Vec4Float(vector)) => float_array_json(vector),
        Some(VoxValuePoolValueRef::Vec2Int(vector)) => int_array_json(vector),
        Some(VoxValuePoolValueRef::Vec3Int(vector)) => int_array_json(vector),
        Some(VoxValuePoolValueRef::Vec4Int(vector)) => int_array_json(vector),
        // classify() routes float and int to Number, and a validated document
        // draws only retained values.
        _ => TreeGridJsonValue::json(Value::Null),
    }
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

/// The `[r, g, b, a]` bytes for the stored linear color at `value_id`, encoded
/// to sRGB at display; the alpha only quantizes, and a three-component color
/// takes opaque alpha.
fn color_bytes(value_pool: &VoxValuePool, value_id: U32Id<BVoxValuePoolValue>) -> [u8; 4] {
    match value_pool.value(value_id) {
        Some(VoxValuePoolValueRef::Vec3Float(&[r, g, b])) => <[u8; 4]>::from(
            TySrgbaF64::from_linear(TyLinSrgbaF64::new(r, g, b, 1.0)).into_format::<u8, u8>(),
        ),
        Some(VoxValuePoolValueRef::Vec4Float(&[r, g, b, a])) => <[u8; 4]>::from(
            TySrgbaF64::from_linear(TyLinSrgbaF64::new(r, g, b, a)).into_format::<u8, u8>(),
        ),
        // classify() reads only float vectors as colors, and a validated
        // document draws only retained values.
        _ => [0, 0, 0, 0],
    }
}

/// The stored linear float components of the color at `value_id`, three or
/// four long by shape.
fn color_floats(value_pool: &VoxValuePool, value_id: U32Id<BVoxValuePoolValue>) -> Vec<f64> {
    match value_pool.value(value_id) {
        Some(VoxValuePoolValueRef::Vec3Float(color)) => color.to_vec(),
        Some(VoxValuePoolValueRef::Vec4Float(color)) => color.to_vec(),
        // classify() reads only float vectors as colors, and a validated
        // document draws only retained values.
        _ => Vec::new(),
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

/// A number as JSON: an integer when it is integral and fits `i64`, else a
/// float, so it reads as it does in the text layouts.
fn number_json(value: f64) -> Value {
    if value.fract() == 0.0 && value.abs() < i64::MAX as f64 {
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
        let palette_node_id = match palette_node {
            Some((palette_index, node_id)) if palette_index == collection.palette_index => node_id,
            _ => {
                let node_id =
                    grid.add_root(TreeGridLabel::bare(collection.palette_index.to_string()));
                palette_node = Some((collection.palette_index, node_id));
                property_node = None;
                node_id
            }
        };
        let data_node_id = match collection.component {
            Some(component) => {
                let property_node_id = match &property_node {
                    Some((key, node_id)) if *key == collection.key => *node_id,
                    _ => grid.add_child(
                        palette_node_id,
                        TreeGridLabel::quoted(collection.key.as_str()),
                    ),
                };
                property_node = Some((collection.key, property_node_id));
                let letter = component_letter(component).to_string();
                grid.add_child(property_node_id, TreeGridLabel::bare(letter))
            }
            None => {
                // A data node is always fresh, so a property selected twice
                // keeps one collection per selector.
                let node_id = grid.add_child(
                    palette_node_id,
                    TreeGridLabel::quoted(collection.key.as_str()),
                );
                property_node = Some((collection.key, node_id));
                node_id
            }
        };
        let node = grid.node_mut(data_node_id);
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
        commands::{
            PaletteShowLabel, PaletteShowLayout, PaletteShowTableShape, PaletteShowType,
            PropertySelector,
        },
        implementation::palette_show::{build_grid, render, resolve_collections},
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
        state.add_value_pool(VoxValuePool::vec_4_float(values).unwrap())
    }

    /// A document with two palettes: palette 0 has `baseColor` and
    /// `metallic` with two materials, palette 1 has `baseColor` with
    /// one material.
    fn sample_state() -> VoxMain {
        let mut state = VoxMain::default();

        let colors_zero_value_pool_id =
            lin_srgba_f64_value_pool(&mut state, &[[255, 0, 0, 255], [0, 255, 0, 128]]);
        let metallic_value_pool_id =
            state.add_value_pool(VoxValuePool::float(vec![1.0, 0.2]).unwrap());
        let colors_one_value_pool_id = lin_srgba_f64_value_pool(&mut state, &[[0, 0, 255, 255]]);

        let mut first = VoxPalette::default();
        first
            .add_property(
                "baseColor".to_owned(),
                colors_zero_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        first
            .add_property(
                "metallic".to_owned(),
                metallic_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        first.add_material(vec![value_id(0), value_id(0)]).unwrap();
        first.add_material(vec![value_id(1), value_id(1)]).unwrap();
        state.add_palette(first).unwrap();

        let mut second = VoxPalette::default();
        second
            .add_property(
                "baseColor".to_owned(),
                colors_one_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        second.add_material(vec![value_id(0)]).unwrap();
        state.add_palette(second).unwrap();

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

    /// The populated grid for the selectors, resolved against `state` with no
    /// asserted type.
    fn grid_for(
        state: &VoxMain,
        fields: &[(&str, &str, &str)],
    ) -> TreeGrid<TreeGridJsonValueCells> {
        build_grid(resolve_collections(state, &selectors(fields), None).unwrap())
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
            &[("0", "baseColor", "value")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(output, "0.\"baseColor\" #FF0000FF #00FF0080\n");
    }

    #[test]
    fn extracts_a_color_component_as_a_byte() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "baseColor.a", "value")],
            PaletteShowLayout::Rows,
        );
        // Alpha bytes FF and 80 as 0..255 integers.
        assert_eq!(output, "0.\"baseColor\".a 255 128\n");
    }

    #[test]
    fn swatch_format_abuts_swatches_into_a_strip() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "baseColor", "swatch")],
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
            state.add_value_pool(VoxValuePool::boolean(vec![true, false]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .add_property(
                "shadows".to_owned(),
                shadows_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette.add_material(vec![value_id(0)]).unwrap();
        palette.add_material(vec![value_id(1)]).unwrap();
        state.add_palette(palette).unwrap();

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
        // the values are not column-aligned: `metallic` stays compact
        // rather than padding out to the wider `baseColor` columns.
        let output = show(
            &state,
            &[("0", "baseColor", "value"), ("0", "metallic", "value")],
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
        let grid = grid_for(&state, &[("0", "baseColor", "value")]);
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
        let grid = grid_for(&state, &[("0", "baseColor", "value")]);
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
            resolve_collections(&state, &[PropertySelector::default_all_auto()], None).unwrap();
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
            "0.\"baseColor\" \x1b[48;2;255;0;0m  \x1b[0m #FF0000FF \x1b[48;2;0;255;0m  \x1b[0m #00FF0080\n\
             \n\
             0.\"metallic\"  1 0.2\n\
             \n\
             1.\"baseColor\" \x1b[48;2;0;0;255m  \x1b[0m #0000FFFF\n"
        );
    }

    #[test]
    fn collections_render_in_selector_order() {
        let state = sample_state();
        // A palette revisited later starts a fresh root rather than merging
        // backward, so pre-order keeps the selector order.
        let output = show(
            &state,
            &[("1", "baseColor", "value"), ("0", "baseColor", "value")],
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
            &[("0", "baseColor", "value"), ("0", "baseColor", "swatch")],
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
    fn columns_stack_collections_under_labels() {
        let state = sample_state();
        let output = show(
            &state,
            &[("0", "baseColor.a", "value"), ("1", "baseColor.a", "value")],
            PaletteShowLayout::Columns,
        );
        assert_eq!(
            output,
            "0.\"baseColor\".a 1.\"baseColor\".a\n255             255\n128\n"
        );
    }

    #[test]
    fn columns_with_label_none_drop_the_label_row() {
        let state = sample_state();
        let grid = grid_for(
            &state,
            &[("0", "baseColor.a", "value"), ("1", "baseColor.a", "value")],
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
            "└ 0\n  ├ \"baseColor\": #FF0000FF #00FF0080\n  └ \"metallic\": 1 0.2\n"
        );
    }

    #[test]
    fn header_labels_group_rows_under_palette_headings() {
        let state = sample_state();
        let grid = grid_for(&state, &[("*", "baseColor", "value")]);
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
        let grid = grid_for(&state, &[("*", "baseColor", "value")]);
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
            &[("*", "baseColor", "value")],
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
        let grid = grid_for(&state, &[("*", "baseColor", "value")]);
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
        let grid = grid_for(&state, &[("*", "baseColor", "value")]);
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
            &[("0", "baseColor", "value"), ("0", "baseColor.a", "value")],
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
             | label       | value               | a       |\n\
             | ----------- | ------------------- | ------- |\n\
             | \"baseColor\" | #FF0000FF #00FF0080 | 255 128 |\n"
        );
    }

    #[test]
    fn a_label_mode_on_the_hierarchy_layout_is_invalid_input() {
        let state = sample_state();
        let grid = grid_for(&state, &[("0", "baseColor", "value")]);
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
            &[("0", "baseColor", "value"), ("0", "baseColor.a", "value")],
            PaletteShowLayout::JsonCompact,
        );
        assert_eq!(
            output,
            "[{\"label\":\"0\",\"children\":[{\"label\":\"baseColor\",\
             \"values\":[\"#FF0000FF\",\"#00FF0080\"],\"children\":[\
             {\"label\":\"a\",\"values\":[255,128]}]}]}]\n"
        );
    }

    #[test]
    fn pretty_json_is_indented_and_matches_compact() {
        let state = sample_state();
        let fields: &[(&str, &str, &str)] =
            &[("0", "baseColor", "value"), ("0", "baseColor.a", "value")];
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
        let collections =
            resolve_collections(&state, &selectors(&[("0", "*", "value")]), None).unwrap();
        let keys: Vec<&str> = collections.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(keys, ["baseColor", "metallic"]);
    }

    #[test]
    fn star_palette_skips_a_palette_lacking_a_named_property() {
        let state = sample_state();
        // Only palette 0 has `metallic`; palette 1 is skipped, not an error.
        let collections =
            resolve_collections(&state, &selectors(&[("*", "metallic", "value")]), None).unwrap();
        let labels: Vec<(usize, &str)> = collections
            .iter()
            .map(|c| (c.palette_index, c.key.as_str()))
            .collect();
        assert_eq!(labels, [(0, "metallic")]);
    }

    #[test]
    fn named_palette_out_of_range_is_an_error() {
        let state = sample_state();
        assert!(
            resolve_collections(&state, &selectors(&[("5", "baseColor", "value")]), None).is_err()
        );
    }

    #[test]
    fn named_property_absent_on_a_named_palette_is_an_error() {
        let state = sample_state();
        assert!(
            resolve_collections(&state, &selectors(&[("0", "missing", "value")]), None).is_err()
        );
    }

    #[test]
    fn a_component_on_a_scalar_is_an_error() {
        let state = sample_state();
        assert!(
            resolve_collections(&state, &selectors(&[("0", "metallic.r", "value")]), None).is_err()
        );
    }

    #[test]
    fn auto_swatches_colors_but_prints_scalars_and_components_as_numbers() {
        let state = sample_state();
        let scalar = show(
            &state,
            &[("0", "metallic", "auto")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(scalar, "0.\"metallic\" 1 0.2\n");
        let component = show(
            &state,
            &[("0", "baseColor.r", "auto")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(component, "0.\"baseColor\".r 255 0\n");
    }

    /// One palette binding `emissiveColor`, three components per the glTF
    /// vocabulary, to a `vec-3-float` value pool holding a red.
    fn three_component_state() -> VoxMain {
        let mut state = VoxMain::default();
        let emissive_value_pool_id =
            state.add_value_pool(VoxValuePool::vec_3_float(vec![[1.0, 0.0, 0.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .add_property(
                "emissiveColor".to_owned(),
                emissive_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette.add_material(vec![value_id(0)]).unwrap();
        state.add_palette(palette).unwrap();
        state
    }

    #[test]
    fn a_three_component_color_renders_hex_without_alpha() {
        let state = three_component_state();
        let output = show(
            &state,
            &[("0", "emissiveColor", "value")],
            PaletteShowLayout::Rows,
        );
        // Six hex digits, no alpha pair.
        assert_eq!(output, "0.\"emissiveColor\" #FF0000\n");
    }

    #[test]
    fn a_three_component_color_has_no_alpha_component() {
        let state = three_component_state();
        // `.a` is out of range on a three-component color, but `.r` reads.
        assert!(
            resolve_collections(
                &state,
                &selectors(&[("0", "emissiveColor.a", "value")]),
                None
            )
            .is_err()
        );
        let red = show(
            &state,
            &[("0", "emissiveColor.r", "value")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(red, "0.\"emissiveColor\".r 255\n");
    }

    #[test]
    fn an_hdr_color_renders_functional_notation() {
        let mut state = VoxMain::default();
        // A component above 1 has no hex spelling, so the exact stored linear
        // values render functional.
        let emissive_value_pool_id =
            state.add_value_pool(VoxValuePool::vec_4_float(vec![[2.0, 1.0, 0.5, 1.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .add_property(
                "emissiveColor".to_owned(),
                emissive_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette.add_material(vec![value_id(0)]).unwrap();
        state.add_palette(palette).unwrap();

        let output = show(
            &state,
            &[("0", "emissiveColor", "value")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(output, "0.\"emissiveColor\" lrgba(2, 1, 0.5, 1)\n");
    }

    /// One palette binding the custom `tint` to a `vec-3-float` value pool
    /// holding a red, which no name classifies.
    fn custom_vector_state() -> VoxMain {
        let mut state = VoxMain::default();
        let tint_value_pool_id =
            state.add_value_pool(VoxValuePool::vec_3_float(vec![[1.0, 0.0, 0.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .add_property("tint".to_owned(), tint_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.add_material(vec![value_id(0)]).unwrap();
        state.add_palette(palette).unwrap();
        state
    }

    #[test]
    fn a_custom_float_vector_defaults_to_plain_numbers() {
        let state = custom_vector_state();
        // A custom vec-3-float is a color or a normal, and nothing says which,
        // so it renders its numbers and carries no color component.
        let output = show(&state, &[("0", "tint", "value")], PaletteShowLayout::Rows);
        assert_eq!(output, "0.\"tint\" [1,0,0]\n");
        assert!(
            resolve_collections(&state, &selectors(&[("0", "tint.r", "value")]), None).is_err()
        );
    }

    #[test]
    fn type_color_asserts_color_for_a_custom_key() {
        let state = custom_vector_state();
        let collections = resolve_collections(
            &state,
            &selectors(&[("0", "tint", "value"), ("0", "tint.r", "value")]),
            Some(PaletteShowType::Color),
        )
        .unwrap();
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
            "0.\"tint\"   #FF0000\n\
             \n\
             0.\"tint\".r 255\n"
        );
    }

    #[test]
    fn type_color_never_reclassifies_a_vocabulary_name() {
        let state = sample_state();
        // `metallic` is a scalar by name, so the assertion does not touch it.
        let collections = resolve_collections(
            &state,
            &selectors(&[("0", "metallic", "value")]),
            Some(PaletteShowType::Color),
        )
        .unwrap();
        let output = render(
            &build_grid(collections),
            PaletteShowLayout::Rows,
            None,
            None,
            None,
            Width::Unlimited,
        )
        .unwrap();
        assert_eq!(output, "0.\"metallic\" 1 0.2\n");
    }

    #[test]
    fn an_int_value_pool_renders_integers() {
        let mut state = VoxMain::default();
        let count_value_pool_id = state.add_value_pool(VoxValuePool::int(vec![3, 7]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .add_property("count".to_owned(), count_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.add_material(vec![value_id(0)]).unwrap();
        palette.add_material(vec![value_id(1)]).unwrap();
        state.add_palette(palette).unwrap();

        let output = show(&state, &[("0", "count", "value")], PaletteShowLayout::Rows);
        assert_eq!(output, "0.\"count\" 3 7\n");
    }

    #[test]
    fn a_json_value_pool_renders_arrays_rather_than_null() {
        let mut state = VoxMain::default();
        let extra_value_pool_id = state.add_value_pool(
            VoxValuePool::json(vec![VoxValue::Array(vec![
                VoxValue::Number(1.0),
                VoxValue::Number(2.0),
            ])])
            .unwrap(),
        );
        let mut palette = VoxPalette::default();
        palette
            .add_property("extra".to_owned(), extra_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.add_material(vec![value_id(0)]).unwrap();
        state.add_palette(palette).unwrap();

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
        let value_pool_id = state.add_value_pool(VoxValuePool::boolean(vec![true]).unwrap());
        let mut palette = VoxPalette::default();
        // A binding with no property name, reached through the `*` property.
        palette
            .add_property(String::new(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.add_material(vec![value_id(0)]).unwrap();
        state.add_palette(palette).unwrap();

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

    #[test]
    fn a_shared_value_pool_cell_repeats_per_material() {
        // Both material rows draw the strength value pool's one value, so the
        // column shows it twice.
        let mut state = VoxMain::default();
        let strengths_value_pool_id = state.add_value_pool(VoxValuePool::float(vec![2.0]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .add_property(
                "emissiveStrength".to_owned(),
                strengths_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette.add_material(vec![value_id(0)]).unwrap();
        palette.add_material(vec![value_id(0)]).unwrap();
        state.add_palette(palette).unwrap();

        let output = show(
            &state,
            &[("0", "emissiveStrength", "value")],
            PaletteShowLayout::Rows,
        );
        assert_eq!(output, "0.\"emissiveStrength\" 2 2\n");
    }
}
