use crate::{
    Format, Result, SelectIndex,
    commands::{PaletteListFields, PaletteListLayout},
    implementation::{self, CONNECTOR_LAST, CONNECTOR_MID, EXTENSION_LAST, EXTENSION_MID},
};
use branded_id::U32Id;
use serde_json::{Map, Value, json};
use std::{
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use voxcore::{BVoxPalette, VoxMain, VoxPalette};

/// A selected palette paired with its id, the working unit the renderers walk.
type Entry<'a> = (U32Id<BVoxPalette>, &'a VoxPalette);

/// Loads the voxel file at `input` and prints a per-palette overview: each
/// palette's index and, when enabled by `fields`, its property keys, material
/// count, and referencing objects. `filters` narrows the palettes and `layout`
/// chooses the Markdown table, the hierarchy tree, or a JSON form.
pub fn palette_list(
    input: &Path,
    from: Option<Format>,
    filters: &[SelectIndex],
    fields: PaletteListFields,
    layout: PaletteListLayout,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;
    let palettes = select_palettes(&state, filters)?;
    let output = render(&state, &palettes, fields, layout)?;
    implementation::write_stdout(output.as_bytes())
}

/// The palettes to list: every palette whose index matches any of `filters`, in
/// index order, or every palette when `filters` is empty. Errors when `filters`
/// is given but matches no palette, so a stray index is caught.
fn select_palettes<'a>(state: &'a VoxMain, filters: &[SelectIndex]) -> Result<Vec<Entry<'a>>> {
    let selected: Vec<Entry> = state
        .iter_palettes()
        .filter(|(id, _)| {
            filters.is_empty()
                || filters
                    .iter()
                    .any(|filter| filter.contains(id.to_u32() as usize))
        })
        .collect();
    if !filters.is_empty() && selected.is_empty() {
        return Err(IOError::new(
            ErrorKind::InvalidInput,
            "no palettes match the given filters",
        )
        .into());
    }
    Ok(selected)
}

/// Renders `palettes` in `layout`, the testable core of [`palette_list`].
fn render(
    state: &VoxMain,
    palettes: &[Entry],
    fields: PaletteListFields,
    layout: PaletteListLayout,
) -> Result<String> {
    match layout {
        PaletteListLayout::Markdown => Ok(render_markdown(state, palettes, fields)),
        PaletteListLayout::Hierarchy => Ok(render_hierarchy(state, palettes, fields)),
        PaletteListLayout::PrettyJson => render_json(state, palettes, fields, true),
        PaletteListLayout::CompactJson => render_json(state, palettes, fields, false),
    }
}

/// The listing as one aligned table, a column per enabled field beside the
/// always-shown index.
fn render_markdown(state: &VoxMain, palettes: &[Entry], fields: PaletteListFields) -> String {
    let mut headers = vec!["index"];
    if fields.properties {
        headers.push("properties");
    }
    if fields.materials {
        headers.push("materials");
    }
    if fields.objects {
        headers.push("used by");
    }

    let rows: Vec<Vec<String>> = palettes
        .iter()
        .map(|(id, palette)| {
            let mut columns = vec![id.to_u32().to_string()];
            if fields.properties {
                columns.push(implementation::md_cell(
                    &implementation::property_labels(palette).join(", "),
                ));
            }
            if fields.materials {
                columns.push(palette.material_count().to_string());
            }
            if fields.objects {
                columns.push(implementation::md_cell(
                    &referencing_names(state, *id).join(", "),
                ));
            }
            columns
        })
        .collect();
    implementation::markdown_table(&headers, &rows)
}

