use crate::{Format, Result, commands::InfoLayout, implementation};
use branded_id::U32Id;
use std::{fs, num::NonZeroU8, path::Path};
use treegrid::{
    BTreeGridNode, TreeGrid, TreeGridJsonValue, TreeGridJsonValueCells, TreeGridLabel,
    TreeGridRecordsTableOptions, TreeGridRenderJson, TreeGridRenderTables, TreeGridTableShape,
    TreeGridValue,
};
use voxcore::{BVoxObject, VoxMain, VoxObject};
use voxj_codec::from_voxj_or_voxjz_file_bytes;

/// The record sections' heading level, beneath the `# {input}` title the
/// command prints itself.
const SECTION_LEVEL: NonZeroU8 = NonZeroU8::new(2).unwrap();

/// Loads the voxel file at `input` and reports what it contains in `layout`.
/// The document is read into voxcore first; only the format and, for Voxel Json,
/// the document version come from outside that model.
pub fn info(input: &Path, from: Option<Format>, layout: InfoLayout) -> Result<()> {
    let state = implementation::load_state(input, from)?;
    // load_state resolved the format, so this inference cannot fail.
    let format = from
        .or_else(|| Format::from_path(input))
        .expect("load_state resolved the input format");
    let voxj_version = match format {
        Format::Voxj => Some(read_voxj_version(input)?),
        _ => None,
    };
    let name = input
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| input.display().to_string());
    let output = render(&state, format, voxj_version, &name, layout);
    implementation::write_stdout(output.as_bytes())
}

/// The Voxel Json document version of `input`; voxcore does not carry it.
fn read_voxj_version(input: &Path) -> Result<u32> {
    Ok(from_voxj_or_voxjz_file_bytes(&fs::read(input)?)?.version)
}

/// Renders the report for `state` in `layout`, the testable core of [`info`].
fn render(
    state: &VoxMain,
    format: Format,
    voxj_version: Option<u32>,
    name: &str,
    layout: InfoLayout,
) -> String {
    match layout {
        InfoLayout::Tables => render_tables(state, format, voxj_version, name),
        InfoLayout::JsonPretty => build_json_grid(state, format, voxj_version).render_json_pretty(),
        InfoLayout::JsonCompact => {
            build_json_grid(state, format, voxj_version).render_json_compact()
        }
    }
}

/// The report as a file-name heading over three record-table sections:
/// document, palettes, objects.
fn render_tables(state: &VoxMain, format: Format, voxj_version: Option<u32>, name: &str) -> String {
    let tables = build_records_grid(state, format, voxj_version).render_tables(
        &TreeGridTableShape::Records(
            TreeGridRecordsTableOptions::default().with_level(SECTION_LEVEL),
        ),
    );
    format!("# {name}\n\n{tables}")
}

