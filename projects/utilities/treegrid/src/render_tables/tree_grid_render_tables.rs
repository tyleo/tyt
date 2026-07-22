use crate::{
    BTreeGridNode, TreeGrid, TreeGridCells, TreeGridNestedTableOptions, TreeGridTableLabelMode,
    TreeGridTableShape,
    render::{self, Cell},
    render_tables,
};
use branded_id::U32Id;

/// The `tables` render.
pub trait TreeGridRenderTables {
    /// Renders the `tables` layout: aligned markdown tables led by a
    /// `#` column of 0-based value indices, one column per data node.
    fn render_tables(&self, shape: &TreeGridTableShape) -> String;
}

impl<C: TreeGridCells> TreeGridRenderTables for TreeGrid<C> {
    fn render_tables(&self, shape: &TreeGridTableShape) -> String {
        let blocks = match shape {
            TreeGridTableShape::Nested(options) => self.nested_blocks(options),
            TreeGridTableShape::Flat => self.flat_blocks(),
        };
        if blocks.is_empty() {
            String::new()
        } else {
            format!("{}\n", blocks.join("\n\n"))
        }
    }
}

impl<C: TreeGridCells> TreeGrid<C> {
    /// The nested blocks: headings and group tables in the grouping
    /// walk's order.
    fn nested_blocks(&self, options: &TreeGridNestedTableOptions) -> Vec<String> {
        let mut blocks: Vec<String> = Vec::new();
        // Groups arrive depth-first with parents before children, so a
        // depth-indexed stack of cumulative paths recovers each
        // branch's dot-joined path without parent links in the arena.
        // The stacked segments stay bare; the annotation joins only
        // the branch's own heading.
        let mut paths: Vec<String> = Vec::new();
        for group in self.groups() {
            if let Some(branch) = group.branch {
                let node = self.node(branch);
                let leaf = node.label.render();
                paths.truncate(group.depth);
                let path = match paths.last() {
                    Some(parent) => format!("{parent}.{leaf}"),
                    None => leaf.clone(),
                };
                let text = match options.label {
                    TreeGridTableLabelMode::Concat => path.clone(),
                    TreeGridTableLabelMode::Header => leaf,
                };
                let text = match &node.annotation {
                    Some(annotation) => format!("{text} {annotation}"),
                    None => text,
                };
                blocks.push(render::heading(options.level, group.depth, &text));
                paths.push(path);
            }
            if !group.members.is_empty() {
                let labels: Vec<String> = group
                    .members
                    .iter()
                    .map(|&id| self.node(id).annotated_label())
                    .collect();
                blocks.push(self.table_block(&labels, &group.members));
            }
        }
        blocks
    }

    /// The flat block: one table over every data node, when any.
    fn flat_blocks(&self) -> Vec<String> {
        let (labels, ids): (Vec<String>, Vec<U32Id<BTreeGridNode>>) =
            self.data_paths().into_iter().unzip();
        if ids.is_empty() {
            return Vec::new();
        }
        vec![self.table_block(&labels, &ids)]
    }

    /// One table over `ids`: a `#` index column, then one column per
    /// node headed by its label, one row per value index.
    fn table_block(&self, labels: &[String], ids: &[U32Id<BTreeGridNode>]) -> String {
        let columns: Vec<Vec<Cell>> = ids
            .iter()
            .map(|&id| {
                let node = self.node(id);
                node.values
                    .iter()
                    .map(|value| table_cell(Cell::render(self.cells(), node.format, value)))
                    .collect()
            })
            .collect();
        let row_count = columns.iter().map(Vec::len).max().unwrap_or(0);

        let mut headers = vec![Cell::text("#")];
        headers.extend(labels.iter().map(|label| table_cell(Cell::text(label))));
        let rows: Vec<Vec<Cell>> = (0..row_count)
            .map(|row| {
                let mut cells = vec![Cell::text(row.to_string())];
                cells.extend(
                    columns
                        .iter()
                        .map(|column| column.get(row).cloned().unwrap_or_else(|| Cell::text(""))),
                );
                cells
            })
            .collect();
        let mut table = render_tables::markdown_table(&headers, &rows);
        // markdown_table ends its last line with `\n`; the block join
        // re-adds it.
        table.pop();
        table
    }
}

