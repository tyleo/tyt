use crate::{BTreeGridNode, TreeGrid, TreeGridJsonCells};
use branded_id::U32Id;
use serde_json::{Error as JsonError, Map, Value};

/// The JSON renders: the record envelope, pretty or compact.
///
/// Implemented for grids whose cell policy carries JSON forms
/// ([`TreeGridJsonCells`]), so a JSON render is uncallable on a
/// text-only policy.
pub trait TreeGridRenderJson {
    /// Renders the `json-pretty` layout: one record per root node,
    /// pretty-printed.
    fn render_json_pretty(&self) -> String;

    /// Renders the `json-compact` layout: the envelope of
    /// [`render_json_pretty`](Self::render_json_pretty) on one line.
    fn render_json_compact(&self) -> String;
}

impl<C: TreeGridJsonCells> TreeGridRenderJson for TreeGrid<C> {
    fn render_json_pretty(&self) -> String {
        finish(serde_json::to_string_pretty(&self.records()))
    }

    fn render_json_compact(&self) -> String {
        finish(serde_json::to_string(&self.records()))
    }
}

impl<C: TreeGridJsonCells> TreeGrid<C> {
    /// The envelope: an array of root records.
    fn records(&self) -> Value {
        Value::Array(self.roots().iter().map(|&root| self.record(root)).collect())
    }

    /// `id`'s record. Insertion order is the emitted key order.
    fn record(&self, id: U32Id<BTreeGridNode>) -> Value {
        let node = self.node(id);
        let mut record = Map::new();
        record.insert(
            "label".to_owned(),
            Value::String(node.label.text().to_owned()),
        );
        if let Some(annotation) = &node.annotation {
            record.insert("annotation".to_owned(), Value::String(annotation.clone()));
        }
        if !node.values.is_empty() {
            let values = node
                .values
                .iter()
                .map(|value| self.cells().json(value))
                .collect();
            record.insert("values".to_owned(), Value::Array(values));
        }
        if !node.children().is_empty() {
            let children = node
                .children()
                .iter()
                .map(|&child| self.record(child))
                .collect();
            record.insert("children".to_owned(), Value::Array(children));
        }
        Value::Object(record)
    }
}

