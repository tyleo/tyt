use crate::{
    BTreeGridNode, TreeGrid, TreeGridCells, TreeGridNestedTableOptions,
    TreeGridRecordsTableOptions, TreeGridTableLabelMode, TreeGridTableShape,
    render::{self, Cell},
    render_tables,
};
use branded_id::U32Id;

/// The `tables` render.
pub trait TreeGridRenderTables {
    /// Renders the `tables` layout: aligned markdown tables in the
    /// given shape.
    fn render_tables(&self, shape: &TreeGridTableShape) -> String;
}

impl<C: TreeGridCells> TreeGridRenderTables for TreeGrid<C> {
    fn render_tables(&self, shape: &TreeGridTableShape) -> String {
        let blocks = match shape {
            TreeGridTableShape::Nested(options) => self.nested_blocks(options),
            TreeGridTableShape::Flat => self.flat_blocks(),
            TreeGridTableShape::Records(options) => self.records_blocks(options),
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
        // the branch's heading.
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

    /// The records blocks: a heading-less table of the data-bearing
    /// roots, then one heading and table per value-less root that
    /// leads to data.
    fn records_blocks(&self, options: &TreeGridRecordsTableOptions) -> Vec<String> {
        let mut blocks: Vec<String> = Vec::new();
        let leading: Vec<U32Id<BTreeGridNode>> = self
            .roots()
            .iter()
            .copied()
            .filter(|&root| !self.node(root).values.is_empty())
            .collect();
        if !leading.is_empty() {
            blocks.push(self.records_table(&leading));
        }
        for &root in self.roots() {
            let node = self.node(root);
            if !node.values.is_empty() || !self.leads_to_data(root) {
                continue;
            }
            blocks.push(render::heading(options.level, 0, &node.annotated_label()));
            let rows: Vec<U32Id<BTreeGridNode>> = node
                .children()
                .iter()
                .copied()
                .filter(|&child| self.bears_data(child))
                .collect();
            blocks.push(self.records_table(&rows));
        }
        blocks
    }

    /// One records table over `rows`: a `label` column, a `value`
    /// column when any row bears values, then the union of the rows'
    /// relative descendant data paths in first-encounter order. A row
    /// without data at a column leaves the cell blank.
    fn records_table(&self, rows: &[U32Id<BTreeGridNode>]) -> String {
        let row_entries: Vec<Vec<(String, Vec<U32Id<BTreeGridNode>>)>> =
            rows.iter().map(|&row| self.relative_entries(row)).collect();
        let mut columns: Vec<String> = Vec::new();
        for entries in &row_entries {
            for (path, _) in entries {
                if !columns.contains(path) {
                    columns.push(path.clone());
                }
            }
        }
        let has_value = rows.iter().any(|&row| !self.node(row).values.is_empty());

        let mut headers = vec![Cell::text("label")];
        if has_value {
            headers.push(Cell::text("value"));
        }
        headers.extend(columns.iter().map(|path| table_cell(Cell::text(path))));
        let table_rows: Vec<Vec<Cell>> = rows
            .iter()
            .zip(&row_entries)
            .map(|(&row, entries)| {
                let mut cells = vec![table_cell(Cell::text(self.node(row).annotated_label()))];
                if has_value {
                    cells.push(self.joined_cell(&[row]));
                }
                for path in &columns {
                    let ids = match entries.iter().find(|(entry, _)| entry == path) {
                        Some((_, ids)) => ids.as_slice(),
                        None => &[],
                    };
                    cells.push(self.joined_cell(ids));
                }
                cells
            })
            .collect();
        let mut table = render_tables::markdown_table(&headers, &table_rows);
        // markdown_table ends its last line with `\n`; the block join
        // re-adds it.
        table.pop();
        table
    }

    /// The relative descendant data paths under `row` in pre-order,
    /// same-path entries merged in encounter order.
    fn relative_entries(
        &self,
        row: U32Id<BTreeGridNode>,
    ) -> Vec<(String, Vec<U32Id<BTreeGridNode>>)> {
        let mut paths: Vec<(String, U32Id<BTreeGridNode>)> = Vec::new();
        for &child in self.node(row).children() {
            self.collect_data_paths(child, "", &mut paths);
        }
        let mut entries: Vec<(String, Vec<U32Id<BTreeGridNode>>)> = Vec::new();
        for (path, id) in paths {
            match entries.iter_mut().find(|(entry, _)| *entry == path) {
                Some((_, ids)) => ids.push(id),
                None => entries.push((path, vec![id])),
            }
        }
        entries
    }

    /// One escaped table cell joining every value of `ids` with the
    /// separator rule; no values leave the cell blank.
    fn joined_cell(&self, ids: &[U32Id<BTreeGridNode>]) -> Cell {
        let mut cells: Vec<Cell> = Vec::new();
        for &id in ids {
            let node = self.node(id);
            for value in &node.values {
                cells.push(Cell::render(self.cells(), node.format, value));
            }
        }
        if cells.is_empty() {
            return Cell::text("");
        }
        let separator = Cell::separator(&cells);
        let width = cells.iter().map(|cell| cell.width).sum::<usize>()
            + separator.len() * (cells.len() - 1);
        let rendered = cells
            .iter()
            .map(|cell| cell.rendered.as_str())
            .collect::<Vec<&str>>()
            .join(separator);
        table_cell(Cell {
            rendered,
            width,
            bare_visual: separator.is_empty(),
        })
    }

    /// Whether the node or any descendant bears values.
    fn bears_data(&self, id: U32Id<BTreeGridNode>) -> bool {
        !self.node(id).values.is_empty() || self.leads_to_data(id)
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
        TreeGridRecordsTableOptions, TreeGridRenderTables, TreeGridTableLabelMode,
        TreeGridTableShape, TreeGridValue,
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
        assert_eq!(
            TreeGrid::new().render_tables(&TreeGridTableShape::Records(
                TreeGridRecordsTableOptions::default()
            )),
            ""
        );
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
    fn records_transpose_one_row_per_entity() {
        assert_eq!(
            hierarchy_example().render_tables(&TreeGridTableShape::Records(
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
        let root = grid.add_root(TreeGridLabel::bare("palettes"));
        for (index, count, properties) in [
            ("0", 2, &["baseColorFactor", "metallicFactor"][..]),
            ("1", 1, &["baseColorFactor"][..]),
        ] {
            let palette = grid.add_child(root, TreeGridLabel::bare(index));
            let materials = grid.add_child(palette, TreeGridLabel::bare("materialCount"));
            grid.push_value(materials, TreeGridValue::int(count));
            let names = grid.add_child(palette, TreeGridLabel::bare("properties"));
            for property in properties {
                grid.push_value(names, TreeGridValue::new(*property));
            }
        }

        assert_eq!(
            grid.render_tables(&TreeGridTableShape::Records(
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
        let root = grid.add_root(TreeGridLabel::bare("Document"));
        let format = grid.add_child(root, TreeGridLabel::bare("Format"));
        grid.push_value(format, TreeGridValue::new("voxj"));
        let ext = grid.add_child(root, TreeGridLabel::bare("Has ext"));
        grid.push_value(ext, TreeGridValue::new("no"));

        assert_eq!(
            grid.render_tables(&TreeGridTableShape::Records(
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
        let stats = grid.add_root(TreeGridLabel::bare("stats"));
        grid.push_value(stats, TreeGridValue::int(5));
        let deep = grid.add_child(stats, TreeGridLabel::bare("deep"));
        grid.push_value(deep, TreeGridValue::int(9));
        let palettes = grid.add_root(TreeGridLabel::bare("palettes"));
        let palette = grid.add_child(palettes, TreeGridLabel::bare("0"));
        let materials = grid.add_child(palette, TreeGridLabel::bare("materialCount"));
        grid.push_value(materials, TreeGridValue::int(1));

        assert_eq!(
            grid.render_tables(&TreeGridTableShape::Records(
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
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let base = grid.add_child(palette, TreeGridLabel::quoted("baseColorFactor"));
        grid.node_mut(base).format = Some(TreeGridCellFormat::Visual);
        grid.push_value(base, TreeGridValue::srgba8([255, 0, 0, 255]));
        grid.push_value(base, TreeGridValue::srgba8([0, 255, 0, 128]));

        assert_eq!(
            grid.render_tables(&TreeGridTableShape::Records(
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
        let root = grid.add_root(TreeGridLabel::bare("palettes"));
        let tint = grid.add_child(root, TreeGridLabel::quoted("tint"));
        grid.node_mut(tint).annotation = Some("(scalar)".to_string());
        grid.push_value(tint, TreeGridValue::float(2.0));
        let alpha = grid.add_child(tint, TreeGridLabel::bare("a"));
        grid.node_mut(alpha).annotation = Some("(alpha)".to_string());
        grid.node_mut(alpha).format = Some(TreeGridCellFormat::Text);
        grid.push_value(alpha, TreeGridValue::unorm8(128));

        assert_eq!(
            grid.render_tables(&TreeGridTableShape::Records(
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
        let root = grid.add_root(TreeGridLabel::bare("r"));
        let entity = grid.add_child(root, TreeGridLabel::bare("e"));
        let first = grid.add_child(entity, TreeGridLabel::bare("x"));
        grid.push_value(first, TreeGridValue::int(1));
        let second = grid.add_child(entity, TreeGridLabel::bare("x"));
        grid.push_value(second, TreeGridValue::int(2));

        assert_eq!(
            grid.render_tables(&TreeGridTableShape::Records(
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