/// The flat forest the markdown tables render: `Document`, `Palettes`, and
/// `Objects` section roots whose rows bake every cell as one pre-formatted
/// value under its column's label.
fn build_records_grid(state: &VoxMain, format: Format, voxj_version: Option<u32>) -> TreeGrid {
    let mut grid = TreeGrid::new();

    let document = grid.add_root(TreeGridLabel::bare("Document"));
    add_cell(&mut grid, document, "Format", format.name().to_string());
    if let Some(version) = voxj_version {
        add_cell(&mut grid, document, "Voxj version", version.to_string());
    }
    add_cell(
        &mut grid,
        document,
        "Has ext",
        yes_no(state.ext().is_some()).to_string(),
    );
    add_cell(
        &mut grid,
        document,
        "Has edit",
        yes_no(has_edit(state)).to_string(),
    );

    let palettes = grid.add_root(TreeGridLabel::bare("Palettes"));
    for (id, palette) in state.iter_palettes() {
        let row = grid.add_child(palettes, TreeGridLabel::bare(id.to_u32().to_string()));
        add_cell(
            &mut grid,
            row,
            "Properties",
            implementation::property_labels(palette).join(", "),
        );
        add_cell(
            &mut grid,
            row,
            "Materials",
            palette.material_count().to_string(),
        );
    }

    let objects = grid.add_root(TreeGridLabel::bare("Objects"));
    for (id, object) in state.iter_objects() {
        let row = grid.add_child(objects, TreeGridLabel::bare(id.to_u32().to_string()));
        let origin = object.origin();
        let edit = match edit_bounds(object) {
            Some(edit) => dimensions(edit),
            None => "-".to_string(),
        };
        add_cell(&mut grid, row, "Name", object.name().to_string());
        add_cell(&mut grid, row, "Bounds", dimensions(content_bounds(object)));
        add_cell(&mut grid, row, "Edit bounds", edit);
        add_cell(
            &mut grid,
            row,
            "Origin",
            format!("{}, {}, {}", origin.x, origin.y, origin.z),
        );
        add_cell(&mut grid, row, "Voxels", object.live_count().to_string());
        add_cell(&mut grid, row, "Layers", object.layer_count().to_string());
        add_cell(
            &mut grid,
            row,
            "Sampled",
            sampled_layer_count(state, id).to_string(),
        );
    }
    grid
}

/// Adds a `label` node bearing `value` as one pre-formatted cell under
/// `parent`.
fn add_cell(grid: &mut TreeGrid, parent: U32Id<BTreeGridNode>, label: &str, value: String) {
    let node = grid.add_child(parent, TreeGridLabel::bare(label));
    grid.push_value(node, TreeGridValue::new(value));
}

/// The report tree the JSON layouts render as the shared envelope:
/// `document`, `palettes`, and `objects` roots carrying each field as a
/// typed value under a snake_case label.
fn build_json_grid(
    state: &VoxMain,
    format: Format,
    voxj_version: Option<u32>,
) -> TreeGrid<TreeGridJsonValueCells> {
    let mut grid = TreeGrid::with_cells(TreeGridJsonValueCells);

    let document = grid.add_root(TreeGridLabel::bare("document"));
    add_field(
        &mut grid,
        document,
        "format",
        TreeGridJsonValue::new(format.name()),
    );
    if let Some(version) = voxj_version {
        add_field(
            &mut grid,
            document,
            "voxj_version",
            TreeGridJsonValue::int(i64::from(version)),
        );
    }
    add_field(
        &mut grid,
        document,
        "has_ext",
        TreeGridJsonValue::bool(state.ext().is_some()),
    );
    add_field(
        &mut grid,
        document,
        "has_edit",
        TreeGridJsonValue::bool(has_edit(state)),
    );

    let palettes = grid.add_root(TreeGridLabel::bare("palettes"));
    for (id, palette) in state.iter_palettes() {
        let branch = grid.add_child(palettes, TreeGridLabel::bare(id.to_u32().to_string()));
        let properties = grid.add_child(branch, TreeGridLabel::bare("properties"));
        for (label, annotation) in implementation::property_entries(palette) {
            let node = grid.add_child(properties, label);
            grid.node_mut(node).annotation = annotation;
        }
        add_field(
            &mut grid,
            branch,
            "materials",
            TreeGridJsonValue::int(palette.material_count() as i64),
        );
    }

    let objects = grid.add_root(TreeGridLabel::bare("objects"));
    for (id, object) in state.iter_objects() {
        let branch = grid.add_child(objects, TreeGridLabel::bare(id.to_u32().to_string()));
        let bounds = content_bounds(object);
        let origin = object.origin();
        add_field(
            &mut grid,
            branch,
            "name",
            TreeGridJsonValue::new(object.name()),
        );
        add_triple(
            &mut grid,
            branch,
            "bounds",
            [
                i64::from(bounds.0),
                i64::from(bounds.1),
                i64::from(bounds.2),
            ],
        );
        if let Some(edit) = edit_bounds(object) {
            add_triple(
                &mut grid,
                branch,
                "edit_bounds",
                [i64::from(edit.0), i64::from(edit.1), i64::from(edit.2)],
            );
        }
        add_triple(
            &mut grid,
            branch,
            "origin",
            [
                i64::from(origin.x),
                i64::from(origin.y),
                i64::from(origin.z),
            ],
        );
        add_field(
            &mut grid,
            branch,
            "voxels",
            TreeGridJsonValue::int(object.live_count() as i64),
        );
        add_field(
            &mut grid,
            branch,
            "layers",
            TreeGridJsonValue::int(object.layer_count() as i64),
        );
        add_field(
            &mut grid,
            branch,
            "sampled_layers",
            TreeGridJsonValue::int(sampled_layer_count(state, id) as i64),
        );
    }
    grid
}

