use crate::{
    BTreeGridNode, TreeGrid, TreeGridCells, TreeGridColumnsOptions, TreeGridLabelMode,
    render::{self, Cell},
};
use branded_id::U32Id;

/// The `columns` render.
pub trait TreeGridRenderColumns {
    /// Renders the `columns` layout: one padded column of cells per
    /// data node, in pre-order, values reading straight down.
    fn render_columns(&self, options: &TreeGridColumnsOptions) -> String;
}

impl<C: TreeGridCells> TreeGridRenderColumns for TreeGrid<C> {
    fn render_columns(&self, options: &TreeGridColumnsOptions) -> String {
        let mut blocks: Vec<String> = Vec::new();
        match options.label {
            TreeGridLabelMode::None => {
                let ids: Vec<U32Id<BTreeGridNode>> =
                    self.data_paths().into_iter().map(|(_, id)| id).collect();
                blocks.extend(self.column_block(None, &ids));
            }
            TreeGridLabelMode::Concat => {
                let (labels, ids): (Vec<String>, Vec<U32Id<BTreeGridNode>>) =
                    self.data_paths().into_iter().unzip();
                blocks.extend(self.column_block(Some(&labels), &ids));
            }
            TreeGridLabelMode::Header(header) => {
                for group in self.groups() {
                    if let Some(branch) = group.branch {
                        blocks.push(render::heading(
                            header.level,
                            group.depth,
                            &self.node(branch).annotated_label(),
                        ));
                    }
                    let labels: Vec<String> = group
                        .members
                        .iter()
                        .map(|&id| self.node(id).annotated_label())
                        .collect();
                    blocks.extend(self.column_block(Some(&labels), &group.members));
                }
            }
        }
        if blocks.is_empty() {
            String::new()
        } else {
            format!("{}\n", blocks.join("\n\n"))
        }
    }
}

impl<C: TreeGridCells> TreeGrid<C> {
    /// One block of columns: an optional label line, then one line
    /// per value index; `None` when there are no columns.
    fn column_block(
        &self,
        labels: Option<&[String]>,
        ids: &[U32Id<BTreeGridNode>],
    ) -> Option<String> {
        if ids.is_empty() {
            return None;
        }
        let columns: Vec<Vec<Cell>> = ids
            .iter()
            .map(|&id| {
                let node = self.node(id);
                node.values
                    .iter()
                    .map(|value| Cell::render(self.cells(), node.format, value))
                    .collect()
            })
            .collect();
        let widths: Vec<usize> = columns
            .iter()
            .enumerate()
            .map(|(index, column)| {
                let cells = column.iter().map(|cell| cell.width).max().unwrap_or(0);
                match labels {
                    Some(labels) => render::visible_width(&labels[index]).max(cells),
                    None => cells,
                }
            })
            .collect();
        let row_count = columns.iter().map(Vec::len).max().unwrap_or(0);

        let mut lines: Vec<String> = Vec::new();
        if let Some(labels) = labels {
            lines.push(join_padded(labels.iter().map(String::as_str), &widths));
        }
        for row in 0..row_count {
            lines.push(join_padded(
                columns
                    .iter()
                    .map(|column| column.get(row).map_or("", |cell| cell.rendered.as_str())),
                &widths,
            ));
        }
        Some(lines.join("\n"))
    }
}

