use crate::{
    TreeGrid, TreeGridCells, TreeGridTableLabelMode, TreeGridTableShape, TreeGridTableShapeKind,
    render::TreeGridRenderFramedTables, render_md_tables::MdTableFrame,
};
use std::num::NonZeroU8;

/// The `md-tables` render.
pub trait TreeGridRenderMdTables {
    /// Renders the `md-tables` layout: aligned markdown tables in the
    /// given shape.
    fn render_md_tables(&self, shape: &TreeGridTableShape) -> String;
}

impl<C: TreeGridCells> TreeGridRenderMdTables for TreeGrid<C> {
    fn render_md_tables(&self, shape: &TreeGridTableShape) -> String {
        let (kind, label, level) = match *shape {
            TreeGridTableShape::Flat => (
                TreeGridTableShapeKind::Flat,
                TreeGridTableLabelMode::Concat,
                NonZeroU8::MIN,
            ),

            TreeGridTableShape::Nested(options) => {
                (TreeGridTableShapeKind::Nested, options.label, options.level)
            }

            TreeGridTableShape::Records(options) => (
                TreeGridTableShapeKind::Records,
                TreeGridTableLabelMode::Concat,
                options.level,
            ),
        };
        self.render_framed_tables(kind, label, &MdTableFrame { level })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGrid, TreeGridCellFormat, TreeGridLabel, TreeGridNestedTableOptions,
        TreeGridRecordsTableOptions, TreeGridRenderMdTables, TreeGridTableLabelMode,
        TreeGridTableShape, TreeGridValue,
    };
    use std::num::NonZeroU8;

    /// The rendering spec's worked example.
    fn worked_example() -> TreeGrid {
        let mut grid = TreeGrid::new();
        let palette = grid.retain_root(TreeGridLabel::bare("0"));
        let base = grid.retain_child(palette, TreeGridLabel::quoted("baseColorFactor"));
        grid.node_mut(base).format = Some(TreeGridCellFormat::Text);
        grid.push_value(base, TreeGridValue::srgba8([255, 0, 0, 255]));
        grid.push_value(base, TreeGridValue::srgba8([0, 255, 0, 128]));
        let metallic = grid.retain_child(palette, TreeGridLabel::quoted("metallicFactor"));
        grid.node_mut(metallic).format = Some(TreeGridCellFormat::Text);
        grid.push_value(metallic, TreeGridValue::unorm(1.0));
        grid.push_value(metallic, TreeGridValue::unorm(0.2));
        let second = grid.retain_root(TreeGridLabel::bare("1"));
        let attribute = grid.retain_child(second, TreeGridLabel::quoted("baseColorFactor"));
        let component = grid.retain_child(attribute, TreeGridLabel::bare("a"));
        grid.node_mut(component).format = Some(TreeGridCellFormat::Text);
        grid.push_value(component, TreeGridValue::unorm8(255));
        grid
    }

    /// The rendering spec's hierarchy-data worked example, cut to one
    /// full scene node and one tag-only sibling.
    fn hierarchy_example() -> TreeGrid {
        let mut grid = TreeGrid::new();
        let root = grid.retain_root(TreeGridLabel::bare("root"));
        let tank = grid.retain_child(root, TreeGridLabel::quoted("energy-tank-1"));
        grid.push_value(tank, TreeGridValue::new("{node: 0}"));
        let transform = grid.retain_child(tank, TreeGridLabel::bare("transform"));
        let position = grid.retain_child(transform, TreeGridLabel::bare("position"));
        grid.push_value(position, TreeGridValue::new("[12.50, 0.50, 10.00]"));
        let rotation = grid.retain_child(transform, TreeGridLabel::bare("rotation"));
        grid.push_value(rotation, TreeGridValue::new("[0.00, 0.00, 0.00]"));
        let scale = grid.retain_child(transform, TreeGridLabel::bare("scale"));
        grid.push_value(scale, TreeGridValue::new("[1.00, 1.00, 1.00]"));
        let object = grid.retain_child(tank, TreeGridLabel::quoted("energy-tank-1"));
        grid.push_value(object, TreeGridValue::new("{object: 0, instance: 0}"));
        let second = grid.retain_child(root, TreeGridLabel::quoted("energy-tank-2"));
        grid.push_value(second, TreeGridValue::new("{node: 1}"));
        grid
    }

