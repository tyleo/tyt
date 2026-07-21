use crate::{Format, ReportLayout, Result, implementation};
use branded_id::U32Id;
use serde_json::{Map, Value, json};
use std::{fs, path::Path};
use voxcore::{BVoxObject, VoxMain, VoxObject};
use voxj_codec::from_voxj_or_voxjz_file_bytes;

/// Loads the voxel file at `input` and reports what it contains in `layout`.
/// The document is read into voxcore first; only the format and, for Voxel Json,
/// the document version come from outside that model.
pub fn info(input: &Path, from: Option<Format>, layout: ReportLayout) -> Result<()> {
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
    let output = render(&state, format, voxj_version, &name, layout)?;
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
    layout: ReportLayout,
) -> Result<String> {
    match layout {
        ReportLayout::Markdown => Ok(render_markdown(state, format, voxj_version, name)),
        ReportLayout::PrettyJson => render_json(state, format, voxj_version, true),
        ReportLayout::CompactJson => render_json(state, format, voxj_version, false),
    }
}

/// The report as a file-name heading and three tables: document, palettes,
/// objects.
fn render_markdown(
    state: &VoxMain,
    format: Format,
    voxj_version: Option<u32>,
    name: &str,
) -> String {
    let mut document_rows = vec![implementation::row(["Format", format.name()])];
    if let Some(version) = voxj_version {
        document_rows.push(implementation::row(["Voxj version", &version.to_string()]));
    }
    document_rows.push(implementation::row([
        "Has ext",
        yes_no(state.ext().is_some()),
    ]));
    document_rows.push(implementation::row(["Has edit", yes_no(has_edit(state))]));

    let palette_rows: Vec<Vec<String>> = state
        .iter_palettes()
        .map(|(id, palette)| {
            implementation::row([
                &id.to_u32().to_string(),
                &implementation::md_cell(&implementation::property_labels(palette).join(", ")),
                &palette.material_count().to_string(),
            ])
        })
        .collect();

    let object_rows: Vec<Vec<String>> = state
        .iter_objects()
        .map(|(id, object)| {
            let origin = object.origin();
            let edit = match edit_bounds(object) {
                Some(edit) => dimensions(edit),
                None => "-".to_string(),
            };
            implementation::row([
                &id.to_u32().to_string(),
                &implementation::md_cell(object.name()),
                &dimensions(content_bounds(object)),
                &edit,
                &format!("{}, {}, {}", origin.x, origin.y, origin.z),
                &object.live_count().to_string(),
                &object.layer_count().to_string(),
                &sampled_layer_count(state, id).to_string(),
            ])
        })
        .collect();

    let mut output = format!("# {name}\n\n");
    output.push_str("## Document\n\n");
    output.push_str(&implementation::markdown_table(
        &["Property", "Value"],
        &document_rows,
    ));
    output.push_str("\n## Palettes\n\n");
    output.push_str(&implementation::markdown_table(
        &["Id", "Properties", "Materials"],
        &palette_rows,
    ));
    output.push_str("\n## Objects\n\n");
    output.push_str(&implementation::markdown_table(
        &[
            "Id",
            "Name",
            "Bounds",
            "Edit bounds",
            "Origin",
            "Voxels",
            "Layers",
            "Sampled",
        ],
        &object_rows,
    ));
    output
}