/// Adds a `label` node bearing one typed `value` under `parent`.
fn add_field(
    grid: &mut TreeGrid<TreeGridJsonValueCells>,
    parent: U32Id<BTreeGridNode>,
    label: &str,
    value: TreeGridJsonValue,
) {
    let node = grid.add_child(parent, TreeGridLabel::bare(label));
    grid.push_value(node, value);
}

/// Adds a `label` node whose series is an integer triple under `parent`.
fn add_triple(
    grid: &mut TreeGrid<TreeGridJsonValueCells>,
    parent: U32Id<BTreeGridNode>,
    label: &str,
    triple: [i64; 3],
) {
    let node = grid.add_child(parent, TreeGridLabel::bare(label));
    for component in triple {
        grid.push_value(node, TreeGridJsonValue::int(component));
    }
}

/// How many of an object's layers are sampled: those whose palette has
/// materials.
fn sampled_layer_count(state: &VoxMain, id: U32Id<BVoxObject>) -> usize {
    state
        .iter_sampled_layers(id)
        .map(|layers| layers.count())
        .unwrap_or(0)
}

/// Whether any object has an [`edit_bounds`] build volume.
fn has_edit(state: &VoxMain) -> bool {
    state
        .iter_objects()
        .any(|(_, object)| edit_bounds(object).is_some())
}

/// The size of an object's tight live-voxel extent. `(0, 0, 0)` when empty.
fn content_bounds(object: &VoxObject) -> (u32, u32, u32) {
    match object.live_extent() {
        Some((_, size)) => (size.x, size.y, size.z),
        None => (0, 0, 0),
    }
}

/// The object's build volume, but only when it has margin: wider than the tight
/// content or offset from the min corner. `None` when it matches the content.
fn edit_bounds(object: &VoxObject) -> Option<(u32, u32, u32)> {
    let build = object.bounds();
    let has_margin = match object.live_extent() {
        // Off the min corner or short of the build volume on any axis.
        Some((min, size)) => {
            min.x != 0
                || min.y != 0
                || min.z != 0
                || size.x != build.x
                || size.y != build.y
                || size.z != build.z
        }
        // No live voxels: a non-empty build volume is all margin.
        None => build.x != 0 || build.y != 0 || build.z != 0,
    };
    has_margin.then_some((build.x, build.y, build.z))
}

/// `XxYxZ` for a size triple.
fn dimensions(size: (u32, u32, u32)) -> String {
    format!("{}x{}x{}", size.0, size.1, size.2)
}