    #[test]
    fn an_empty_grid_renders_the_empty_string() {
        assert_eq!(
            TreeGrid::new().render_md_tables(&TreeGridTableShape::default()),
            ""
        );
        assert_eq!(
            TreeGrid::new().render_md_tables(&TreeGridTableShape::Flat),
            ""
        );
        assert_eq!(
            TreeGrid::new().render_md_tables(&TreeGridTableShape::Records(
                TreeGridRecordsTableOptions::default()
            )),
            ""
        );
    }

    #[test]
    fn nested_concat_headings_carry_full_paths() {
        assert_eq!(
            worked_example().render_md_tables(&TreeGridTableShape::default()),
            "# 0\n\
             \n\
             | #   | \"baseColorFactor\" | \"metallicFactor\" |\n\
             | --- | ----------------- | ---------------- |\n\
             | 0   | #FF0000FF         | 1                |\n\
             | 1   | #00FF0080         | 0.2              |\n\
             \n\
             # 1\n\
             \n\
             ## 1.\"baseColorFactor\"\n\
             \n\
             | #   | a   |\n\
             | --- | --- |\n\
             | 0   | 255 |\n"
        );
    }

    #[test]
    fn nested_header_headings_carry_leaf_segments() {
        let shape = TreeGridTableShape::Nested(
            TreeGridNestedTableOptions::default().with_label(TreeGridTableLabelMode::Header),
        );
        assert_eq!(
            worked_example().render_md_tables(&shape),
            "# 0\n\
             \n\
             | #   | \"baseColorFactor\" | \"metallicFactor\" |\n\
             | --- | ----------------- | ---------------- |\n\
             | 0   | #FF0000FF         | 1                |\n\
             | 1   | #00FF0080         | 0.2              |\n\
             \n\
             # 1\n\
             \n\
             ## \"baseColorFactor\"\n\
             \n\
             | #   | a   |\n\
             | --- | --- |\n\
             | 0   | 255 |\n"
        );
    }

    #[test]
    fn a_branch_with_values_is_a_column_and_a_heading() {
        let shape = TreeGridTableShape::Nested(
            TreeGridNestedTableOptions::default().with_label(TreeGridTableLabelMode::Header),
        );
        assert_eq!(
            hierarchy_example().render_md_tables(&shape),
            "# root\n\
             \n\
             | #   | \"energy-tank-1\" | \"energy-tank-2\" |\n\
             | --- | --------------- | --------------- |\n\
             | 0   | {node: 0}       | {node: 1}       |\n\
             \n\
             ## \"energy-tank-1\"\n\
             \n\
             | #   | \"energy-tank-1\"          |\n\
             | --- | ------------------------ |\n\
             | 0   | {object: 0, instance: 0} |\n\
             \n\
             ### transform\n\
             \n\
             | #   | position             | rotation           | scale              |\n\
             | --- | -------------------- | ------------------ | ------------------ |\n\
             | 0   | [12.50, 0.50, 10.00] | [0.00, 0.00, 0.00] | [1.00, 1.00, 1.00] |\n"
        );
    }

    #[test]
    fn concat_paths_accumulate_down_the_branch_chain() {
        assert_eq!(
            hierarchy_example().render_md_tables(&TreeGridTableShape::default()),
            "# root\n\
             \n\
             | #   | \"energy-tank-1\" | \"energy-tank-2\" |\n\
             | --- | --------------- | --------------- |\n\
             | 0   | {node: 0}       | {node: 1}       |\n\
             \n\
             ## root.\"energy-tank-1\"\n\
             \n\
             | #   | \"energy-tank-1\"          |\n\
             | --- | ------------------------ |\n\
             | 0   | {object: 0, instance: 0} |\n\
             \n\
             ### root.\"energy-tank-1\".transform\n\
             \n\
             | #   | position             | rotation           | scale              |\n\
             | --- | -------------------- | ------------------ | ------------------ |\n\
             | 0   | [12.50, 0.50, 10.00] | [0.00, 0.00, 0.00] | [1.00, 1.00, 1.00] |\n"
        );
    }