/// Appends the trailing newline. Serializing a built `Value` cannot
/// fail.
fn finish(serialized: Result<String, JsonError>) -> String {
    let mut output = serialized.expect("serializing a JSON value cannot fail");
    output.push('\n');
    output
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGrid, TreeGridJsonValue, TreeGridJsonValueCells, TreeGridLabel, TreeGridRenderJson,
        TreeGridValue,
    };
    use serde_json::{Value, json};

    /// The rendering spec's worked example, over the JSON-carrying
    /// policy.
    fn worked_example() -> TreeGrid<TreeGridJsonValueCells> {
        let mut grid = TreeGrid::with_cells(TreeGridJsonValueCells);
        let palette = grid.retain_root(TreeGridLabel::bare("0"));
        let base = grid.retain_child(palette, TreeGridLabel::quoted("baseColorFactor"));
        grid.push_value(base, TreeGridJsonValue::srgba8([255, 0, 0, 255]));
        grid.push_value(base, TreeGridJsonValue::srgba8([0, 255, 0, 128]));
        let metallic = grid.retain_child(palette, TreeGridLabel::quoted("metallicFactor"));
        grid.push_value(metallic, TreeGridJsonValue::unorm(1.0));
        grid.push_value(metallic, TreeGridJsonValue::unorm(0.2));
        let second = grid.retain_root(TreeGridLabel::bare("1"));
        let attribute = grid.retain_child(second, TreeGridLabel::quoted("baseColorFactor"));
        let component = grid.retain_child(attribute, TreeGridLabel::bare("a"));
        grid.push_value(component, TreeGridJsonValue::unorm8(255));
        grid
    }

    #[test]
    fn an_empty_grid_renders_the_empty_array() {
        assert_eq!(TreeGrid::new().render_json_pretty(), "[]\n");
        assert_eq!(TreeGrid::new().render_json_compact(), "[]\n");
    }

    #[test]
    fn compact_renders_the_worked_example_envelope() {
        assert_eq!(
            worked_example().render_json_compact(),
            "[{\"label\":\"0\",\"children\":[\
             {\"label\":\"baseColorFactor\",\"values\":[\"#FF0000FF\",\"#00FF0080\"]},\
             {\"label\":\"metallicFactor\",\"values\":[1,0.2]}]},\
             {\"label\":\"1\",\"children\":[\
             {\"label\":\"baseColorFactor\",\"children\":[\
             {\"label\":\"a\",\"values\":[255]}]}]}]\n"
        );
    }

    #[test]
    fn pretty_indents_the_same_envelope() {
        let mut grid = TreeGrid::with_cells(TreeGridJsonValueCells);
        let root = grid.retain_root(TreeGridLabel::bare("0"));
        grid.push_value(root, TreeGridJsonValue::unorm8(255));

        assert_eq!(
            grid.render_json_pretty(),
            "[\n  {\n    \"label\": \"0\",\n    \"values\": [\n      255\n    ]\n  }\n]\n"
        );
    }

    #[test]
    fn a_record_carries_label_annotation_values_and_children() {
        let mut grid = TreeGrid::new();
        let root = grid.retain_root(TreeGridLabel::quoted("energy-tank"));
        grid.node_mut(root).annotation = Some("(Group)".to_owned());
        grid.push_value(root, TreeGridValue::new("{node: 0}"));
        let child = grid.retain_child(root, TreeGridLabel::bare("transform"));
        grid.push_value(child, TreeGridValue::new("[12.5, 0.5, 10.0]"));

        assert_eq!(
            grid.render_json_compact(),
            "[{\"label\":\"energy-tank\",\"annotation\":\"(Group)\",\
             \"values\":[\"{node: 0}\"],\"children\":[\
             {\"label\":\"transform\",\"values\":[\"[12.5, 0.5, 10.0]\"]}]}]\n"
        );
    }

    #[test]
    fn labels_emit_raw_text_without_quote_wrapping() {
        let mut grid = TreeGrid::new();
        grid.retain_root(TreeGridLabel::quoted("has \"quotes\""));

        assert_eq!(
            grid.render_json_compact(),
            "[{\"label\":\"has \\\"quotes\\\"\"}]\n"
        );
    }

    #[test]
    fn duplicate_sibling_labels_survive() {
        let mut grid = TreeGrid::new();
        let root = grid.retain_root(TreeGridLabel::bare("root"));
        for _ in 0..2 {
            let door = grid.retain_child(root, TreeGridLabel::quoted("door"));
            grid.push_value(door, TreeGridValue::new("open"));
        }

        assert_eq!(
            grid.render_json_compact(),
            "[{\"label\":\"root\",\"children\":[\
             {\"label\":\"door\",\"values\":[\"open\"]},\
             {\"label\":\"door\",\"values\":[\"open\"]}]}]\n"
        );
    }

    #[test]
    fn values_carry_their_native_json_forms() {
        let mut grid = TreeGrid::with_cells(TreeGridJsonValueCells);
        let root = grid.retain_root(TreeGridLabel::bare("values"));
        grid.push_value(root, TreeGridJsonValue::int(-3));
        grid.push_value(root, TreeGridJsonValue::bool(true));
        grid.push_value(root, TreeGridJsonValue::json(json!({"node": 0})));
        grid.push_value(root, TreeGridJsonValue::new("{object: 0}"));

        assert_eq!(
            grid.render_json_compact(),
            "[{\"label\":\"values\",\"values\":[-3,true,{\"node\":0},\"{object: 0}\"]}]\n"
        );
    }

    #[test]
    fn pretty_and_compact_carry_equal_values() {
        let grid = worked_example();
        let pretty: Value = serde_json::from_str(&grid.render_json_pretty()).expect("valid JSON");
        let compact: Value = serde_json::from_str(&grid.render_json_compact()).expect("valid JSON");

        assert_eq!(pretty, compact);
    }
}
