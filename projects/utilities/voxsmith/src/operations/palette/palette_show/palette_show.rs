use crate::{
    Error, Result,
    operations::palette::{
        PaletteRef, PaletteShowLabel, PaletteShowLayout, PaletteShowOptions,
        PaletteShowPresentation, PaletteShowReading, PaletteShowTableShape, PropertyRef,
        PropertySelector, palette_show::StoredColor,
    },
    utilities::{VectorComponent, property_names},
};
use branded_id::U32Id;
use serde_json::{Value, json};
use std::result::Result as StdResult;
use treegrid::{
    BTreeGridNode, TreeGrid, TreeGridCellFormat, TreeGridError, TreeGridJsonValue,
    TreeGridJsonValueCells, TreeGridLabel, TreeGridLabelKind, TreeGridOptions,
    TreeGridRenderBoxHierarchy, TreeGridRenderBoxTables, TreeGridRenderJson,
    TreeGridRenderMdTables, TreeGridRenderTextColumns, TreeGridRenderTextRows, TreeGridSwatch,
    TreeGridTableShapeKind,
};
use voxcore::{
    BVoxValuePoolValue, VoxExt, VoxMain, VoxPalette, VoxValue, VoxValueColumn, VoxValuePool,
    VoxValuePoolValues, material::MaterialPropertyKind,
};

/// Renders the value collections `selectors` name in `main`, each a
/// property's values down a palette, populated into a tree grid of palette,
/// property, and component nodes and rendered under `options`. Errors when a
/// selector names an absent palette or property, a component off its shape,
/// a color reading off a color shape, or an option the layout ignores.
pub fn palette_show<T: VoxExt>(
    main: &VoxMain<T>,
    selectors: &[PropertySelector],
    options: &PaletteShowOptions,
) -> Result<String> {
    let value_collections = resolve_value_collections(main, selectors)?;

    let grid = build_grid(value_collections);

    render(&grid, options)
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
    Plain,

    Color(ColorReading),
}

/// A reading that spells a color-shaped value as a color.
#[derive(Clone, Copy)]
enum ColorReading {
    LinearFloat,

    SrgbFloat,

    SrgbHex,
}