/// The listing as a JSON array, pretty or compact: one record per palette in
/// index order, its index and each enabled field.
fn render_json(
    state: &VoxMain,
    palettes: &[Entry],
    fields: PaletteListFields,
    pretty: bool,
) -> Result<String> {
    let records: Vec<Value> = palettes
        .iter()
        .map(|(id, palette)| {
            let mut entry = Map::new();
            entry.insert("index".to_string(), json!(id.to_u32()));
            if fields.properties {
                entry.insert(
                    "properties".to_string(),
                    json!(implementation::property_names(palette)),
                );
                let scalars = implementation::scalar_property_names(palette);
                if !scalars.is_empty() {
                    entry.insert("scalar_properties".to_string(), json!(scalars));
                }
            }
            if fields.materials {
                entry.insert("materials".to_string(), json!(palette.material_count()));
            }
            if fields.objects {
                let used_by: Vec<u32> = referencing_objects(state, *id)
                    .into_iter()
                    .map(|(index, _)| index)
                    .collect();
                entry.insert("used_by".to_string(), json!(used_by));
            }
            Value::Object(entry)
        })
        .collect();
    implementation::to_json_string(&Value::Array(records), pretty)
}

/// The listing as an indented tree, like `hierarchy show`: a `palettes` header
/// over one bare-index branch per palette, each carrying its enabled fields as
/// child branches in the order material count, properties, objects.
fn render_hierarchy(state: &VoxMain, palettes: &[Entry], fields: PaletteListFields) -> String {
    let mut output = String::from("palettes\n");
    let total = palettes.len();
    for (index, (id, palette)) in palettes.iter().enumerate() {
        let last = index + 1 == total;
        let connector = if last { CONNECTOR_LAST } else { CONNECTOR_MID };
        output.push_str(&format!("{connector} {}\n", id.to_u32()));

        let child_prefix = if last { EXTENSION_LAST } else { EXTENSION_MID };
        let mut children = Vec::new();
        if fields.materials {
            children.push(HierarchyChild::MaterialCount(palette.material_count()));
        }
        if fields.properties {
            children.push(HierarchyChild::Names(
                "properties",
                implementation::property_labels(palette),
            ));
        }
        if fields.objects {
            children.push(HierarchyChild::Names(
                "objects",
                referencing_names(state, *id)
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
            ));
        }

        let child_total = children.len();
        for (child_index, child) in children.iter().enumerate() {
            let child_last = child_index + 1 == child_total;
            match child {
                HierarchyChild::MaterialCount(count) => {
                    let connector = if child_last {
                        CONNECTOR_LAST
                    } else {
                        CONNECTOR_MID
                    };
                    output.push_str(&format!(
                        "{child_prefix}{connector} materialCount: {count}\n"
                    ));
                }
                HierarchyChild::Names(header, names) => {
                    render_names_subtree(&mut output, child_prefix, child_last, header, names);
                }
            }
        }
    }
    output
}

/// One enabled child under a palette in the `hierarchy` layout. Collected before
/// rendering so the last enabled child takes the closing connector.
enum HierarchyChild {
    /// A `materialCount: <n>` leaf.
    MaterialCount(usize),
    /// A named subtree, `properties` or `objects`, with one child per name.
    Names(&'static str, Vec<String>),
}

/// Appends a `header` subtree under `prefix` with one bare child per name, or a
/// `header: []` leaf when `names` is empty, matching the `hierarchy show` idiom.
fn render_names_subtree(
    output: &mut String,
    prefix: &str,
    is_last: bool,
    header: &str,
    names: &[String],
) {
    let connector = if is_last {
        CONNECTOR_LAST
    } else {
        CONNECTOR_MID
    };
    if names.is_empty() {
        output.push_str(&format!("{prefix}{connector} {header}: []\n"));
        return;
    }
    output.push_str(&format!("{prefix}{connector} {header}\n"));

    let extension = if is_last {
        EXTENSION_LAST
    } else {
        EXTENSION_MID
    };
    let inner = format!("{prefix}{extension}");
    let total = names.len();
    for (index, name) in names.iter().enumerate() {
        let connector = if index + 1 == total {
            CONNECTOR_LAST
        } else {
            CONNECTOR_MID
        };
        output.push_str(&format!("{inner}{connector} {name}\n"));
    }
}

/// The objects that reference `palette`, in object order, as `(index, name)`.
/// An object appears once however many of its layers reference the palette.
fn referencing_objects(state: &VoxMain, palette: U32Id<BVoxPalette>) -> Vec<(u32, &str)> {
    state
        .iter_objects()
        .filter(|(_, object)| object.iter_layers().any(|(_, id)| id == palette))
        .map(|(id, object)| (id.to_u32(), object.name()))
        .collect()
}

/// The names of the objects that reference `palette`, in object order.
fn referencing_names(state: &VoxMain, palette: U32Id<BVoxPalette>) -> Vec<&str> {
    referencing_objects(state, palette)
        .into_iter()
        .map(|(_, name)| name)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        SelectIndex,
        commands::{PaletteListFields, PaletteListLayout},
        implementation::palette_list::{render, select_palettes},
    };
    use branded_id::{IdVec, U32Id};
    use serde_json::Value;
    use ty_math::TyVector3U32;
    use voxcore::{BVoxPoolValue, VoxBound, VoxMain, VoxObject, VoxPalette, VoxValuePool};