/// The cell escaped for a table: pipes escape and newlines flatten,
/// the added backslashes widening the cell. A bare visual's opaque
/// bytes pass verbatim.
fn table_cell(cell: Cell) -> Cell {
    if cell.bare_visual {
        return cell;
    }
    Cell {
        width: cell.width + cell.rendered.matches('|').count(),
        rendered: render_tables::markdown_cell(&cell.rendered),
        bare_visual: false,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGrid, TreeGridCellFormat, TreeGridLabel, TreeGridNestedTableOptions,
        TreeGridRenderTables, TreeGridTableLabelMode, TreeGridTableShape, TreeGridValue,
    };
    use std::num::NonZeroU8;

    /// The rendering spec's worked example.
    fn worked_example() -> TreeGrid {
        let mut grid = TreeGrid::new();
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let base = grid.add_child(palette, TreeGridLabel::quoted("baseColorFactor"));
        grid.node_mut(base).format = Some(TreeGridCellFormat::Text);
        grid.push_value(base, TreeGridValue::srgba8([255, 0, 0, 255]));
        grid.push_value(base, TreeGridValue::srgba8([0, 255, 0, 128]));
        let metallic = grid.add_child(palette, TreeGridLabel::quoted("metallicFactor"));
        grid.node_mut(metallic).format = Some(TreeGridCellFormat::Text);
        grid.push_value(metallic, TreeGridValue::unorm(1.0));
        grid.push_value(metallic, TreeGridValue::unorm(0.2));
        let second = grid.add_root(TreeGridLabel::bare("1"));
        let attribute = grid.add_child(second, TreeGridLabel::quoted("baseColorFactor"));
        let component = grid.add_child(attribute, TreeGridLabel::bare("a"));
        grid.node_mut(component).format = Some(TreeGridCellFormat::Text);
        grid.push_value(component, TreeGridValue::unorm8(255));
        grid
    }

    /// The rendering spec's hierarchy-data worked example, cut to one
    /// full scene node and one tag-only sibling.
    fn hierarchy_example() -> TreeGrid {
        let mut grid = TreeGrid::new();
        let root = grid.add_root(TreeGridLabel::bare("root"));
        let tank = grid.add_child(root, TreeGridLabel::quoted("energy-tank-1"));
        grid.push_value(tank, TreeGridValue::new("{node: 0}"));
        let transform = grid.add_child(tank, TreeGridLabel::bare("transform"));
        let position = grid.add_child(transform, TreeGridLabel::bare("position"));
        grid.push_value(position, TreeGridValue::new("[12.50, 0.50, 10.00]"));
        let rotation = grid.add_child(transform, TreeGridLabel::bare("rotation"));
        grid.push_value(rotation, TreeGridValue::new("[0.00, 0.00, 0.00]"));
        let scale = grid.add_child(transform, TreeGridLabel::bare("scale"));
        grid.push_value(scale, TreeGridValue::new("[1.00, 1.00, 1.00]"));
        let object = grid.add_child(tank, TreeGridLabel::quoted("energy-tank-1"));
        grid.push_value(object, TreeGridValue::new("{object: 0, instance: 0}"));
        let second = grid.add_child(root, TreeGridLabel::quoted("energy-tank-2"));
        grid.push_value(second, TreeGridValue::new("{node: 1}"));
        grid
    }

    #[test]
    fn an_empty_grid_renders_the_empty_string() {
        assert_eq!(
            TreeGrid::new().render_tables(&TreeGridTableShape::default()),
            ""
        );
        assert_eq!(TreeGrid::new().render_tables(&TreeGridTableShape::Flat), "");
    }

    #[test]
    fn nested_concat_headings_carry_full_paths() {
        assert_eq!(
            worked_example().render_tables(&TreeGridTableShape::default()),
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
            worked_example().render_tables(&shape),
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
            hierarchy_example().render_tables(&shape),
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
            hierarchy_example().render_tables(&TreeGridTableShape::default()),
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
        let count = grid.add_root(TreeGridLabel::bare("materialCount"));
        grid.push_value(count, TreeGridValue::int(2));
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let metallic = grid.add_child(palette, TreeGridLabel::quoted("metallicFactor"));
        grid.node_mut(metallic).format = Some(TreeGridCellFormat::Text);
        grid.push_value(metallic, TreeGridValue::unorm(0.2));

        assert_eq!(
            grid.render_tables(&TreeGridTableShape::default()),
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
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let strength = grid.add_child(palette, TreeGridLabel::quoted("emissiveStrength"));
        grid.node_mut(strength).annotation = Some("(scalar)".to_string());
        grid.push_value(strength, TreeGridValue::float(2.0));
        let tint = grid.add_child(palette, TreeGridLabel::quoted("tint"));
        grid.node_mut(tint).annotation = Some("(scalar)".to_string());
        let alpha = grid.add_child(tint, TreeGridLabel::bare("a"));
        grid.node_mut(alpha).format = Some(TreeGridCellFormat::Text);
        grid.push_value(alpha, TreeGridValue::unorm8(128));
        let sub = grid.add_child(tint, TreeGridLabel::bare("sub"));
        let leaf = grid.add_child(sub, TreeGridLabel::bare("b"));
        grid.push_value(leaf, TreeGridValue::int(7));

        // The heading path stack stays bare, so `sub`'s concat
        // heading skips its parent's annotation.
        assert_eq!(
            grid.render_tables(&TreeGridTableShape::default()),
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
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let strength = grid.add_child(palette, TreeGridLabel::quoted("emissiveStrength"));
        grid.node_mut(strength).annotation = Some("(scalar)".to_string());
        grid.push_value(strength, TreeGridValue::float(2.0));

        assert_eq!(
            grid.render_tables(&TreeGridTableShape::Flat),
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
            worked_example().render_tables(&shape),
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
            worked_example().render_tables(&TreeGridTableShape::Flat),
            "| #   | 0.\"baseColorFactor\" | 0.\"metallicFactor\" | 1.\"baseColorFactor\".a |\n\
             | --- | ------------------- | ------------------ | --------------------- |\n\
             | 0   | #FF0000FF           | 1                  | 255                   |\n\
             | 1   | #00FF0080           | 0.2                |                       |\n"
        );
    }

    #[test]
    fn labels_and_cells_escape_pipes() {
        let mut grid = TreeGrid::new();
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let attribute = grid.add_child(palette, TreeGridLabel::quoted("a|b"));
        grid.node_mut(attribute).format = Some(TreeGridCellFormat::Text);
        grid.push_value(attribute, TreeGridValue::new("x|y"));

        assert_eq!(
            grid.render_tables(&TreeGridTableShape::default()),
            "# 0\n\
             \n\
             | #   | \"a\\|b\" |\n\
             | --- | ------ |\n\
             | 0   | x\\|y   |\n"
        );
    }

    #[test]
    fn a_visual_cell_pads_by_its_declared_width() {
        let mut grid = TreeGrid::new();
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let color = grid.add_child(palette, TreeGridLabel::quoted("c"));
        grid.push_value(color, TreeGridValue::srgba8([255, 0, 0, 255]));

        assert_eq!(
            grid.render_tables(&TreeGridTableShape::default()),
            "# 0\n\
             \n\
             | #   | \"c\"          |\n\
             | --- | ------------ |\n\
             | 0   | \x1b[48;2;255;0;0m  \x1b[0m #FF0000FF |\n"
        );
    }
}