/// Resolves the selectors against the document's palettes into value
/// collections in render order: selector order, then palette order, then
/// property order. A `*` palette or `*` property expands to one value
/// collection per match; a named palette or property that is absent is an
/// error, while a `*` palette quietly skips a palette that lacks a named
/// property.
fn resolve_value_collections<T: VoxExt>(
    main: &VoxMain<T>,
    selectors: &[PropertySelector],
) -> Result<Vec<ValueCollection>> {
    let palettes: Vec<&VoxPalette> = main.iter_palettes().map(|(_, palette)| palette).collect();
    let mut value_collections = Vec::new();
    for selector in selectors {
        match selector.palette {
            PaletteRef::All => {
                for (palette_index, palette) in palettes.iter().enumerate() {
                    value_collections.extend(expand_property(
                        main,
                        palette_index,
                        palette,
                        selector,
                        true,
                    )?);
                }
            }

            PaletteRef::Index(palette_index) => {
                let palette = palettes.get(palette_index).ok_or_else(|| {
                    Error::invalid(format!(
                        "palette index {palette_index} is out of range; the document has {} palette(s)",
                        palettes.len()
                    ))
                })?;
                value_collections.extend(expand_property(
                    main,
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
fn expand_property<T: VoxExt>(
    main: &VoxMain<T>,
    palette_index: usize,
    palette: &VoxPalette,
    selector: &PropertySelector,
    palette_is_wild: bool,
) -> Result<Vec<ValueCollection>> {
    match &selector.property {
        PropertyRef::All => property_names(palette)
            .into_iter()
            .map(|name| {
                build_value_collection(
                    main,
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
                return Err(Error::invalid(format!(
                    "palette {palette_index} has no property `{key}`; available properties: {}",
                    available_keys(palette)
                )));
            }
            Ok(vec![build_value_collection(
                main,
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
fn build_value_collection<T: VoxExt>(
    main: &VoxMain<T>,
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
    let value_pool = main
        .value_pool(value_pool_id)
        .expect("a property references a value pool the main holds");

    // A component is shape addressing: legal on any vector value pool whose
    // width exceeds its index, whatever the reading.
    if let Some(component) = component {
        match vector_width(value_pool) {
            None => {
                return Err(Error::invalid(format!(
                    "property `{key}` is not a vector and has no `.{}` component",
                    component.letter()
                )));
            }

            Some(width) if component.index() >= width => {
                return Err(Error::invalid(format!(
                    "property `{key}` is {width} components wide and has no `.{}` component",
                    component.letter()
                )));
            }

            Some(_) => {}
        }
    }

    let reading = resolve_reading(key, value_pool, reading)?;

    // A material holds one value id per property, so the lookup with
    // this palette's own property id always resolves.
    let value_ids: Vec<_> = palette
        .iter_materials()
        .map(|material_id| {
            palette
                .value_id(material_id, property_id)
                .expect("a material holds a value for every property")
        })
        .collect();

    let samples = samples(key, value_pool, &value_ids, reading, component)?;

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
    match value_pool.values() {
        VoxValuePoolValues::Bool(_)
        | VoxValuePoolValues::Float(_)
        | VoxValuePoolValues::Int(_)
        | VoxValuePoolValues::Json(_)
        | VoxValuePoolValues::String(_) => None,

        VoxValuePoolValues::Vec2Float(_) | VoxValuePoolValues::Vec2Int(_) => Some(2),

        VoxValuePoolValues::Vec3Float(_) | VoxValuePoolValues::Vec3Int(_) => Some(3),

        VoxValuePoolValues::Vec4Float(_) | VoxValuePoolValues::Vec4Int(_) => Some(4),
    }
}

/// Whether the value pool holds a color-shaped value, three or four float
/// components.
fn color_shape(value_pool: &VoxValuePool) -> bool {
    matches!(
        value_pool.values(),
        VoxValuePoolValues::Vec3Float(_) | VoxValuePoolValues::Vec4Float(_)
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
        PaletteShowReading::Auto => match MaterialPropertyKind::of(key) {
            Some(MaterialPropertyKind::ColorRgb | MaterialPropertyKind::ColorRgba) => {
                if !color_shape(value_pool) {
                    return Err(Error::invalid(format!(
                        "property `{key}` names a glTF color and must bind a vec-3-float or \
                         vec-4-float value pool"
                    )));
                }
                Ok(Reading::Color(ColorReading::SrgbHex))
            }

            Some(MaterialPropertyKind::Scalar) | None => Ok(Reading::Plain),
        },

        PaletteShowReading::LinearFloat => {
            color_reading(key, value_pool, "linear-float", ColorReading::LinearFloat)
        }

        PaletteShowReading::Plain => Ok(Reading::Plain),

        PaletteShowReading::SrgbFloat => {
            color_reading(key, value_pool, "srgb-float", ColorReading::SrgbFloat)
        }

        PaletteShowReading::SrgbHex => {
            color_reading(key, value_pool, "srgb-hex", ColorReading::SrgbHex)
        }
    }
}

/// Admits a color reading on a color-shaped value pool and errors on every
/// other kind.
fn color_reading(
    key: &str,
    value_pool: &VoxValuePool,
    name: &str,
    reading: ColorReading,
) -> Result<Reading> {
    if color_shape(value_pool) {
        Ok(Reading::Color(reading))
    } else {
        Err(Error::invalid(format!(
            "property `{key}` does not bind a vec-3-float or vec-4-float value pool, which \
             the `{name}` reading requires"
        )))
    }
}

/// The samples for the values `value_ids` draw, under the resolved `reading`
/// and an optional vector `component`. The kind and the reading match once
/// for all values.
fn samples(
    key: &str,
    value_pool: &VoxValuePool,
    value_ids: &[U32Id<BVoxValuePoolValue>],
    reading: Reading,
    component: Option<VectorComponent>,
) -> Result<Vec<TreeGridJsonValue>> {
    let Reading::Color(reading) = reading else {
        return Ok(match component {
            Some(component) => plain_component_samples(value_pool, value_ids, component.index()),
            None => plain_samples(value_pool, value_ids),
        });
    };

    match value_pool.values() {
        VoxValuePoolValues::Vec3Float(colors) => {
            color_samples(key, colors, value_ids, reading, component)
        }

        VoxValuePoolValues::Vec4Float(colors) => {
            color_samples(key, colors, value_ids, reading, component)
        }

        _ => unreachable!("a color reading resolves only on a color shape"),
    }
}

/// The color-reading samples for the colors `value_ids` draw. A component's
/// grayscale swatch shows the sRGB byte the whole-color spelling carries for
/// that channel. An sRGB reading errors on a component outside `[0, 1]`.
fn color_samples<C: StoredColor>(
    key: &str,
    colors: VoxValueColumn<'_, C>,
    value_ids: &[U32Id<BVoxValuePoolValue>],
    reading: ColorReading,
    component: Option<VectorComponent>,
) -> Result<Vec<TreeGridJsonValue>> {
    let colors = drawn(colors, value_ids);

    let Some(component) = component else {
        let spell: fn(&str, &C) -> Result<TreeGridJsonValue> = match reading {
            ColorReading::LinearFloat => linear_float_sample,
            ColorReading::SrgbFloat => srgb_float_sample,
            ColorReading::SrgbHex => srgb_hex_sample,
        };

        return colors.map(|color| spell(key, color)).collect();
    };

    let spell: fn(&str, &C, usize) -> Result<TreeGridJsonValue> = match (reading, component) {
        (ColorReading::LinearFloat, _) => linear_float_component_sample,
        (ColorReading::SrgbFloat, VectorComponent::A) => srgb_float_alpha_component_sample,
        (ColorReading::SrgbFloat, _) => srgb_float_rgb_component_sample,
        (ColorReading::SrgbHex, _) => srgb_hex_component_sample,
    };

    let index = component.index();
    colors.map(|color| spell(key, color, index)).collect()
}

fn linear_float_sample<C: StoredColor>(_key: &str, color: &C) -> Result<TreeGridJsonValue> {
    Ok(color.linear_float())
}

fn srgb_float_sample<C: StoredColor>(key: &str, color: &C) -> Result<TreeGridJsonValue> {
    require_unit(key, color.components())?;
    Ok(color.srgb_float())
}

fn srgb_hex_sample<C: StoredColor>(key: &str, color: &C) -> Result<TreeGridJsonValue> {
    require_unit(key, color.components())?;
    Ok(color.srgb_hex())
}

/// One color component under `linear-float`: the stored number.
fn linear_float_component_sample<C: StoredColor>(
    _key: &str,
    color: &C,
    index: usize,
) -> Result<TreeGridJsonValue> {
    let byte = color.display_bytes()[index];
    Ok(TreeGridJsonValue::float(color.components()[index]).with_swatch(TreeGridSwatch::Gray(byte)))
}

/// One rgb component under `srgb-float`: transfer-encoded.
fn srgb_float_rgb_component_sample<C: StoredColor>(
    key: &str,
    color: &C,
    index: usize,
) -> Result<TreeGridJsonValue> {
    require_unit(key, &[color.components()[index]])?;
    let byte = color.display_bytes()[index];
    Ok(
        TreeGridJsonValue::float(color.srgb_float_rgb_component(index))
            .with_swatch(TreeGridSwatch::Gray(byte)),
    )
}

/// The alpha component under `srgb-float`: passed through.
fn srgb_float_alpha_component_sample<C: StoredColor>(
    key: &str,
    color: &C,
    index: usize,
) -> Result<TreeGridJsonValue> {
    let stored = color.components()[index];
    require_unit(key, &[stored])?;
    let byte = color.display_bytes()[index];
    Ok(TreeGridJsonValue::float(stored).with_swatch(TreeGridSwatch::Gray(byte)))
}

/// One color component under `srgb-hex`: its sRGB byte.
fn srgb_hex_component_sample<C: StoredColor>(
    key: &str,
    color: &C,
    index: usize,
) -> Result<TreeGridJsonValue> {
    require_unit(key, &[color.components()[index]])?;
    let byte = color.display_bytes()[index];
    Ok(TreeGridJsonValue::new(format!("{byte:02X}")).with_swatch(TreeGridSwatch::Gray(byte)))
}

/// The `plain` samples for whole values: each stored value as it is, with a
/// `float` or `int` number on the grayscale ramp.
fn plain_samples(
    value_pool: &VoxValuePool,
    value_ids: &[U32Id<BVoxValuePoolValue>],
) -> Vec<TreeGridJsonValue> {
    match value_pool.values() {
        VoxValuePoolValues::Bool(flags) => {
            spelled(flags, value_ids, |&flag| TreeGridJsonValue::bool(flag))
        }

        VoxValuePoolValues::Float(numbers) => spelled(numbers, value_ids, |&number| {
            TreeGridJsonValue::unorm(number)
        }),

        VoxValuePoolValues::Int(numbers) => spelled(numbers, value_ids, |&number| {
            TreeGridJsonValue::unorm(number as f64)
        }),

        VoxValuePoolValues::Json(values) => spelled(values, value_ids, |value| {
            TreeGridJsonValue::json(vox_value_to_json(value))
        }),

        VoxValuePoolValues::String(texts) => spelled(texts, value_ids, |text| {
            TreeGridJsonValue::new(text.clone())
        }),

        VoxValuePoolValues::Vec2Float(vectors) => {
            spelled(vectors, value_ids, |vector| float_array_json(vector))
        }

        VoxValuePoolValues::Vec2Int(vectors) => {
            spelled(vectors, value_ids, |vector| int_array_json(vector))
        }

        VoxValuePoolValues::Vec3Float(vectors) => {
            spelled(vectors, value_ids, |vector| float_array_json(vector))
        }

        VoxValuePoolValues::Vec3Int(vectors) => {
            spelled(vectors, value_ids, |vector| int_array_json(vector))
        }

        VoxValuePoolValues::Vec4Float(vectors) => {
            spelled(vectors, value_ids, |vector| float_array_json(vector))
        }

        VoxValuePoolValues::Vec4Int(vectors) => {
            spelled(vectors, value_ids, |vector| int_array_json(vector))
        }
    }
}

/// The `plain` samples for vector component `index`: each stored number on
/// the grayscale ramp.
fn plain_component_samples(
    value_pool: &VoxValuePool,
    value_ids: &[U32Id<BVoxValuePoolValue>],
    index: usize,
) -> Vec<TreeGridJsonValue> {
    match value_pool.values() {
        VoxValuePoolValues::Vec2Float(vectors) => float_components(vectors, value_ids, index),

        VoxValuePoolValues::Vec2Int(vectors) => int_components(vectors, value_ids, index),

        VoxValuePoolValues::Vec3Float(vectors) => float_components(vectors, value_ids, index),

        VoxValuePoolValues::Vec3Int(vectors) => int_components(vectors, value_ids, index),

        VoxValuePoolValues::Vec4Float(vectors) => float_components(vectors, value_ids, index),

        VoxValuePoolValues::Vec4Int(vectors) => int_components(vectors, value_ids, index),

        VoxValuePoolValues::Bool(_)
        | VoxValuePoolValues::Float(_)
        | VoxValuePoolValues::Int(_)
        | VoxValuePoolValues::Json(_)
        | VoxValuePoolValues::String(_) => {
            unreachable!("a component was validated against a vector shape")
        }
    }
}

/// Ramp samples of component `index` of each float vector `value_ids` draws.
fn float_components<const N: usize>(
    vectors: VoxValueColumn<'_, [f64; N]>,
    value_ids: &[U32Id<BVoxValuePoolValue>],
    index: usize,
) -> Vec<TreeGridJsonValue> {
    spelled(vectors, value_ids, |vector| {
        TreeGridJsonValue::unorm(vector[index])
    })
}

/// Ramp samples of component `index` of each int vector `value_ids` draws.
fn int_components<const N: usize>(
    vectors: VoxValueColumn<'_, [i64; N]>,
    value_ids: &[U32Id<BVoxValuePoolValue>],
    index: usize,
) -> Vec<TreeGridJsonValue> {
    spelled(vectors, value_ids, |vector| {
        TreeGridJsonValue::unorm(vector[index] as f64)
    })
}

fn spelled<T>(
    values: VoxValueColumn<'_, T>,
    value_ids: &[U32Id<BVoxValuePoolValue>],
    spell: impl Fn(&T) -> TreeGridJsonValue,
) -> Vec<TreeGridJsonValue> {
    drawn(values, value_ids).map(spell).collect()
}

/// The value each of `value_ids` draws from `values`, in material order.
fn drawn<'a, T>(
    values: VoxValueColumn<'a, T>,
    value_ids: &[U32Id<BVoxValuePoolValue>],
) -> impl Iterator<Item = &'a T> {
    value_ids.iter().map(move |&value_id| {
        values
            .get(value_id)
            .expect("a material draws a retained value")
    })
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
        Err(Error::invalid(format!(
            "property `{key}` holds a component outside [0, 1], which an sRGB reading \
             cannot spell; read it as `linear-float` or `plain`"
        )))
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
            map.entries()
                .iter()
                .map(|entry| (entry.key.clone(), vox_value_to_json(&entry.value)))
                .collect(),
        ),
    }
}

/// The palette's property keys joined for a not-found message.
fn available_keys(palette: &VoxPalette) -> String {
    property_names(palette).join(", ")
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
                let node_id = grid.retain_root(TreeGridLabel::bare(
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

                    _ => grid.retain_child(
                        palette_node_id,
                        TreeGridLabel::quoted(value_collection.key.as_str()),
                    ),
                };
                property_node = Some((value_collection.key, property_node_id));
                let letter = component.letter().to_string();
                grid.retain_child(property_node_id, TreeGridLabel::bare(letter))
            }

            None => {
                // A data node is always fresh, so a property selected twice
                // keeps one value collection per selector.
                let node_id = grid.retain_child(
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

/// Renders the grid under `options`, mapping them into treegrid's loose
/// options. Each render errors on an option it ignores, and only the `Rows`
/// render consumes a width.
fn render(grid: &TreeGrid<TreeGridJsonValueCells>, options: &PaletteShowOptions) -> Result<String> {
    let PaletteShowOptions {
        layout,
        label,
        header_level,
        table_shape,
        width,
    } = *options;
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
        PaletteShowLayout::BoxHierarchy => {
            grid.render_box_hierarchy(&resolve_options(options.resolve_box_hierarchy())?)
        }

        PaletteShowLayout::BoxTables => {
            grid.render_box_tables(resolve_options(options.resolve_box_tables())?)
        }

        PaletteShowLayout::JsonCompact => {
            resolve_options(options.resolve_json())?;
            grid.render_json_compact()
        }

        PaletteShowLayout::JsonPretty => {
            resolve_options(options.resolve_json())?;
            grid.render_json_pretty()
        }

        PaletteShowLayout::MdTables => {
            grid.render_md_tables(&resolve_options(options.resolve_md_tables())?)
        }

        PaletteShowLayout::TextColumns => {
            grid.render_text_columns(&resolve_options(options.resolve_text_columns())?)
        }

        PaletteShowLayout::TextRows => {
            if let Some(columns) = width {
                options = options.with_width(columns);
            }
            grid.render_text_rows(&resolve_options(options.resolve_text_rows())?)
        }
    })
}

/// Maps an invalid option combination into this crate's error.
fn resolve_options<T>(resolved: StdResult<T, TreeGridError>) -> Result<T> {
    Ok(resolved?)
}

#[cfg(test)]
mod tests {
    use crate::{
        operations::palette::{
            PaletteRef, PaletteShowLabel, PaletteShowLayout, PaletteShowOptions,
            PaletteShowPresentation, PaletteShowReading, PaletteShowTableShape, PropertyRef,
            PropertySelector, palette_show, palette_show::palette_show::resolve_value_collections,
        },
        utilities::VectorComponent,
    };
    use branded_id::U32Id;
    use serde_json::Value;
    use std::num::NonZeroU8;
    use ty_math::TySrgbaU8;
    use voxcore::{BVoxValuePool, BVoxValuePoolValue, VoxMain, VoxPalette, VoxValue, VoxValuePool};

    /// The branded value id `index`.
    fn value_id(index: usize) -> U32Id<BVoxValuePoolValue> {
        U32Id::from_u32(index as u32)
    }

    /// A `vec-4-float` value pool of the given 8-bit sRGB colors, each decoded
    /// to linear light, the way the importers store colors.
    fn lin_srgba_f64_value_pool_id(main: &mut VoxMain, colors: &[[u8; 4]]) -> U32Id<BVoxValuePool> {
        let values = colors
            .iter()
            .map(|&[red, green, blue, alpha]| {
                let linear = TySrgbaU8::new(red, green, blue, alpha)
                    .into_format::<f64, f64>()
                    .into_linear();
                [linear.red, linear.green, linear.blue, linear.alpha]
            })
            .collect();
        main.retain_value_pool(VoxValuePool::vec_4_float(values).unwrap())
    }

    /// A document with two palettes: palette 0 has `baseColor` and
    /// `metallic` with two materials, palette 1 has `baseColor` with
    /// one material.
    fn sample_main() -> VoxMain {
        let mut main: VoxMain = VoxMain::default();

        let colors_zero_value_pool_id =
            lin_srgba_f64_value_pool_id(&mut main, &[[255, 0, 0, 255], [0, 255, 0, 128]]);
        let metallic_value_pool_id =
            main.retain_value_pool(VoxValuePool::float(vec![1.0, 0.2]).unwrap());
        let colors_one_value_pool_id = lin_srgba_f64_value_pool_id(&mut main, &[[0, 0, 255, 255]]);

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
        main.retain_palette(first).unwrap();

        let mut second = VoxPalette::default();
        second
            .retain_property(
                "baseColor".to_owned(),
                colors_one_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        second.retain_material(vec![value_id(0)]).unwrap();
        main.retain_palette(second).unwrap();

        main
    }

    /// The selectors the CLI spells `<palette> <property> <presentation>
    /// <reading>`, in the fixtures' shorthand.
    fn selectors(fields: &[(&str, &str, &str, &str)]) -> Vec<PropertySelector> {
        fields
            .iter()
            .map(
                |&(palette, property, presentation, reading)| PropertySelector {
                    palette: match palette {
                        "*" => PaletteRef::All,
                        index => PaletteRef::Index(index.parse().unwrap()),
                    },
                    property: match property {
                        "*" => PropertyRef::All,

                        key => {
                            let (key, component) = match key.rsplit_once('.') {
                                Some((key, letter)) if letter.len() == 1 => {
                                    (key, Some(vector_component(letter)))
                                }

                                _ => (key, None),
                            };
                            PropertyRef::Key {
                                key: key.to_owned(),
                                component,
                            }
                        }
                    },
                    presentation: match presentation {
                        "auto" => PaletteShowPresentation::Auto,
                        "swatch" => PaletteShowPresentation::Swatch,
                        "swatch-value" => PaletteShowPresentation::SwatchValue,
                        "value" => PaletteShowPresentation::Value,
                        other => panic!("unknown presentation {other}"),
                    },
                    reading: match reading {
                        "auto" => PaletteShowReading::Auto,
                        "linear-float" => PaletteShowReading::LinearFloat,
                        "plain" => PaletteShowReading::Plain,
                        "srgb-float" => PaletteShowReading::SrgbFloat,
                        "srgb-hex" => PaletteShowReading::SrgbHex,
                        other => panic!("unknown reading {other}"),
                    },
                },
            )
            .collect()
    }

    fn vector_component(letter: &str) -> VectorComponent {
        match letter {
            "r" => VectorComponent::R,
            "g" => VectorComponent::G,
            "b" => VectorComponent::B,
            "a" => VectorComponent::A,
            "x" => VectorComponent::X,
            "y" => VectorComponent::Y,
            "z" => VectorComponent::Z,
            "w" => VectorComponent::W,
            other => panic!("unknown component {other}"),
        }
    }

    /// Renders the selectors under `options`.
    fn show_with(
        main: &VoxMain,
        fields: &[(&str, &str, &str, &str)],
        options: PaletteShowOptions,
    ) -> String {
        palette_show(main, &selectors(fields), &options).unwrap()
    }

    /// Renders the selectors under `layout` with default label options and no
    /// wrapping.
    fn show(
        main: &VoxMain,
        fields: &[(&str, &str, &str, &str)],
        layout: PaletteShowLayout,
    ) -> String {
        show_with(
            main,
            fields,
            PaletteShowOptions {
                layout,
                ..PaletteShowOptions::default()
            },
        )
    }

    #[test]
    fn value_presentation_prints_canonical_hex_with_a_label() {
        let main = sample_main();
        let output = show(
            &main,
            &[("0", "baseColor", "value", "auto")],
            PaletteShowLayout::TextRows,
        );
        assert_eq!(output, "0.\"baseColor\" #FF0000FF #00FF0080\n");
    }

    #[test]
    fn a_color_component_under_auto_spells_its_hex_pair() {
        let main = sample_main();
        let output = show(
            &main,
            &[("0", "baseColor.a", "value", "auto")],
            PaletteShowLayout::TextRows,
        );
        // A vocabulary color name reads srgb-hex, so the alpha bytes FF and
        // 80 spell their hex pairs.
        assert_eq!(output, "0.\"baseColor\".a FF 80\n");
    }

    #[test]
    fn swatch_presentation_abuts_swatches_into_a_strip() {
        let main = sample_main();
        let output = show(
            &main,
            &[("0", "baseColor", "swatch", "auto")],
            PaletteShowLayout::TextRows,
        );
        assert_eq!(
            output,
            "0.\"baseColor\" \x1b[48;2;255;0;0m  \x1b[0m\x1b[48;2;0;255;0m  \x1b[0m\n"
        );
    }

    #[test]
    fn swatch_spaces_values_with_no_swatch() {
        let mut main: VoxMain = VoxMain::default();
        let shadows_value_pool_id =
            main.retain_value_pool(VoxValuePool::boolean(vec![true, false]));
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
        main.retain_palette(palette).unwrap();

        // Bools have no swatch, so swatch format spaces them rather than
        // abutting them into `truefalse`.
        let output = show(
            &main,
            &[("0", "shadows", "swatch", "auto")],
            PaletteShowLayout::TextRows,
        );
        assert_eq!(output, "0.\"shadows\" true false\n");
    }

    #[test]
    fn rows_pad_only_the_label_not_the_values() {
        let main = sample_main();
        // The labels pad to the longest so each row's first value aligns, but
        // the values are not column-aligned: `metallic` stays compact
        // rather than padding out to the wider `baseColor` columns.
        let output = show(
            &main,
            &[
                ("0", "baseColor", "value", "auto"),
                ("0", "metallic", "value", "auto"),
            ],
            PaletteShowLayout::TextRows,
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
        let main = sample_main();
        // Width 30 leaves 16 columns after the `0."baseColor" ` prefix:
        // one 9-wide hex fits per line, so the second wraps under the first.
        let output = palette_show(
            &main,
            &selectors(&[("0", "baseColor", "value", "auto")]),
            &PaletteShowOptions {
                layout: PaletteShowLayout::TextRows,
                label: None,
                header_level: None,
                table_shape: None,
                width: Some(30),
            },
        )
        .unwrap();
        assert_eq!(
            output,
            "0.\"baseColor\" #FF0000FF\n              #00FF0080\n"
        );
    }

    #[test]
    fn rows_with_label_none_drop_the_label_column() {
        let main = sample_main();
        let output = palette_show(
            &main,
            &selectors(&[("0", "baseColor", "value", "auto")]),
            &PaletteShowOptions {
                layout: PaletteShowLayout::TextRows,
                label: Some(PaletteShowLabel::None),
                header_level: None,
                table_shape: None,
                width: None,
            },
        )
        .unwrap();
        assert_eq!(output, "#FF0000FF #00FF0080\n");
    }

    #[test]
    fn default_selector_shows_every_palette_and_property() {
        let main = sample_main();
        let output = palette_show(
            &main,
            &[PropertySelector::default()],
            &PaletteShowOptions::default(),
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
        let main = sample_main();
        // A palette revisited later starts a fresh root rather than merging
        // backward, so pre-order keeps the selector order.
        let output = show(
            &main,
            &[
                ("1", "baseColor", "value", "auto"),
                ("0", "baseColor", "value", "auto"),
            ],
            PaletteShowLayout::TextRows,
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
        let main = sample_main();
        let output = show(
            &main,
            &[
                ("0", "baseColor", "value", "auto"),
                ("0", "baseColor", "swatch", "auto"),
            ],
            PaletteShowLayout::TextRows,
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
        let main = sample_main();
        let output = show(
            &main,
            &[
                ("0", "baseColor.a", "value", "auto"),
                ("1", "baseColor.a", "value", "auto"),
            ],
            PaletteShowLayout::TextColumns,
        );
        assert_eq!(
            output,
            "0.\"baseColor\".a 1.\"baseColor\".a\nFF              FF\n80\n"
        );
    }

    #[test]
    fn columns_with_label_none_drop_the_label_row() {
        let main = sample_main();
        let output = palette_show(
            &main,
            &selectors(&[
                ("0", "baseColor.a", "value", "auto"),
                ("1", "baseColor.a", "value", "auto"),
            ]),
            &PaletteShowOptions {
                layout: PaletteShowLayout::TextColumns,
                label: Some(PaletteShowLabel::None),
                header_level: None,
                table_shape: None,
                width: None,
            },
        )
        .unwrap();
        assert_eq!(output, "FF FF\n80\n");
    }

    #[test]
    fn hierarchy_layout_renders_the_palette_tree() {
        let main = sample_main();
        let output = show(
            &main,
            &[("0", "*", "value", "auto")],
            PaletteShowLayout::BoxHierarchy,
        );
        assert_eq!(
            output,
            "└ 0\n  ├ \"baseColor\": #FF0000FF #00FF0080\n  └ \"metallic\": 1 0.2\n"
        );
    }

    #[test]
    fn header_labels_group_rows_under_palette_headings() {
        let main = sample_main();
        let output = palette_show(
            &main,
            &selectors(&[("*", "baseColor", "value", "auto")]),
            &PaletteShowOptions {
                layout: PaletteShowLayout::TextRows,
                label: Some(PaletteShowLabel::Header),
                header_level: None,
                table_shape: None,
                width: None,
            },
        )
        .unwrap();
        assert_eq!(
            output,
            "# 0\n\n\"baseColor\" #FF0000FF #00FF0080\n\n# 1\n\n\"baseColor\" #0000FFFF\n"
        );
    }

    #[test]
    fn a_header_level_shifts_the_headings() {
        let main = sample_main();
        let output = palette_show(
            &main,
            &selectors(&[("*", "baseColor", "value", "auto")]),
            &PaletteShowOptions {
                layout: PaletteShowLayout::TextRows,
                label: Some(PaletteShowLabel::Header),
                header_level: NonZeroU8::new(2),
                table_shape: None,
                width: None,
            },
        )
        .unwrap();
        assert!(output.starts_with("## 0\n"));
    }

    #[test]
    fn box_tables_group_one_box_per_palette_under_bare_lines() {
        let main = sample_main();
        let output = show(
            &main,
            &[("*", "baseColor", "value", "auto")],
            PaletteShowLayout::BoxTables,
        );
        assert_eq!(
            output,
            "0\n\
             \n\
             ┌───┬─────────────┐\n\
             │ # │ \"baseColor\" │\n\
             ├───┼─────────────┤\n\
             │ 0 │ #FF0000FF   │\n\
             ├───┼─────────────┤\n\
             │ 1 │ #00FF0080   │\n\
             └───┴─────────────┘\n\
             \n\
             1\n\
             \n\
             ┌───┬─────────────┐\n\
             │ # │ \"baseColor\" │\n\
             ├───┼─────────────┤\n\
             │ 0 │ #0000FFFF   │\n\
             └───┴─────────────┘\n"
        );
    }

    #[test]
    fn box_tables_reject_header_labels() {
        let main = sample_main();
        let result = palette_show(
            &main,
            &selectors(&[("*", "baseColor", "value", "auto")]),
            &PaletteShowOptions {
                layout: PaletteShowLayout::BoxTables,
                label: Some(PaletteShowLabel::Header),
                header_level: None,
                table_shape: None,
                width: None,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn nested_tables_group_one_table_per_palette() {
        let main = sample_main();
        let output = show(
            &main,
            &[("*", "baseColor", "value", "auto")],
            PaletteShowLayout::MdTables,
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
        let main = sample_main();
        let output = palette_show(
            &main,
            &selectors(&[("*", "baseColor", "value", "auto")]),
            &PaletteShowOptions {
                layout: PaletteShowLayout::MdTables,
                label: None,
                header_level: None,
                table_shape: Some(PaletteShowTableShape::Flat),
                width: None,
            },
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
        let main = sample_main();
        let output = palette_show(
            &main,
            &selectors(&[("*", "baseColor", "value", "auto")]),
            &PaletteShowOptions {
                layout: PaletteShowLayout::MdTables,
                label: None,
                header_level: None,
                table_shape: Some(PaletteShowTableShape::Records),
                width: None,
            },
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
        let main = sample_main();
        let output = palette_show(
            &main,
            &selectors(&[
                ("0", "baseColor", "value", "auto"),
                ("0", "baseColor.a", "value", "auto"),
            ]),
            &PaletteShowOptions {
                layout: PaletteShowLayout::MdTables,
                label: None,
                header_level: None,
                table_shape: Some(PaletteShowTableShape::Records),
                width: None,
            },
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
        let main = sample_main();
        let result = palette_show(
            &main,
            &selectors(&[("0", "baseColor", "value", "auto")]),
            &PaletteShowOptions {
                layout: PaletteShowLayout::BoxHierarchy,
                label: Some(PaletteShowLabel::Concat),
                header_level: None,
                table_shape: None,
                width: None,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn compact_json_nests_component_records_under_the_property() {
        let main = sample_main();
        let output = show(
            &main,
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
        let main = sample_main();
        let fields: &[(&str, &str, &str, &str)] = &[
            ("0", "baseColor", "value", "auto"),
            ("0", "baseColor.a", "value", "auto"),
        ];
        let pretty = show(&main, fields, PaletteShowLayout::JsonPretty);
        let compact = show(&main, fields, PaletteShowLayout::JsonCompact);
        // Indented, and carrying the same data as the compact form.
        assert!(pretty.contains("\n  "));
        let pretty_value: Value = serde_json::from_str(&pretty).unwrap();
        let compact_value: Value = serde_json::from_str(&compact).unwrap();
        assert_eq!(pretty_value, compact_value);
    }

    #[test]
    fn star_property_expands_to_every_property() {
        let main = sample_main();
        let value_collections =
            resolve_value_collections(&main, &selectors(&[("0", "*", "value", "auto")])).unwrap();
        let keys: Vec<&str> = value_collections.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(keys, ["baseColor", "metallic"]);
    }

    #[test]
    fn star_palette_skips_a_palette_lacking_a_named_property() {
        let main = sample_main();
        // Only palette 0 has `metallic`; palette 1 is skipped, not an error.
        let value_collections =
            resolve_value_collections(&main, &selectors(&[("*", "metallic", "value", "auto")]))
                .unwrap();
        let labels: Vec<(usize, &str)> = value_collections
            .iter()
            .map(|c| (c.palette_index, c.key.as_str()))
            .collect();
        assert_eq!(labels, [(0, "metallic")]);
    }

    #[test]
    fn named_palette_out_of_range_is_an_error() {
        let main = sample_main();
        assert!(
            resolve_value_collections(&main, &selectors(&[("5", "baseColor", "value", "auto")]))
                .is_err()
        );
    }

    #[test]
    fn named_property_absent_on_a_named_palette_is_an_error() {
        let main = sample_main();
        assert!(
            resolve_value_collections(&main, &selectors(&[("0", "missing", "value", "auto")]))
                .is_err()
        );
    }

    #[test]
    fn a_component_on_a_scalar_is_an_error() {
        let main = sample_main();
        assert!(
            resolve_value_collections(&main, &selectors(&[("0", "metallic.r", "value", "auto")]))
                .is_err()
        );
    }

    #[test]
    fn auto_presentation_prints_scalars_and_components_as_bare_text() {
        let main = sample_main();
        let scalar = show(
            &main,
            &[("0", "metallic", "auto", "auto")],
            PaletteShowLayout::TextRows,
        );
        assert_eq!(scalar, "0.\"metallic\" 1 0.2\n");
        let component = show(
            &main,
            &[("0", "baseColor.r", "auto", "auto")],
            PaletteShowLayout::TextRows,
        );
        assert_eq!(component, "0.\"baseColor\".r FF 00\n");
    }

    /// One palette binding `emissiveColor`, three components per the glTF
    /// vocabulary, to a `vec-3-float` value pool holding a red.
    fn three_component_main() -> VoxMain {
        let mut main: VoxMain = VoxMain::default();
        let emissive_value_pool_id =
            main.retain_value_pool(VoxValuePool::vec_3_float(vec![[1.0, 0.0, 0.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property(
                "emissiveColor".to_owned(),
                emissive_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        main.retain_palette(palette).unwrap();
        main
    }

    #[test]
    fn a_three_component_color_renders_hex_without_alpha() {
        let main = three_component_main();
        let output = show(
            &main,
            &[("0", "emissiveColor", "value", "auto")],
            PaletteShowLayout::TextRows,
        );
        // Six hex digits, no alpha pair.
        assert_eq!(output, "0.\"emissiveColor\" #FF0000\n");
    }

    #[test]
    fn a_three_component_color_has_no_alpha_component() {
        let main = three_component_main();
        // `.a` is out of the three-wide shape, but `.r` reads its hex pair.
        assert!(
            resolve_value_collections(
                &main,
                &selectors(&[("0", "emissiveColor.a", "value", "auto")]),
            )
            .is_err()
        );
        let red = show(
            &main,
            &[("0", "emissiveColor.r", "value", "auto")],
            PaletteShowLayout::TextRows,
        );
        assert_eq!(red, "0.\"emissiveColor\".r FF\n");
    }

    #[test]
    fn an_hdr_vocabulary_color_errors_under_auto() {
        let mut main: VoxMain = VoxMain::default();
        // The vocabulary bounds a glTF color at [0, 1], so auto errors on the
        // HDR red and an explicit reading spells the exact stored values.
        let emissive_value_pool_id =
            main.retain_value_pool(VoxValuePool::vec_4_float(vec![[2.0, 1.0, 0.5, 1.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property(
                "emissiveColor".to_owned(),
                emissive_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        main.retain_palette(palette).unwrap();

        assert!(
            resolve_value_collections(
                &main,
                &selectors(&[("0", "emissiveColor", "value", "auto")]),
            )
            .is_err()
        );
        let output = show(
            &main,
            &[("0", "emissiveColor", "value", "linear-float")],
            PaletteShowLayout::TextRows,
        );
        assert_eq!(output, "0.\"emissiveColor\" lin_srgba(2, 1, 0.5, 1)\n");
    }

    #[test]
    fn a_vocabulary_color_off_its_shape_errors_under_auto() {
        let mut main: VoxMain = VoxMain::default();
        // `baseColor` names a glTF color, so a binding no color reading
        // spells is an error under auto, whatever the shape holds.
        let base_color_value_pool_id =
            main.retain_value_pool(VoxValuePool::vec_3_int(vec![[1, 0, 0]]).unwrap());
        let strength_value_pool_id =
            main.retain_value_pool(VoxValuePool::float(vec![1.0]).unwrap());
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
        main.retain_palette(palette).unwrap();

        for key in ["baseColor", "emissiveColor"] {
            assert!(
                resolve_value_collections(&main, &selectors(&[("0", key, "value", "auto")]))
                    .is_err()
            );
        }
    }

    /// One palette binding the custom `tint`, a key outside the glTF
    /// vocabulary, to a `vec-3-float` value pool holding a red.
    fn custom_vector_main() -> VoxMain {
        let mut main: VoxMain = VoxMain::default();
        let tint_value_pool_id =
            main.retain_value_pool(VoxValuePool::vec_3_float(vec![[1.0, 0.0, 0.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property("tint".to_owned(), tint_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        main.retain_palette(palette).unwrap();
        main
    }

    #[test]
    fn a_custom_float_vector_defaults_to_plain_numbers() {
        let main = custom_vector_main();
        // A custom vec-3-float could hold a color or a normal, so it renders
        // plain numbers. A component reads its stored float, and an index
        // past the width errors.
        let output = show(
            &main,
            &[("0", "tint", "value", "auto")],
            PaletteShowLayout::TextRows,
        );
        assert_eq!(output, "0.\"tint\" [1,0,0]\n");
        let component = show(
            &main,
            &[("0", "tint.r", "value", "auto")],
            PaletteShowLayout::TextRows,
        );
        assert_eq!(component, "0.\"tint\".r 1\n");
        assert!(
            resolve_value_collections(&main, &selectors(&[("0", "tint.w", "value", "auto")]))
                .is_err()
        );
    }

    #[test]
    fn an_srgb_hex_reading_asserts_color_for_a_custom_key() {
        let main = custom_vector_main();
        let output = show(
            &main,
            &[
                ("0", "tint", "value", "srgb-hex"),
                ("0", "tint.r", "value", "srgb-hex"),
            ],
            PaletteShowLayout::TextRows,
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
        let main = sample_main();
        // `metallic` binds a float value pool, which no color reading spells.
        for reading in ["linear-float", "srgb-float", "srgb-hex"] {
            assert!(
                resolve_value_collections(
                    &main,
                    &selectors(&[("0", "metallic", "value", reading)])
                )
                .is_err()
            );
        }
    }

    /// One palette binding the custom `tint` to a `vec-4-float` value pool
    /// holding `[1, 0, 0.25, 0.5]`, the design page's worked example.
    fn custom_tint_vec_4_main() -> VoxMain {
        let mut main: VoxMain = VoxMain::default();
        let tint_value_pool_id =
            main.retain_value_pool(VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.25, 0.5]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property("tint".to_owned(), tint_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        main.retain_palette(palette).unwrap();
        main
    }

    #[test]
    fn the_readings_spell_one_color_per_the_design_examples() {
        let main = custom_tint_vec_4_main();
        let row = |fields| show(&main, &[fields], PaletteShowLayout::TextRows);
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
        let mut main: VoxMain = VoxMain::default();
        // The sRGB readings never clamp: the HDR red errors under both, and
        // the auto fallback stays available through linear-float.
        let tint_value_pool_id =
            main.retain_value_pool(VoxValuePool::vec_4_float(vec![[2.0, 1.0, 0.5, 1.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property("tint".to_owned(), tint_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        main.retain_palette(palette).unwrap();

        for reading in ["srgb-hex", "srgb-float"] {
            assert!(
                resolve_value_collections(&main, &selectors(&[("0", "tint", "value", reading)]))
                    .is_err()
            );
        }
        let output = show(
            &main,
            &[("0", "tint", "value", "linear-float")],
            PaletteShowLayout::TextRows,
        );
        assert_eq!(output, "0.\"tint\" lin_srgba(2, 1, 0.5, 1)\n");
    }

    #[test]
    fn a_component_reads_any_vector_shape() {
        let mut main: VoxMain = VoxMain::default();
        let position_value_pool_id =
            main.retain_value_pool(VoxValuePool::vec_3_int(vec![[3, 7, 2]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property(
                "position".to_owned(),
                position_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        main.retain_palette(palette).unwrap();

        // An int vector's component reads its stored int; the width still
        // bounds the index, and no color reading spells an int vector.
        let output = show(
            &main,
            &[("0", "position.x", "value", "auto")],
            PaletteShowLayout::TextRows,
        );
        assert_eq!(output, "0.\"position\".x 3\n");
        assert!(
            resolve_value_collections(&main, &selectors(&[("0", "position.w", "value", "auto")]))
                .is_err()
        );
        assert!(
            resolve_value_collections(
                &main,
                &selectors(&[("0", "position.x", "value", "srgb-hex")]),
            )
            .is_err()
        );
    }

    #[test]
    fn an_int_value_pool_renders_integers() {
        let mut main: VoxMain = VoxMain::default();
        let count_value_pool_id = main.retain_value_pool(VoxValuePool::int(vec![3, 7]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property("count".to_owned(), count_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        palette.retain_material(vec![value_id(1)]).unwrap();
        main.retain_palette(palette).unwrap();

        let output = show(
            &main,
            &[("0", "count", "value", "auto")],
            PaletteShowLayout::TextRows,
        );
        assert_eq!(output, "0.\"count\" 3 7\n");
    }

    #[test]
    fn a_json_value_pool_renders_arrays_rather_than_null() {
        let mut main: VoxMain = VoxMain::default();
        let extra_value_pool_id = main.retain_value_pool(VoxValuePool::json(vec![
            VoxValue::Array(vec![VoxValue::Number(1.0), VoxValue::Number(2.0)]),
        ]));
        let mut palette = VoxPalette::default();
        palette
            .retain_property("extra".to_owned(), extra_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        main.retain_palette(palette).unwrap();

        // The array survives into both the text and JSON layouts.
        let text = show(
            &main,
            &[("0", "extra", "value", "auto")],
            PaletteShowLayout::TextRows,
        );
        assert_eq!(text, "0.\"extra\" [1,2]\n");
        let json = show(
            &main,
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
        let mut main: VoxMain = VoxMain::default();
        let value_pool_id = main.retain_value_pool(VoxValuePool::boolean(vec![true]));
        let mut palette = VoxPalette::default();
        // A binding with no property name, reached through the `*` property.
        palette
            .retain_property(String::new(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![value_id(0)]).unwrap();
        main.retain_palette(palette).unwrap();

        // An empty name prints quoted as `""` rather than vanishing after the
        // `0.` prefix.
        let row = show(
            &main,
            &[("0", "*", "value", "auto")],
            PaletteShowLayout::TextRows,
        );
        assert_eq!(row, "0.\"\" true\n");
        // JSON keeps the raw name; its own string quoting is enough there.
        let json = show(
            &main,
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
        let mut main: VoxMain = VoxMain::default();
        let strengths_value_pool_id =
            main.retain_value_pool(VoxValuePool::float(vec![2.0]).unwrap());
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
        main.retain_palette(palette).unwrap();

        let output = show(
            &main,
            &[("0", "emissiveStrength", "value", "auto")],
            PaletteShowLayout::TextRows,
        );
        assert_eq!(output, "0.\"emissiveStrength\" 2 2\n");
    }
}