    /// The branded value id `index`.
    fn value(index: usize) -> U32Id<BVoxPoolValue> {
        U32Id::from_u32(index as u32)
    }

    /// Every field enabled, the bare-`palette list` default.
    fn all_fields() -> PaletteListFields {
        PaletteListFields {
            properties: true,
            materials: true,
            objects: true,
        }
    }

    /// Two palettes and two objects: `a` samples palette 0, `b` samples both.
    /// Palette 0 carries `baseColorFactor` and `metallicFactor` with two
    /// materials, palette 1 carries `baseColorFactor` with one material plus a
    /// scalar `emissiveStrength`.
    fn shared_state() -> VoxMain {
        let mut state = VoxMain::default();

        // Colors and metallic values back the properties; only the property
        // names and material counts reach the listing, so the values are
        // arbitrary.
        let colors = state.add_value_pool(VoxValuePool::Srgba {
            values: IdVec::from_vec(vec![
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
                [0.0, 0.0, 1.0, 1.0],
            ]),
        });
        let metallic = state.add_value_pool(VoxValuePool::Float {
            min: VoxBound::Number(0.0),
            max: VoxBound::Number(1.0),
            values: IdVec::from_vec(vec![0.0, 1.0]),
        });

        let mut zero = VoxPalette::default();
        zero.add_array_property("baseColorFactor".to_owned(), colors);
        zero.add_array_property("metallicFactor".to_owned(), metallic);
        let zero_material = zero.add_material(vec![value(0), value(0)]).unwrap();
        zero.add_material(vec![value(1), value(1)]).unwrap();
        let zero = state.add_palette(zero);

        let mut one = VoxPalette::default();
        one.add_array_property("baseColorFactor".to_owned(), colors);
        one.add_scalar_property("emissiveStrength".to_owned(), metallic, value(1));
        let one_material = one.add_material(vec![value(2)]).unwrap();
        let one = state.add_palette(one);

        let mut a = VoxObject::new("a".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        a.add_layer(zero, zero_material);
        state.add_object(a);

        let mut b = VoxObject::new("b".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        b.add_layer(zero, zero_material);
        b.add_layer(one, one_material);
        state.add_object(b);

        state
    }

    /// Renders every palette of `state` with all fields in `layout`.
    fn render_all(state: &VoxMain, layout: PaletteListLayout) -> String {
        let palettes = select_palettes(state, &[]).unwrap();
        render(state, &palettes, all_fields(), layout).unwrap()
    }

    #[test]
    fn markdown_lists_one_row_per_palette() {
        assert_eq!(
            render_all(&shared_state(), PaletteListLayout::Markdown),
            "| index | properties                                 | materials | used by |\n\
             | ----- | ------------------------------------------ | --------- | ------- |\n\
             | 0     | baseColorFactor, metallicFactor            | 2         | a, b    |\n\
             | 1     | baseColorFactor, emissiveStrength (scalar) | 1         | b       |\n"
        );
    }

    #[test]
    fn markdown_drops_a_disabled_field_column() {
        let state = shared_state();
        let palettes = select_palettes(&state, &[]).unwrap();
        let fields = PaletteListFields {
            properties: true,
            materials: true,
            objects: false,
        };
        let output = render(&state, &palettes, fields, PaletteListLayout::Markdown).unwrap();
        assert_eq!(
            output,
            "| index | properties                                 | materials |\n\
             | ----- | ------------------------------------------ | --------- |\n\
             | 0     | baseColorFactor, metallicFactor            | 2         |\n\
             | 1     | baseColorFactor, emissiveStrength (scalar) | 1         |\n"
        );
    }

    #[test]
    fn hierarchy_nests_fields_under_each_palette() {
        assert_eq!(
            render_all(&shared_state(), PaletteListLayout::Hierarchy),
            "palettes\n\
             ├ 0\n\
             │ ├ materialCount: 2\n\
             │ ├ properties\n\
             │ │ ├ baseColorFactor\n\
             │ │ └ metallicFactor\n\
             │ └ objects\n\
             │   ├ a\n\
             │   └ b\n\
             └ 1\n\
             \u{20}\u{20}├ materialCount: 1\n\
             \u{20}\u{20}├ properties\n\
             \u{20}\u{20}│ ├ baseColorFactor\n\
             \u{20}\u{20}│ └ emissiveStrength (scalar)\n\
             \u{20}\u{20}└ objects\n\
             \u{20}\u{20}\u{20}\u{20}└ b\n"
        );
    }

    #[test]
    fn hierarchy_drops_a_disabled_field_branch() {
        let state = shared_state();
        let palettes = select_palettes(&state, &[]).unwrap();
        let fields = PaletteListFields {
            properties: false,
            materials: true,
            objects: true,
        };
        let output = render(&state, &palettes, fields, PaletteListLayout::Hierarchy).unwrap();
        assert_eq!(
            output,
            "palettes\n\
             ├ 0\n\
             │ ├ materialCount: 2\n\
             │ └ objects\n\
             │   ├ a\n\
             │   └ b\n\
             └ 1\n\
             \u{20}\u{20}├ materialCount: 1\n\
             \u{20}\u{20}└ objects\n\
             \u{20}\u{20}\u{20}\u{20}└ b\n"
        );
    }

    #[test]
    fn compact_json_records_indices_properties_materials_and_users() {
        assert_eq!(
            render_all(&shared_state(), PaletteListLayout::CompactJson),
            "[{\"index\":0,\"properties\":[\"baseColorFactor\",\"metallicFactor\"],\"materials\":2,\"used_by\":[0,1]},\
             {\"index\":1,\"properties\":[\"baseColorFactor\",\"emissiveStrength\"],\
             \"scalar_properties\":[\"emissiveStrength\"],\"materials\":1,\"used_by\":[1]}]\n"
        );
    }

    #[test]
    fn pretty_json_is_multiline_and_matches_compact() {
        let state = shared_state();
        let pretty = render_all(&state, PaletteListLayout::PrettyJson);
        let compact = render_all(&state, PaletteListLayout::CompactJson);
        assert!(pretty.starts_with("[\n"));
        let pretty_value: Value = serde_json::from_str(&pretty).unwrap();
        let compact_value: Value = serde_json::from_str(&compact).unwrap();
        assert_eq!(pretty_value, compact_value);
    }

    #[test]
    fn a_filter_lists_only_the_matching_palettes() {
        let state = shared_state();
        let filters = ["1".parse::<SelectIndex>().unwrap()];
        let palettes = select_palettes(&state, &filters).unwrap();
        let output = render(&state, &palettes, all_fields(), PaletteListLayout::Markdown).unwrap();
        assert_eq!(
            output,
            "| index | properties                                 | materials | used by |\n\
             | ----- | ------------------------------------------ | --------- | ------- |\n\
             | 1     | baseColorFactor, emissiveStrength (scalar) | 1         | b       |\n"
        );
    }

    #[test]
    fn a_range_filter_unions_its_indices() {
        let state = shared_state();
        let filters = ["0-5".parse::<SelectIndex>().unwrap()];
        let palettes = select_palettes(&state, &filters).unwrap();
        assert_eq!(palettes.len(), 2);
    }

    #[test]
    fn a_filter_matching_no_palette_errors() {
        let state = shared_state();
        let filters = ["9".parse::<SelectIndex>().unwrap()];
        assert!(select_palettes(&state, &filters).is_err());
    }

    #[test]
    fn an_unreferenced_palette_lists_no_users() {
        let mut state = VoxMain::default();
        let colors = state.add_value_pool(VoxValuePool::Srgba {
            values: IdVec::from_vec(vec![[1.0, 1.0, 1.0, 1.0]]),
        });
        let mut palette = VoxPalette::default();
        palette.add_array_property("baseColorFactor".to_owned(), colors);
        palette.add_material(vec![value(0)]).unwrap();
        state.add_palette(palette);

        assert_eq!(
            render_all(&state, PaletteListLayout::Markdown),
            "| index | properties      | materials | used by |\n\
             | ----- | --------------- | --------- | ------- |\n\
             | 0     | baseColorFactor | 1         |         |\n"
        );

        assert_eq!(
            render_all(&state, PaletteListLayout::CompactJson),
            "[{\"index\":0,\"properties\":[\"baseColorFactor\"],\"materials\":1,\"used_by\":[]}]\n"
        );
    }

    #[test]
    fn an_unreferenced_palette_shows_an_empty_objects_branch() {
        let mut state = VoxMain::default();
        let colors = state.add_value_pool(VoxValuePool::Srgba {
            values: IdVec::from_vec(vec![[1.0, 1.0, 1.0, 1.0]]),
        });
        let mut palette = VoxPalette::default();
        palette.add_array_property("baseColorFactor".to_owned(), colors);
        palette.add_material(vec![value(0)]).unwrap();
        state.add_palette(palette);

        assert_eq!(
            render_all(&state, PaletteListLayout::Hierarchy),
            "palettes\n\
             └ 0\n\
             \u{20}\u{20}├ materialCount: 1\n\
             \u{20}\u{20}├ properties\n\
             \u{20}\u{20}│ └ baseColorFactor\n\
             \u{20}\u{20}└ objects: []\n"
        );
    }

    #[test]
    fn a_scalar_only_palette_lists_its_pinned_property() {
        // No materials at all: the palette is never sampled, but its pinned
        // property still lists.
        let mut state = VoxMain::default();
        let strengths = state.add_value_pool(VoxValuePool::Float {
            min: VoxBound::Number(0.0),
            max: VoxBound::None,
            values: IdVec::from_vec(vec![2.0]),
        });
        let mut palette = VoxPalette::default();
        palette.add_scalar_property("emissiveStrength".to_owned(), strengths, value(0));
        state.add_palette(palette);

        assert_eq!(
            render_all(&state, PaletteListLayout::Markdown),
            "| index | properties                | materials | used by |\n\
             | ----- | ------------------------- | --------- | ------- |\n\
             | 0     | emissiveStrength (scalar) | 0         |         |\n"
        );

        assert_eq!(
            render_all(&state, PaletteListLayout::CompactJson),
            "[{\"index\":0,\"properties\":[\"emissiveStrength\"],\
             \"scalar_properties\":[\"emissiveStrength\"],\"materials\":0,\"used_by\":[]}]\n"
        );
    }
}
