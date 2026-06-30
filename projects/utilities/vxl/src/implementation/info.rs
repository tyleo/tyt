use crate::{Format, InfoLayout, Result, implementation};
use serde_json::{Map, Value, json};
use std::{fs, io::Error as IOError, path::Path};
use voxcore::{VoxMain, VoxObject, VoxPalette};
use voxj_codec::from_voxj_or_voxjz_file_bytes;

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
    layout: InfoLayout,
) -> Result<String> {
    match layout {
        InfoLayout::Markdown => Ok(render_markdown(state, format, voxj_version, name)),
        InfoLayout::PrettyJson => render_json(state, format, voxj_version, true),
        InfoLayout::CompactJson => render_json(state, format, voxj_version, false),
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
    let mut document_rows = vec![row(["Format", format.name()])];
    if let Some(version) = voxj_version {
        document_rows.push(row(["Voxj version", &version.to_string()]));
    }
    document_rows.push(row(["Has ext", yes_no(state.ext().is_some())]));
    document_rows.push(row(["Has edit", yes_no(has_edit(state))]));

    let palette_rows: Vec<Vec<String>> = state
        .iter_palettes()
        .map(|(id, palette)| {
            row([
                &id.to_u32().to_string(),
                &md_cell(&attribute_names(palette).join(", ")),
                &palette.cell_count().to_string(),
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
            row([
                &id.to_u32().to_string(),
                &md_cell(object.name()),
                &dimensions(content_bounds(object)),
                &edit,
                &format!("{}, {}, {}", origin.x, origin.y, origin.z),
                &object.live_count().to_string(),
                &object.palette_ref_count().to_string(),
            ])
        })
        .collect();

    let mut output = format!("# {name}\n\n");
    output.push_str("## Document\n\n");
    output.push_str(&markdown_table(["Property", "Value"], &document_rows));
    output.push_str("\n## Palettes\n\n");
    output.push_str(&markdown_table(
        ["Id", "Attributes", "Cells"],
        &palette_rows,
    ));
    output.push_str("\n## Objects\n\n");
    output.push_str(&markdown_table(
        [
            "Id",
            "Name",
            "Bounds",
            "Edit bounds",
            "Origin",
            "Voxels",
            "Palettes",
        ],
        &object_rows,
    ));
    output
}

/// Collects borrowed cells into one owned table row.
fn row<const N: usize>(cells: [&str; N]) -> Vec<String> {
    cells.iter().map(|cell| cell.to_string()).collect()
}

/// One table: header, separator, then rows, every column padded to its widest
/// cell. Columns are at least three wide so the separator stays valid Markdown.
fn markdown_table<const N: usize>(headers: [&str; N], rows: &[Vec<String>]) -> String {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count().max(3)).collect();
    for row in rows {
        for (column, cell) in row.iter().enumerate() {
            widths[column] = widths[column].max(cell.chars().count());
        }
    }

    let separators: Vec<String> = widths.iter().map(|width| "-".repeat(*width)).collect();
    let mut output = table_row(headers.iter().map(|h| h.to_string()), &widths);
    output.push_str(&table_row(separators.into_iter(), &widths));
    for row in rows {
        output.push_str(&table_row(row.iter().cloned(), &widths));
    }
    output
}

/// One `| a | b |` line, each cell padded to its column width.
fn table_row(cells: impl Iterator<Item = String>, widths: &[usize]) -> String {
    let mut output = String::from("|");
    for (cell, &width) in cells.zip(widths) {
        output.push_str(&format!(" {cell:<width$} |"));
    }
    output.push('\n');
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
            json!({
                "id": id.to_u32(),
                "attributes": attribute_names(palette),
                "cells": palette.cell_count(),
            })
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
            entry.insert("palettes".to_string(), json!(object.palette_ref_count()));
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
    let document = Value::Object(document);

    let mut output = if pretty {
        serde_json::to_string_pretty(&document)
    } else {
        serde_json::to_string(&document)
    }
    .map_err(IOError::other)?;
    output.push('\n');
    Ok(output)
}

/// Whether any object has an [`edit_bounds`] build volume.
fn has_edit(state: &VoxMain) -> bool {
    state
        .iter_objects()
        .any(|(_, object)| edit_bounds(object).is_some())
}

/// A palette's attribute keys, in id order.
fn attribute_names(palette: &VoxPalette) -> Vec<&str> {
    palette.iter_attributes().map(|(_, name)| name).collect()
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

/// `text` safe for one table cell: pipes escaped, newlines flattened to spaces.
fn md_cell(text: &str) -> String {
    text.replace('|', "\\|").replace(['\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use crate::{Format, InfoLayout, implementation::info::render};
    use serde_json::Value;
    use ty_math::TyVector3U32;
    use voxcore::{VoxMain, VoxObject, VoxPalette, VoxValue};

    /// One `rgba` palette and one tight 1x1x1 object sampling its one cell.
    fn tight_state() -> VoxMain {
        let mut state = VoxMain::default();
        let mut palette = VoxPalette::default();
        palette.add_attribute("rgba".to_owned());
        let cell = palette
            .add_cell(vec![VoxValue::Text("#FF0000FF".to_owned())])
            .unwrap();
        let palette = state.add_palette(palette);

        let mut object = VoxObject::new("body".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.add_palette_ref(palette, cell);
        let voxel = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel, &[cell]).unwrap();
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
            InfoLayout::Markdown,
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
             | Id  | Attributes | Cells |\n\
             | --- | ---------- | ----- |\n\
             | 0   | rgba       | 1     |\n\
             \n## Objects\n\n\
             | Id  | Name | Bounds | Edit bounds | Origin  | Voxels | Palettes |\n\
             | --- | ---- | ------ | ----------- | ------- | ------ | -------- |\n\
             | 0   | body | 1x1x1  | -           | 0, 0, 0 | 1      | 1        |\n"
        );
    }

    #[test]
    fn compact_json_reports_the_document_in_order() {
        let output = render(
            &tight_state(),
            Format::Voxj,
            Some(2),
            "test.voxj",
            InfoLayout::CompactJson,
        )
        .unwrap();
        assert_eq!(
            output,
            "{\"format\":\"voxj\",\"voxj_version\":2,\"has_ext\":false,\"has_edit\":false,\
             \"palettes\":[{\"id\":0,\"attributes\":[\"rgba\"],\"cells\":1}],\
             \"objects\":[{\"id\":0,\"name\":\"body\",\"bounds\":[1,1,1],\"origin\":[0,0,0],\
             \"voxels\":1,\"palettes\":1}]}\n"
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
            InfoLayout::PrettyJson,
        )
        .unwrap();
        let compact = render(
            &state,
            Format::Voxj,
            Some(2),
            "test.voxj",
            InfoLayout::CompactJson,
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
            InfoLayout::Markdown,
        )
        .unwrap();
        assert!(markdown.contains("| Format   | mvox  |\n"));
        assert!(!markdown.contains("Voxj version"));

        let json = render(
            &tight_state(),
            Format::MVox,
            None,
            "model.mvox",
            InfoLayout::CompactJson,
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
            InfoLayout::Markdown,
        )
        .unwrap();
        assert!(markdown.contains("| Has edit     | yes   |\n"));
        // Content 1x1x1, edit build volume 3x1x1, origin spaced.
        assert!(
            markdown.contains(
                "| 0   | margin | 1x1x1  | 3x1x1       | 0, 0, 0 | 1      | 0        |\n"
            )
        );

        let json = render(
            &state,
            Format::Voxj,
            Some(1),
            "sample.voxj",
            InfoLayout::CompactJson,
        )
        .unwrap();
        assert!(json.contains("\"has_edit\":true"));
        assert!(json.contains("\"bounds\":[1,1,1],\"edit_bounds\":[3,1,1]"));
    }
}