/// The report as one JSON object, pretty or compact. Keys keep insertion order.
fn render_json(
    state: &VoxMain,
    format: Format,
    voxj_version: Option<u32>,
    pretty: bool,
) -> Result<String> {
    let palettes: Vec<Value> = state
        .iter_palettes()
        .map(|(id, palette)| {
            // Field by field so `scalar_properties` appears only when present.
            let mut entry = Map::new();
            entry.insert("id".to_string(), json!(id.to_u32()));
            entry.insert(
                "properties".to_string(),
                json!(implementation::property_names(palette)),
            );
            let scalars = implementation::scalar_property_names(palette);
            if !scalars.is_empty() {
                entry.insert("scalar_properties".to_string(), json!(scalars));
            }
            entry.insert("materials".to_string(), json!(palette.material_count()));
            Value::Object(entry)
        })
        .collect();

    let objects: Vec<Value> = state
        .iter_objects()
        .map(|(id, object)| {
            let bounds = content_bounds(object);
            let origin = object.origin();
            // Field by field so `edit_bounds` appears only when present.
            let mut entry = Map::new();
            entry.insert("id".to_string(), json!(id.to_u32()));
            entry.insert("name".to_string(), json!(object.name()));
            entry.insert("bounds".to_string(), json!([bounds.0, bounds.1, bounds.2]));
            if let Some(edit) = edit_bounds(object) {
                entry.insert("edit_bounds".to_string(), json!([edit.0, edit.1, edit.2]));
            }
            entry.insert("origin".to_string(), json!([origin.x, origin.y, origin.z]));
            entry.insert("voxels".to_string(), json!(object.live_count()));
            entry.insert("layers".to_string(), json!(object.layer_count()));
            entry.insert(
                "sampled_layers".to_string(),
                json!(sampled_layer_count(state, id)),
            );
            Value::Object(entry)
        })
        .collect();

    let mut document = Map::new();
    document.insert("format".to_string(), json!(format.name()));
    if let Some(version) = voxj_version {
        document.insert("voxj_version".to_string(), json!(version));
    }
    document.insert("has_ext".to_string(), json!(state.ext().is_some()));
    document.insert("has_edit".to_string(), json!(has_edit(state)));
    document.insert("palettes".to_string(), Value::Array(palettes));
    document.insert("objects".to_string(), Value::Array(objects));
    implementation::to_json_string(&Value::Object(document), pretty)
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
    use crate::{Format, ReportLayout, implementation::info::render};
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
    fn markdown_lists_document_palette_and_object_tables() {
        let output = render(
            &tight_state(),
            Format::Voxj,
            Some(2),
            "test.voxj",
            ReportLayout::Markdown,
        )
        .unwrap();
        assert_eq!(
            output,
            "# test.voxj\n\n\
             ## Document\n\n\
             | Property     | Value |\n\
             | ------------ | ----- |\n\
             | Format       | voxj  |\n\
             | Voxj version | 2     |\n\
             | Has ext      | no    |\n\
             | Has edit     | no    |\n\
             \n## Palettes\n\n\
             | Id  | Properties      | Materials |\n\
             | --- | --------------- | --------- |\n\
             | 0   | baseColorFactor | 1         |\n\
             \n## Objects\n\n\
             | Id  | Name | Bounds | Edit bounds | Origin  | Voxels | Layers | Sampled |\n\
             | --- | ---- | ------ | ----------- | ------- | ------ | ------ | ------- |\n\
             | 0   | body | 1x1x1  | -           | 0, 0, 0 | 1      | 1      | 1       |\n"
        );
    }

    #[test]
    fn compact_json_reports_the_document_in_order() {
        let output = render(
            &tight_state(),
            Format::Voxj,
            Some(2),
            "test.voxj",
            ReportLayout::CompactJson,
        )
        .unwrap();
        assert_eq!(
            output,
            "{\"format\":\"voxj\",\"voxj_version\":2,\"has_ext\":false,\"has_edit\":false,\
             \"palettes\":[{\"id\":0,\"properties\":[\"baseColorFactor\"],\"materials\":1}],\
             \"objects\":[{\"id\":0,\"name\":\"body\",\"bounds\":[1,1,1],\"origin\":[0,0,0],\
             \"voxels\":1,\"layers\":1,\"sampled_layers\":1}]}\n"
        );
    }

    #[test]
    fn pretty_json_is_multiline_and_matches_compact() {
        let state = tight_state();
        let pretty = render(
            &state,
            Format::Voxj,
            Some(2),
            "test.voxj",
            ReportLayout::PrettyJson,
        )
        .unwrap();
        let compact = render(
            &state,
            Format::Voxj,
            Some(2),
            "test.voxj",
            ReportLayout::CompactJson,
        )
        .unwrap();
        assert!(pretty.starts_with("{\n"));
        assert!(pretty.contains("\"format\": \"voxj\""));
        let pretty_value: Value = serde_json::from_str(&pretty).unwrap();
        let compact_value: Value = serde_json::from_str(&compact).unwrap();
        assert_eq!(pretty_value, compact_value);
    }

    #[test]
    fn omits_voxj_version_for_other_formats() {
        let markdown = render(
            &tight_state(),
            Format::MVox,
            None,
            "model.mvox",
            ReportLayout::Markdown,
        )
        .unwrap();
        assert!(markdown.contains("| Format   | mvox  |\n"));
        assert!(!markdown.contains("Voxj version"));

        let json = render(
            &tight_state(),
            Format::MVox,
            None,
            "model.mvox",
            ReportLayout::CompactJson,
        )
        .unwrap();
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

        let markdown = render(
            &state,
            Format::Voxj,
            Some(1),
            "sample.voxj",
            ReportLayout::Markdown,
        )
        .unwrap();
        assert!(markdown.contains("| Has edit     | yes   |\n"));
        // Content 1x1x1, edit build volume 3x1x1, origin spaced.
        assert!(markdown.contains(
            "| 0   | margin | 1x1x1  | 3x1x1       | 0, 0, 0 | 1      | 0      | 0       |\n"
        ));

        let json = render(
            &state,
            Format::Voxj,
            Some(1),
            "sample.voxj",
            ReportLayout::CompactJson,
        )
        .unwrap();
        assert!(json.contains("\"has_edit\":true"));
        assert!(json.contains("\"bounds\":[1,1,1],\"edit_bounds\":[3,1,1]"));
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

        let markdown = render(
            &state,
            Format::Voxj,
            Some(1),
            "pinned.voxj",
            ReportLayout::Markdown,
        )
        .unwrap();
        assert!(markdown.contains("| 0   | emissiveStrength (scalar) | 0         |\n"));
        assert!(markdown.contains("| 1   | baseColorFactor           | 1         |\n"));
        // Two layers, one sampled.
        assert!(markdown.contains("| 2      | 1       |\n"));

        let json = render(
            &state,
            Format::Voxj,
            Some(1),
            "pinned.voxj",
            ReportLayout::CompactJson,
        )
        .unwrap();
        assert!(json.contains(
            "{\"id\":0,\"properties\":[\"emissiveStrength\"],\
             \"scalar_properties\":[\"emissiveStrength\"],\"materials\":0}"
        ));
        assert!(json.contains("\"layers\":2,\"sampled_layers\":1"));
    }
}
