use crate::{Format, ReportLayout, Result, implementation};
use branded_id::U32Id;
use serde_json::{Value, json};
use std::path::Path;
use voxcore::{BVoxPalette, VoxMain};

/// Loads the voxel file at `input` and prints a one-line-per-palette overview:
/// each palette's index, ordered attribute keys, cell count, and the objects
/// that reference it. `layout` chooses the Markdown table or a JSON form.
pub fn palette_list(input: &Path, from: Option<Format>, layout: ReportLayout) -> Result<()> {
    let state = implementation::load_state(input, from)?;
    let output = render(&state, layout)?;
    implementation::write_stdout(output.as_bytes())
}

/// Renders the listing for `state` in `layout`, the testable core of
/// [`palette_list`].
fn render(state: &VoxMain, layout: ReportLayout) -> Result<String> {
    match layout {
        ReportLayout::Markdown => Ok(render_markdown(state)),
        ReportLayout::PrettyJson => render_json(state, true),
        ReportLayout::CompactJson => render_json(state, false),
    }
}

/// The listing as one aligned table: index, attributes, cells, and the names of
/// the referencing objects.
fn render_markdown(state: &VoxMain) -> String {
    let rows: Vec<Vec<String>> = state
        .iter_palettes()
        .map(|(id, palette)| {
            let names: Vec<&str> = referencing_objects(state, id)
                .into_iter()
                .map(|(_, name)| name)
                .collect();
            implementation::row([
                &id.to_u32().to_string(),
                &implementation::md_cell(&implementation::attribute_names(palette).join(", ")),
                &palette.cell_count().to_string(),
                &implementation::md_cell(&names.join(", ")),
            ])
        })
        .collect();
    implementation::markdown_table(&["index", "attributes", "cells", "used by"], &rows)
}

/// The listing as a JSON array, pretty or compact, one record per palette in
/// index order: its index, attribute keys, cell count, and the indices of the
/// objects that reference it.
fn render_json(state: &VoxMain, pretty: bool) -> Result<String> {
    let palettes: Vec<Value> = state
        .iter_palettes()
        .map(|(id, palette)| {
            let used_by: Vec<u32> = referencing_objects(state, id)
                .into_iter()
                .map(|(index, _)| index)
                .collect();
            json!({
                "index": id.to_u32(),
                "attributes": implementation::attribute_names(palette),
                "cells": palette.cell_count(),
                "used_by": used_by,
            })
        })
        .collect();
    implementation::to_json_string(&Value::Array(palettes), pretty)
}

/// The objects that reference `palette`, in object order, as `(index, name)`.
/// An object appears once however many times it references the palette.
fn referencing_objects(state: &VoxMain, palette: U32Id<BVoxPalette>) -> Vec<(u32, &str)> {
    state
        .iter_objects()
        .filter(|(_, object)| object.iter_palette_refs().any(|(_, id)| id == palette))
        .map(|(id, object)| (id.to_u32(), object.name()))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{ReportLayout, implementation::palette_list::render};
    use serde_json::Value;
    use ty_math::TyVector3U32;
    use voxcore::{VoxMain, VoxObject, VoxPalette, VoxValue};

    /// Two palettes and two objects: `a` samples palette 0, `b` samples both.
    /// Palette 0 carries `rgba` and `metallic` with two cells, palette 1 carries
    /// `rgba` with one cell.
    fn shared_state() -> VoxMain {
        let mut state = VoxMain::default();

        let mut zero = VoxPalette::default();
        zero.add_attribute("rgba".to_owned());
        zero.add_attribute("metallic".to_owned());
        let zero_cell = zero
            .add_cell(vec![
                VoxValue::Text("#FF0000FF".to_owned()),
                VoxValue::Number(0.0),
            ])
            .unwrap();
        zero.add_cell(vec![
            VoxValue::Text("#00FF00FF".to_owned()),
            VoxValue::Number(1.0),
        ])
        .unwrap();
        let zero = state.add_palette(zero);

        let mut one = VoxPalette::default();
        one.add_attribute("rgba".to_owned());
        let one_cell = one
            .add_cell(vec![VoxValue::Text("#0000FFFF".to_owned())])
            .unwrap();
        let one = state.add_palette(one);

        let mut a = VoxObject::new("a".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        a.add_palette_ref(zero, zero_cell);
        state.add_object(a);

        let mut b = VoxObject::new("b".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        b.add_palette_ref(zero, zero_cell);
        b.add_palette_ref(one, one_cell);
        state.add_object(b);

        state
    }

    #[test]
    fn markdown_lists_one_row_per_palette() {
        let output = render(&shared_state(), ReportLayout::Markdown).unwrap();
        assert_eq!(
            output,
            "| index | attributes     | cells | used by |\n\
             | ----- | -------------- | ----- | ------- |\n\
             | 0     | rgba, metallic | 2     | a, b    |\n\
             | 1     | rgba           | 1     | b       |\n"
        );
    }

    #[test]
    fn compact_json_records_indices_attributes_cells_and_users() {
        let output = render(&shared_state(), ReportLayout::CompactJson).unwrap();
        assert_eq!(
            output,
            "[{\"index\":0,\"attributes\":[\"rgba\",\"metallic\"],\"cells\":2,\"used_by\":[0,1]},\
             {\"index\":1,\"attributes\":[\"rgba\"],\"cells\":1,\"used_by\":[1]}]\n"
        );
    }

    #[test]
    fn pretty_json_is_multiline_and_matches_compact() {
        let state = shared_state();
        let pretty = render(&state, ReportLayout::PrettyJson).unwrap();
        let compact = render(&state, ReportLayout::CompactJson).unwrap();
        assert!(pretty.starts_with("[\n"));
        let pretty_value: Value = serde_json::from_str(&pretty).unwrap();
        let compact_value: Value = serde_json::from_str(&compact).unwrap();
        assert_eq!(pretty_value, compact_value);
    }

    #[test]
    fn an_unreferenced_palette_lists_no_users() {
        let mut state = VoxMain::default();
        let mut palette = VoxPalette::default();
        palette.add_attribute("rgba".to_owned());
        palette
            .add_cell(vec![VoxValue::Text("#FFFFFFFF".to_owned())])
            .unwrap();
        state.add_palette(palette);

        let markdown = render(&state, ReportLayout::Markdown).unwrap();
        assert_eq!(
            markdown,
            "| index | attributes | cells | used by |\n\
             | ----- | ---------- | ----- | ------- |\n\
             | 0     | rgba       | 1     |         |\n"
        );

        let json = render(&state, ReportLayout::CompactJson).unwrap();
        assert_eq!(
            json,
            "[{\"index\":0,\"attributes\":[\"rgba\"],\"cells\":1,\"used_by\":[]}]\n"
        );
    }
}