/// `yes` or `no` for a flag.
fn yes_no(flag: bool) -> &'static str {
    if flag { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use crate::{Format, commands::InfoLayout, implementation::info::render};
    use branded_id::{IdVec, U32Id};
    use serde_json::Value;
    use ty_math::TyVector3U32;
    use voxcore::{VoxBound, VoxMain, VoxObject, VoxPalette, VoxValuePool};

    /// One `baseColorFactor` palette and one tight 1x1x1 object sampling its one
    /// material.
    fn tight_state() -> VoxMain {
        let mut state = VoxMain::default();
        let pool = state.add_value_pool(VoxValuePool::Srgba {
            values: IdVec::from_vec(vec![[1.0, 0.0, 0.0, 1.0]]),
        });
        let mut palette = VoxPalette::default();
        palette.add_array_property("baseColorFactor".to_owned(), pool);
        let material = palette.add_material(vec![U32Id::from_u32(0)]).unwrap();
        let palette = state.add_palette(palette);

        let mut object = VoxObject::new("body".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.add_layer(palette, material);
        let voxel = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel, &[material]).unwrap();
        state.add_object(object);
        state
    }

    #[test]
    fn tables_lists_document_palette_and_object_sections() {
        let output = render(
            &tight_state(),
            Format::Voxj,
            Some(2),
            "test.voxj",
            InfoLayout::Tables,
        );
        assert_eq!(
            output,
            "# test.voxj\n\n\
             ## Document\n\n\
             | label        | value |\n\
             | ------------ | ----- |\n\
             | Format       | voxj  |\n\
             | Voxj version | 2     |\n\
             | Has ext      | no    |\n\
             | Has edit     | no    |\n\
             \n## Palettes\n\n\
             | label | Properties      | Materials |\n\
             | ----- | --------------- | --------- |\n\
             | 0     | baseColorFactor | 1         |\n\
             \n## Objects\n\n\
             | label | Name | Bounds | Edit bounds | Origin  | Voxels | Layers | Sampled |\n\
             | ----- | ---- | ------ | ----------- | ------- | ------ | ------ | ------- |\n\
             | 0     | body | 1x1x1  | -           | 0, 0, 0 | 1      | 1      | 1       |\n"
        );
    }

    #[test]
    fn json_compact_nests_the_envelope_fields_under_each_section() {
        let output = render(
            &tight_state(),
            Format::Voxj,
            Some(2),
            "test.voxj",
            InfoLayout::JsonCompact,
        );
        assert_eq!(
            output,
            "[{\"label\":\"document\",\"children\":[\
             {\"label\":\"format\",\"values\":[\"voxj\"]},\
             {\"label\":\"voxj_version\",\"values\":[2]},\
             {\"label\":\"has_ext\",\"values\":[false]},\
             {\"label\":\"has_edit\",\"values\":[false]}]},\
             {\"label\":\"palettes\",\"children\":[{\"label\":\"0\",\"children\":[\
             {\"label\":\"properties\",\"children\":[{\"label\":\"baseColorFactor\"}]},\
             {\"label\":\"materials\",\"values\":[1]}]}]},\
             {\"label\":\"objects\",\"children\":[{\"label\":\"0\",\"children\":[\
             {\"label\":\"name\",\"values\":[\"body\"]},\
             {\"label\":\"bounds\",\"values\":[1,1,1]},\
             {\"label\":\"origin\",\"values\":[0,0,0]},\
             {\"label\":\"voxels\",\"values\":[1]},\
             {\"label\":\"layers\",\"values\":[1]},\
             {\"label\":\"sampled_layers\",\"values\":[1]}]}]}]\n"
        );
    }

    #[test]
    fn json_pretty_is_multiline_and_matches_compact() {
        let state = tight_state();
        let pretty = render(
            &state,
            Format::Voxj,
            Some(2),
            "test.voxj",
            InfoLayout::JsonPretty,
        );
        let compact = render(
            &state,
            Format::Voxj,
            Some(2),
            "test.voxj",
            InfoLayout::JsonCompact,
        );
        assert!(pretty.starts_with("[\n"));
        assert!(pretty.contains("\"label\": \"document\""));
        let pretty_value: Value = serde_json::from_str(&pretty).unwrap();
        let compact_value: Value = serde_json::from_str(&compact).unwrap();
        assert_eq!(pretty_value, compact_value);
    }

    #[test]
    fn omits_voxj_version_for_other_formats() {
        let tables = render(
            &tight_state(),
            Format::MVox,
            None,
            "model.mvox",
            InfoLayout::Tables,
        );
        assert!(tables.contains("| Format   | mvox  |\n"));
        assert!(!tables.contains("Voxj version"));

        let json = render(
            &tight_state(),
            Format::MVox,
            None,
            "model.mvox",
            InfoLayout::JsonCompact,
        );
        assert!(!json.contains("voxj_version"));
    }

    #[test]
    fn reports_edit_bounds_when_an_object_carries_margin() {
        // 3x1x1 build volume, one voxel off the min corner: 1x1x1 content with
        // margin.
        let mut state = VoxMain::default();
        let mut object = VoxObject::new("margin".to_owned(), TyVector3U32::new(3, 1, 1)).unwrap();
        let voxel = object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap();
        object.retain_voxel(voxel, &[]).unwrap();
        state.add_object(object);

        let tables = render(
            &state,
            Format::Voxj,
            Some(1),
            "sample.voxj",
            InfoLayout::Tables,
        );
        assert!(tables.contains("| Has edit     | yes   |\n"));
        // Content 1x1x1, edit build volume 3x1x1, origin spaced.
        assert!(tables.contains(
            "| 0     | margin | 1x1x1  | 3x1x1       | 0, 0, 0 | 1      | 0      | 0       |\n"
        ));

        let json = render(
            &state,
            Format::Voxj,
            Some(1),
            "sample.voxj",
            InfoLayout::JsonCompact,
        );
        assert!(json.contains("{\"label\":\"has_edit\",\"values\":[true]}"));
        assert!(json.contains(
            "{\"label\":\"bounds\",\"values\":[1,1,1]},\
             {\"label\":\"edit_bounds\",\"values\":[3,1,1]}"
        ));
    }

    #[test]
    fn reports_scalar_properties_and_unsampled_layers() {
        // Palette 0 pins `emissiveStrength` with no materials, so the layer
        // referencing it is unsampled.
        let mut state = VoxMain::default();
        let strengths = state.add_value_pool(VoxValuePool::Float {
            min: VoxBound::Number(0.0),
            max: VoxBound::None,
            values: IdVec::from_vec(vec![2.0]),
        });
        let mut pinned = VoxPalette::default();
        pinned.add_scalar_property("emissiveStrength".to_owned(), strengths, U32Id::from_u32(0));
        let pinned = state.add_palette(pinned);

        let colors = state.add_value_pool(VoxValuePool::Srgba {
            values: IdVec::from_vec(vec![[1.0, 0.0, 0.0, 1.0]]),
        });
        let mut sampled = VoxPalette::default();
        sampled.add_array_property("baseColorFactor".to_owned(), colors);
        let material = sampled.add_material(vec![U32Id::from_u32(0)]).unwrap();
        let sampled = state.add_palette(sampled);

        let mut object = VoxObject::new("body".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.add_layer(sampled, material);
        // An unsampled layer's cells are ignored filler, so any id fills.
        object.add_layer(pinned, U32Id::from_u32(0));
        state.add_object(object);

        let tables = render(
            &state,
            Format::Voxj,
            Some(1),
            "pinned.voxj",
            InfoLayout::Tables,
        );
        assert!(tables.contains("| 0     | emissiveStrength (scalar) | 0         |\n"));
        assert!(tables.contains("| 1     | baseColorFactor           | 1         |\n"));
        // Two layers, one sampled.
        assert!(tables.contains("| 2      | 1       |\n"));

        let json = render(
            &state,
            Format::Voxj,
            Some(1),
            "pinned.voxj",
            InfoLayout::JsonCompact,
        );
        assert!(json.contains(
            "{\"label\":\"0\",\"children\":[{\"label\":\"properties\",\"children\":[\
             {\"label\":\"emissiveStrength\",\"annotation\":\"(scalar)\"}]},\
             {\"label\":\"materials\",\"values\":[0]}]}"
        ));
        assert!(json.contains(
            "{\"label\":\"layers\",\"values\":[2]},\
             {\"label\":\"sampled_layers\",\"values\":[1]}"
        ));
    }
}
