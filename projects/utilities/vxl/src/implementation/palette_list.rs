use crate::{
    Format, Result, SelectIndex,
    commands::{PaletteListFields, PaletteListLayout},
    implementation,
};
use branded_id::U32Id;
use std::{
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use treegrid::{
    BTreeGridNode, TreeGrid, TreeGridHierarchyOptions, TreeGridJsonValue, TreeGridJsonValueCells,
    TreeGridLabel, TreeGridRecordsTableOptions, TreeGridRenderHierarchy, TreeGridRenderJson,
    TreeGridRenderTables, TreeGridTableShape, TreeGridValue,
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
    let output = render(&state, &palettes, fields, layout);
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
) -> String {
    match layout {
        PaletteListLayout::Tables => build_records_grid(state, palettes, fields).render_tables(
            &TreeGridTableShape::Records(TreeGridRecordsTableOptions::default()),
        ),
        PaletteListLayout::Hierarchy => build_grid(state, palettes, fields)
            .render_hierarchy(&TreeGridHierarchyOptions::default().with_bare_roots(true)),
        PaletteListLayout::JsonPretty => build_grid(state, palettes, fields).render_json_pretty(),
        PaletteListLayout::JsonCompact => build_grid(state, palettes, fields).render_json_compact(),
    }
}

/// The listing tree the hierarchy and JSON layouts share: a bare `palettes`
/// root over one bare-index branch per palette, its enabled fields as child
/// branches.
fn build_grid(
    state: &VoxMain,
    palettes: &[Entry],
    fields: PaletteListFields,
) -> TreeGrid<TreeGridJsonValueCells> {
    let mut grid = TreeGrid::with_cells(TreeGridJsonValueCells);
    let root = grid.add_root(TreeGridLabel::bare("palettes"));
    for (id, palette) in palettes {
        let branch = grid.add_child(root, TreeGridLabel::bare(id.to_u32().to_string()));
        if fields.materials {
            let count = grid.add_child(branch, TreeGridLabel::bare("materialCount"));
            grid.push_value(
                count,
                TreeGridJsonValue::int(palette.material_count() as i64),
            );
        }
        if fields.properties {
            let names = implementation::property_names(palette)
                .into_iter()
                .map(|name| TreeGridLabel::quoted(name.to_owned()))
                .collect();
            add_names_subtree(&mut grid, branch, "properties", names);
        }
        if fields.objects {
            let names = referencing_names(state, *id)
                .into_iter()
                .map(TreeGridLabel::quoted)
                .collect();
            add_names_subtree(&mut grid, branch, "objects", names);
        }
    }
    grid
}

/// The flat forest the markdown table renders: the `palettes` root over one
/// row per palette, each enabled field one data child whose comma-joined cell
/// text is baked as a single value, pushed even when empty so every enabled
/// column appears in every row.
fn build_records_grid(state: &VoxMain, palettes: &[Entry], fields: PaletteListFields) -> TreeGrid {
    let mut grid = TreeGrid::new();
    let root = grid.add_root(TreeGridLabel::bare("palettes"));
    for (id, palette) in palettes {
        let row = grid.add_child(root, TreeGridLabel::bare(id.to_u32().to_string()));
        if fields.properties {
            let cell = implementation::property_names(palette).join(", ");
            let node = grid.add_child(row, TreeGridLabel::bare("properties"));
            grid.push_value(node, TreeGridValue::new(cell));
        }
        if fields.materials {
            let node = grid.add_child(row, TreeGridLabel::bare("materials"));
            grid.push_value(
                node,
                TreeGridValue::new(palette.material_count().to_string()),
            );
        }
        if fields.objects {
            let node = grid.add_child(row, TreeGridLabel::bare("used by"));
            grid.push_value(
                node,
                TreeGridValue::new(referencing_names(state, *id).join(", ")),
            );
        }
    }
    grid
}

/// Adds a `header` subtree under `parent` with one child per label, or a
/// `header: []` leaf when `labels` is empty.
fn add_names_subtree(
    grid: &mut TreeGrid<TreeGridJsonValueCells>,
    parent: U32Id<BTreeGridNode>,
    header: &str,
    labels: Vec<TreeGridLabel>,
) {
    let subtree = grid.add_child(parent, TreeGridLabel::bare(header));
    if labels.is_empty() {
        grid.push_value(subtree, TreeGridJsonValue::new("[]"));
        return;
    }
    for label in labels {
        grid.add_child(subtree, label);
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
    use branded_id::U32Id;
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
    /// materials, palette 1 carries `baseColorFactor` and `emissiveStrength`
    /// with one material.
    fn shared_state() -> VoxMain {
        let mut state = VoxMain::default();

        // Colors and metallic values back the properties; only the property
        // names and material counts reach the listing, so the values are
        // arbitrary.
        let colors = state.add_value_pool(
            VoxValuePool::srgba(vec![
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
                [0.0, 0.0, 1.0, 1.0],
            ])
            .unwrap(),
        );
        let metallic = state.add_value_pool(
            VoxValuePool::float(VoxBound::Number(0.0), VoxBound::Number(1.0), vec![0.0, 1.0])
                .unwrap(),
        );

        let mut zero = VoxPalette::default();
        zero.add_property("baseColorFactor".to_owned(), colors, U32Id::from_u32(0))
            .unwrap();
        zero.add_property("metallicFactor".to_owned(), metallic, U32Id::from_u32(0))
            .unwrap();
        let zero_material = zero.add_material(vec![value(0), value(0)]).unwrap();
        zero.add_material(vec![value(1), value(1)]).unwrap();
        let zero = state.add_palette(zero).unwrap();

        let mut one = VoxPalette::default();
        one.add_property("baseColorFactor".to_owned(), colors, U32Id::from_u32(0))
            .unwrap();
        one.add_property("emissiveStrength".to_owned(), metallic, U32Id::from_u32(0))
            .unwrap();
        let one_material = one.add_material(vec![value(2), value(1)]).unwrap();
        let one = state.add_palette(one).unwrap();

        let mut a = VoxObject::new("a".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        a.add_layer(zero, zero_material);
        state.add_object(a).unwrap();

        let mut b = VoxObject::new("b".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        b.add_layer(zero, zero_material);
        b.add_layer(one, one_material);
        state.add_object(b).unwrap();

        state
    }

    /// Renders every palette of `state` with all fields in `layout`.
    fn render_all(state: &VoxMain, layout: PaletteListLayout) -> String {
        let palettes = select_palettes(state, &[]).unwrap();
        render(state, &palettes, all_fields(), layout)
    }

    #[test]
    fn tables_lists_one_row_per_palette() {
        assert_eq!(
            render_all(&shared_state(), PaletteListLayout::Tables),
            "# palettes\n\
             \n\
             | label | properties                        | materials | used by |\n\
             | ----- | --------------------------------- | --------- | ------- |\n\
             | 0     | baseColorFactor, metallicFactor   | 2         | a, b    |\n\
             | 1     | baseColorFactor, emissiveStrength | 1         | b       |\n"
        );
    }

    #[test]
    fn tables_drops_a_disabled_field_column() {
        let state = shared_state();
        let palettes = select_palettes(&state, &[]).unwrap();
        let fields = PaletteListFields {
            properties: true,
            materials: true,
            objects: false,
        };
        let output = render(&state, &palettes, fields, PaletteListLayout::Tables);
        assert_eq!(
            output,
            "# palettes\n\
             \n\
             | label | properties                        | materials |\n\
             | ----- | --------------------------------- | --------- |\n\
             | 0     | baseColorFactor, metallicFactor   | 2         |\n\
             | 1     | baseColorFactor, emissiveStrength | 1         |\n"
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
             │ │ ├ \"baseColorFactor\"\n\
             │ │ └ \"metallicFactor\"\n\
             │ └ objects\n\
             │   ├ \"a\"\n\
             │   └ \"b\"\n\
             └ 1\n\
             \u{20}\u{20}├ materialCount: 1\n\
             \u{20}\u{20}├ properties\n\
             \u{20}\u{20}│ ├ \"baseColorFactor\"\n\
             \u{20}\u{20}│ └ \"emissiveStrength\"\n\
             \u{20}\u{20}└ objects\n\
             \u{20}\u{20}\u{20}\u{20}└ \"b\"\n"
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
        let output = render(&state, &palettes, fields, PaletteListLayout::Hierarchy);
        assert_eq!(
            output,
            "palettes\n\
             ├ 0\n\
             │ ├ materialCount: 2\n\
             │ └ objects\n\
             │   ├ \"a\"\n\
             │   └ \"b\"\n\
             └ 1\n\
             \u{20}\u{20}├ materialCount: 1\n\
             \u{20}\u{20}└ objects\n\
             \u{20}\u{20}\u{20}\u{20}└ \"b\"\n"
        );
    }

    #[test]
    fn json_compact_nests_the_envelope_fields_under_each_palette() {
        assert_eq!(
            render_all(&shared_state(), PaletteListLayout::JsonCompact),
            "[{\"label\":\"palettes\",\"children\":[\
             {\"label\":\"0\",\"children\":[\
             {\"label\":\"materialCount\",\"values\":[2]},\
             {\"label\":\"properties\",\"children\":[\
             {\"label\":\"baseColorFactor\"},{\"label\":\"metallicFactor\"}]},\
             {\"label\":\"objects\",\"children\":[{\"label\":\"a\"},{\"label\":\"b\"}]}]},\
             {\"label\":\"1\",\"children\":[\
             {\"label\":\"materialCount\",\"values\":[1]},\
             {\"label\":\"properties\",\"children\":[\
             {\"label\":\"baseColorFactor\"},\
             {\"label\":\"emissiveStrength\"}]},\
             {\"label\":\"objects\",\"children\":[{\"label\":\"b\"}]}]}]}]\n"
        );
    }

    #[test]
    fn json_pretty_is_multiline_and_matches_compact() {
        let state = shared_state();
        let pretty = render_all(&state, PaletteListLayout::JsonPretty);
        let compact = render_all(&state, PaletteListLayout::JsonCompact);
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
        let output = render(&state, &palettes, all_fields(), PaletteListLayout::Tables);
        assert_eq!(
            output,
            "# palettes\n\
             \n\
             | label | properties                        | materials | used by |\n\
             | ----- | --------------------------------- | --------- | ------- |\n\
             | 1     | baseColorFactor, emissiveStrength | 1         | b       |\n"
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
        let colors = state.add_value_pool(VoxValuePool::srgba(vec![[1.0, 1.0, 1.0, 1.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .add_property("baseColorFactor".to_owned(), colors, U32Id::from_u32(0))
            .unwrap();
        palette.add_material(vec![value(0)]).unwrap();
        state.add_palette(palette).unwrap();

        assert_eq!(
            render_all(&state, PaletteListLayout::Tables),
            "# palettes\n\
             \n\
             | label | properties      | materials | used by |\n\
             | ----- | --------------- | --------- | ------- |\n\
             | 0     | baseColorFactor | 1         |         |\n"
        );

        assert_eq!(
            render_all(&state, PaletteListLayout::JsonCompact),
            "[{\"label\":\"palettes\",\"children\":[{\"label\":\"0\",\"children\":[\
             {\"label\":\"materialCount\",\"values\":[1]},\
             {\"label\":\"properties\",\"children\":[{\"label\":\"baseColorFactor\"}]},\
             {\"label\":\"objects\",\"values\":[\"[]\"]}]}]}]\n"
        );
    }

    #[test]
    fn an_unreferenced_palette_shows_an_empty_objects_branch() {
        let mut state = VoxMain::default();
        let colors = state.add_value_pool(VoxValuePool::srgba(vec![[1.0, 1.0, 1.0, 1.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .add_property("baseColorFactor".to_owned(), colors, U32Id::from_u32(0))
            .unwrap();
        palette.add_material(vec![value(0)]).unwrap();
        state.add_palette(palette).unwrap();

        assert_eq!(
            render_all(&state, PaletteListLayout::Hierarchy),
            "palettes\n\
             └ 0\n\
             \u{20}\u{20}├ materialCount: 1\n\
             \u{20}\u{20}├ properties\n\
             \u{20}\u{20}│ └ \"baseColorFactor\"\n\
             \u{20}\u{20}└ objects: []\n"
        );
    }
}