    #[test]
    fn root_level_data_tables_first_with_no_heading() {
        let mut grid = TreeGrid::new();
        let count = grid.retain_root(TreeGridLabel::bare("materialCount"));
        grid.push_value(count, TreeGridValue::int(2));
        let palette = grid.retain_root(TreeGridLabel::bare("0"));
        let metallic = grid.retain_child(palette, TreeGridLabel::quoted("metallicFactor"));
        grid.node_mut(metallic).format = Some(TreeGridCellFormat::Text);
        grid.push_value(metallic, TreeGridValue::unorm(0.2));

        assert_eq!(
            grid.render_md_tables(&TreeGridTableShape::default()),
            "| #   | materialCount |\n\
             | --- | ------------- |\n\
             | 0   | 2             |\n\
             \n\
             # 0\n\
             \n\
             | #   | \"metallicFactor\" |\n\
             | --- | ---------------- |\n\
             | 0   | 0.2              |\n"
        );
    }

    #[test]
    fn annotations_suffix_column_headers_and_headings() {
        let mut grid = TreeGrid::new();
        let palette = grid.retain_root(TreeGridLabel::bare("0"));
        let strength = grid.retain_child(palette, TreeGridLabel::quoted("emissiveStrength"));
        grid.node_mut(strength).annotation = Some("(scalar)".to_string());
        grid.push_value(strength, TreeGridValue::float(2.0));
        let tint = grid.retain_child(palette, TreeGridLabel::quoted("tint"));
        grid.node_mut(tint).annotation = Some("(scalar)".to_string());
        let alpha = grid.retain_child(tint, TreeGridLabel::bare("a"));
        grid.node_mut(alpha).format = Some(TreeGridCellFormat::Text);
        grid.push_value(alpha, TreeGridValue::unorm8(128));
        let sub = grid.retain_child(tint, TreeGridLabel::bare("sub"));
        let leaf = grid.retain_child(sub, TreeGridLabel::bare("b"));
        grid.push_value(leaf, TreeGridValue::int(7));

        // The heading path stack stays bare, so `sub`'s concat
        // heading skips its parent's annotation.
        assert_eq!(
            grid.render_md_tables(&TreeGridTableShape::default()),
            "# 0\n\
             \n\
             | #   | \"emissiveStrength\" (scalar) |\n\
             | --- | --------------------------- |\n\
             | 0   | 2                           |\n\
             \n\
             ## 0.\"tint\" (scalar)\n\
             \n\
             | #   | a   |\n\
             | --- | --- |\n\
             | 0   | 128 |\n\
             \n\
             ### 0.\"tint\".sub\n\
             \n\
             | #   | b   |\n\
             | --- | --- |\n\
             | 0   | 7   |\n"
        );
    }

    #[test]
    fn flat_headers_carry_annotations() {
        let mut grid = TreeGrid::new();
        let palette = grid.retain_root(TreeGridLabel::bare("0"));
        let strength = grid.retain_child(palette, TreeGridLabel::quoted("emissiveStrength"));
        grid.node_mut(strength).annotation = Some("(scalar)".to_string());
        grid.push_value(strength, TreeGridValue::float(2.0));

        assert_eq!(
            grid.render_md_tables(&TreeGridTableShape::Flat),
            "| #   | 0.\"emissiveStrength\" (scalar) |\n\
             | --- | ----------------------------- |\n\
             | 0   | 2                             |\n"
        );
    }

    #[test]
    fn a_heading_past_level_six_renders_bold() {
        let shape = TreeGridTableShape::Nested(
            TreeGridNestedTableOptions::default().with_level(NonZeroU8::new(6).unwrap()),
        );
        assert_eq!(
            worked_example().render_md_tables(&shape),
            "###### 0\n\
             \n\
             | #   | \"baseColorFactor\" | \"metallicFactor\" |\n\
             | --- | ----------------- | ---------------- |\n\
             | 0   | #FF0000FF         | 1                |\n\
             | 1   | #00FF0080         | 0.2              |\n\
             \n\
             ###### 1\n\
             \n\
             **1.\"baseColorFactor\"**\n\
             \n\
             | #   | a   |\n\
             | --- | --- |\n\
             | 0   | 255 |\n"
        );
    }