/// Joins one value per column, each padded to its column's width, with
/// one space between columns, right-trimmed.
fn join_padded<'a>(values: impl Iterator<Item = &'a str>, widths: &[usize]) -> String {
    values
        .zip(widths)
        .map(|(value, &width)| render::pad_right(value, width))
        .collect::<Vec<_>>()
        .join(" ")
        .trim_end()
        .to_string()
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGrid, TreeGridCellFormat, TreeGridColumnsOptions, TreeGridHeaderOptions, TreeGridLabel,
        TreeGridLabelMode, TreeGridRenderColumns, TreeGridValue,
    };

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

    /// Two alpha-component columns, the first one value longer.
    fn component_columns() -> TreeGrid {
        let mut grid = TreeGrid::new();
        let first = grid.retain_root(TreeGridLabel::bare("0"));
        let attribute = grid.retain_child(first, TreeGridLabel::quoted("baseColorFactor"));
        let component = grid.retain_child(attribute, TreeGridLabel::bare("a"));
        grid.node_mut(component).format = Some(TreeGridCellFormat::Text);
        grid.push_value(component, TreeGridValue::unorm8(255));
        grid.push_value(component, TreeGridValue::unorm8(128));
        let second = grid.retain_root(TreeGridLabel::bare("1"));
        let attribute = grid.retain_child(second, TreeGridLabel::quoted("baseColorFactor"));
        let component = grid.retain_child(attribute, TreeGridLabel::bare("a"));
        grid.node_mut(component).format = Some(TreeGridCellFormat::Text);
        grid.push_value(component, TreeGridValue::unorm8(255));
        grid
    }

    #[test]
    fn an_empty_grid_renders_the_empty_string() {
        assert_eq!(
            TreeGrid::new().render_columns(&TreeGridColumnsOptions::default()),
            ""
        );
    }

    #[test]
    fn concat_labels_top_the_columns() {
        assert_eq!(
            component_columns().render_columns(&TreeGridColumnsOptions::default()),
            "0.\"baseColorFactor\".a 1.\"baseColorFactor\".a\n\
             255                   255\n\
             128\n"
        );
    }

    #[test]
    fn none_labels_drop_the_label_line() {
        let options = TreeGridColumnsOptions::default().with_label(TreeGridLabelMode::None);
        assert_eq!(
            component_columns().render_columns(&options),
            "255 255\n\
             128\n"
        );
    }

    #[test]
    fn a_label_widens_its_column_and_short_columns_leave_blanks() {
        assert_eq!(
            worked_example().render_columns(&TreeGridColumnsOptions::default()),
            "0.\"baseColorFactor\" 0.\"metallicFactor\" 1.\"baseColorFactor\".a\n\
             #FF0000FF           1                  255\n\
             #00FF0080           0.2\n"
        );
    }

    #[test]
    fn header_labels_group_column_blocks_under_headings() {
        let options = TreeGridColumnsOptions::default()
            .with_label(TreeGridLabelMode::Header(TreeGridHeaderOptions::default()));
        assert_eq!(
            worked_example().render_columns(&options),
            "# 0\n\
             \n\
             \"baseColorFactor\" \"metallicFactor\"\n\
             #FF0000FF         1\n\
             #00FF0080         0.2\n\
             \n\
             # 1\n\
             \n\
             ## \"baseColorFactor\"\n\
             \n\
             a\n\
             255\n"
        );
    }

    #[test]
    fn an_annotation_suffixes_and_widens_the_column_label() {
        let mut grid = TreeGrid::new();
        let palette = grid.retain_root(TreeGridLabel::bare("0"));
        let strength = grid.retain_child(palette, TreeGridLabel::quoted("emissiveStrength"));
        grid.node_mut(strength).annotation = Some("(scalar)".to_string());
        grid.push_value(strength, TreeGridValue::float(2.0));
        let base = grid.retain_child(palette, TreeGridLabel::quoted("baseColorFactor"));
        grid.node_mut(base).format = Some(TreeGridCellFormat::Text);
        grid.push_value(base, TreeGridValue::srgba8([255, 0, 0, 255]));

        assert_eq!(
            grid.render_columns(&TreeGridColumnsOptions::default()),
            "0.\"emissiveStrength\" (scalar) 0.\"baseColorFactor\"\n\
             2                             #FF0000FF\n"
        );
    }

    #[test]
    fn an_annotation_suffixes_the_header_label_and_heading() {
        let mut grid = TreeGrid::new();
        let palette = grid.retain_root(TreeGridLabel::bare("0"));
        let tint = grid.retain_child(palette, TreeGridLabel::quoted("tint"));
        grid.node_mut(tint).annotation = Some("(scalar)".to_string());
        grid.node_mut(tint).format = Some(TreeGridCellFormat::Text);
        grid.push_value(tint, TreeGridValue::srgba8([0, 255, 0, 128]));
        let alpha = grid.retain_child(tint, TreeGridLabel::bare("a"));
        grid.node_mut(alpha).format = Some(TreeGridCellFormat::Text);
        grid.push_value(alpha, TreeGridValue::unorm8(128));

        let options = TreeGridColumnsOptions::default()
            .with_label(TreeGridLabelMode::Header(TreeGridHeaderOptions::default()));
        assert_eq!(
            grid.render_columns(&options),
            "# 0\n\
             \n\
             \"tint\" (scalar)\n\
             #00FF0080\n\
             \n\
             ## \"tint\" (scalar)\n\
             \n\
             a\n\
             128\n"
        );
    }

    #[test]
    fn swatch_columns_align_past_ansi_escapes() {
        let mut grid = TreeGrid::new();
        let palette = grid.retain_root(TreeGridLabel::bare("0"));
        let base = grid.retain_child(palette, TreeGridLabel::quoted("baseColorFactor"));
        grid.push_value(base, TreeGridValue::srgba8([255, 0, 0, 255]));
        grid.push_value(base, TreeGridValue::srgba8([0, 255, 0, 128]));
        let metallic = grid.retain_child(palette, TreeGridLabel::quoted("metallicFactor"));
        grid.push_value(metallic, TreeGridValue::float(1.0));
        grid.push_value(metallic, TreeGridValue::float(0.2));

        // The decorated cells are 12 columns visible, so they pad to
        // the 19-column label like plain text.
        assert_eq!(
            grid.render_columns(&TreeGridColumnsOptions::default()),
            "0.\"baseColorFactor\" 0.\"metallicFactor\"\n\
             \x1b[48;2;255;0;0m  \x1b[0m #FF0000FF        1\n\
             \x1b[48;2;0;255;0m  \x1b[0m #00FF0080        0.2\n"
        );
    }
}