    #[test]
    fn flat_tables_compare_across_the_forest() {
        assert_eq!(
            worked_example().render_md_tables(&TreeGridTableShape::Flat),
            "| #   | 0.\"baseColorFactor\" | 0.\"metallicFactor\" | 1.\"baseColorFactor\".a |\n\
             | --- | ------------------- | ------------------ | --------------------- |\n\
             | 0   | #FF0000FF           | 1                  | 255                   |\n\
             | 1   | #00FF0080           | 0.2                |                       |\n"
        );
    }

    #[test]
    fn labels_and_cells_escape_pipes() {
        let mut grid = TreeGrid::new();
        let palette = grid.retain_root(TreeGridLabel::bare("0"));
        let attribute = grid.retain_child(palette, TreeGridLabel::quoted("a|b"));
        grid.node_mut(attribute).format = Some(TreeGridCellFormat::Text);
        grid.push_value(attribute, TreeGridValue::new("x|y"));

        assert_eq!(
            grid.render_md_tables(&TreeGridTableShape::default()),
            "# 0\n\
             \n\
             | #   | \"a\\|b\" |\n\
             | --- | ------ |\n\
             | 0   | x\\|y   |\n"
        );
    }

    #[test]
    fn records_transpose_one_row_per_entity() {
        assert_eq!(
            hierarchy_example().render_md_tables(&TreeGridTableShape::Records(
                TreeGridRecordsTableOptions::default()
            )),
            "# root\n\
             \n\
             | label           | value     | transform.position   | transform.rotation | transform.scale    | \"energy-tank-1\"          |\n\
             | --------------- | --------- | -------------------- | ------------------ | ------------------ | ------------------------ |\n\
             | \"energy-tank-1\" | {node: 0} | [12.50, 0.50, 10.00] | [0.00, 0.00, 0.00] | [1.00, 1.00, 1.00] | {object: 0, instance: 0} |\n\
             | \"energy-tank-2\" | {node: 1} |                      |                    |                    |                          |\n"
        );
    }

    #[test]
    fn records_list_one_row_per_root_child() {
        let mut grid = TreeGrid::new();
        let root = grid.retain_root(TreeGridLabel::bare("palettes"));
        for (index, count, properties) in [
            ("0", 2, &["baseColorFactor", "metallicFactor"][..]),
            ("1", 1, &["baseColorFactor"][..]),
        ] {
            let palette = grid.retain_child(root, TreeGridLabel::bare(index));
            let materials = grid.retain_child(palette, TreeGridLabel::bare("materialCount"));
            grid.push_value(materials, TreeGridValue::int(count));
            let names = grid.retain_child(palette, TreeGridLabel::bare("properties"));
            for property in properties {
                grid.push_value(names, TreeGridValue::new(*property));
            }
        }

        assert_eq!(
            grid.render_md_tables(&TreeGridTableShape::Records(
                TreeGridRecordsTableOptions::default()
            )),
            "# palettes\n\
             \n\
             | label | materialCount | properties                     |\n\
             | ----- | ------------- | ------------------------------ |\n\
             | 0     | 2             | baseColorFactor metallicFactor |\n\
             | 1     | 1             | baseColorFactor                |\n"
        );
    }

    #[test]
    fn single_valued_rows_make_a_property_table() {
        let mut grid = TreeGrid::new();
        let root = grid.retain_root(TreeGridLabel::bare("Document"));
        let format = grid.retain_child(root, TreeGridLabel::bare("Format"));
        grid.push_value(format, TreeGridValue::new("voxj"));
        let ext = grid.retain_child(root, TreeGridLabel::bare("Has ext"));
        grid.push_value(ext, TreeGridValue::new("no"));

        assert_eq!(
            grid.render_md_tables(&TreeGridTableShape::Records(
                TreeGridRecordsTableOptions::default().with_level(NonZeroU8::new(2).unwrap())
            )),
            "## Document\n\
             \n\
             | label   | value |\n\
             | ------- | ----- |\n\
             | Format  | voxj  |\n\
             | Has ext | no    |\n"
        );
    }

    #[test]
    fn data_roots_row_a_leading_table_and_consume_their_subtrees() {
        let mut grid = TreeGrid::new();
        let stats = grid.retain_root(TreeGridLabel::bare("stats"));
        grid.push_value(stats, TreeGridValue::int(5));
        let deep = grid.retain_child(stats, TreeGridLabel::bare("deep"));
        grid.push_value(deep, TreeGridValue::int(9));
        let palettes = grid.retain_root(TreeGridLabel::bare("palettes"));
        let palette = grid.retain_child(palettes, TreeGridLabel::bare("0"));
        let materials = grid.retain_child(palette, TreeGridLabel::bare("materialCount"));
        grid.push_value(materials, TreeGridValue::int(1));

        assert_eq!(
            grid.render_md_tables(&TreeGridTableShape::Records(
                TreeGridRecordsTableOptions::default()
            )),
            "| label | value | deep |\n\
             | ----- | ----- | ---- |\n\
             | stats | 5     | 9    |\n\
             \n\
             # palettes\n\
             \n\
             | label | materialCount |\n\
             | ----- | ------------- |\n\
             | 0     | 1             |\n"
        );
    }

    #[test]
    fn a_multi_valued_cell_joins_with_the_separator_rule() {
        let mut grid = TreeGrid::new();
        let palette = grid.retain_root(TreeGridLabel::bare("0"));
        let base = grid.retain_child(palette, TreeGridLabel::quoted("baseColorFactor"));
        grid.node_mut(base).format = Some(TreeGridCellFormat::Visual);
        grid.push_value(base, TreeGridValue::srgba8([255, 0, 0, 255]));
        grid.push_value(base, TreeGridValue::srgba8([0, 255, 0, 128]));

        assert_eq!(
            grid.render_md_tables(&TreeGridTableShape::Records(
                TreeGridRecordsTableOptions::default()
            )),
            "# 0\n\
             \n\
             | label             | value |\n\
             | ----------------- | ----- |\n\
             | \"baseColorFactor\" | \x1b[48;2;255;0;0m  \x1b[0m\x1b[48;2;0;255;0m  \x1b[0m  |\n"
        );
    }

    #[test]
    fn records_labels_carry_annotations() {
        let mut grid = TreeGrid::new();
        let root = grid.retain_root(TreeGridLabel::bare("palettes"));
        let tint = grid.retain_child(root, TreeGridLabel::quoted("tint"));
        grid.node_mut(tint).annotation = Some("(scalar)".to_string());
        grid.push_value(tint, TreeGridValue::float(2.0));
        let alpha = grid.retain_child(tint, TreeGridLabel::bare("a"));
        grid.node_mut(alpha).annotation = Some("(alpha)".to_string());
        grid.node_mut(alpha).format = Some(TreeGridCellFormat::Text);
        grid.push_value(alpha, TreeGridValue::unorm8(128));

        assert_eq!(
            grid.render_md_tables(&TreeGridTableShape::Records(
                TreeGridRecordsTableOptions::default()
            )),
            "# palettes\n\
             \n\
             | label           | value | a (alpha) |\n\
             | --------------- | ----- | --------- |\n\
             | \"tint\" (scalar) | 2     | 128       |\n"
        );
    }

    #[test]
    fn same_path_descendants_merge_into_one_column() {
        let mut grid = TreeGrid::new();
        let root = grid.retain_root(TreeGridLabel::bare("r"));
        let entity = grid.retain_child(root, TreeGridLabel::bare("e"));
        let first = grid.retain_child(entity, TreeGridLabel::bare("x"));
        grid.push_value(first, TreeGridValue::int(1));
        let second = grid.retain_child(entity, TreeGridLabel::bare("x"));
        grid.push_value(second, TreeGridValue::int(2));

        assert_eq!(
            grid.render_md_tables(&TreeGridTableShape::Records(
                TreeGridRecordsTableOptions::default()
            )),
            "# r\n\
             \n\
             | label | x   |\n\
             | ----- | --- |\n\
             | e     | 1 2 |\n"
        );
    }

    #[test]
    fn a_visual_cell_pads_by_its_declared_width() {
        let mut grid = TreeGrid::new();
        let palette = grid.retain_root(TreeGridLabel::bare("0"));
        let color = grid.retain_child(palette, TreeGridLabel::quoted("c"));
        grid.push_value(color, TreeGridValue::srgba8([255, 0, 0, 255]));

        assert_eq!(
            grid.render_md_tables(&TreeGridTableShape::default()),
            "# 0\n\
             \n\
             | #   | \"c\"          |\n\
             | --- | ------------ |\n\
             | 0   | \x1b[48;2;255;0;0m  \x1b[0m #FF0000FF |\n"
        );
    }
}
