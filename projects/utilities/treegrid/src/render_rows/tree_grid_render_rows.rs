use crate::{
    BTreeGridNode, TreeGrid, TreeGridCells, TreeGridLabelMode, TreeGridRowsOptions,
    render::{self, Cell},
};
use branded_id::U32Id;

/// The `rows` render.
pub trait TreeGridRenderRows {
    /// Renders the `rows` layout: one labeled row of cells per data
    /// node, in pre-order, with a blank line between rows.
    fn render_rows(&self, options: &TreeGridRowsOptions) -> String;
}

impl<C: TreeGridCells> TreeGridRenderRows for TreeGrid<C> {
    fn render_rows(&self, options: &TreeGridRowsOptions) -> String {
        let mut blocks: Vec<String> = Vec::new();
        match options.label {
            TreeGridLabelMode::None => {
                for (_, id) in self.data_paths() {
                    blocks.push(self.row_block(None, 0, id, options.width));
                }
            }
            TreeGridLabelMode::Concat => {
                let rows = self.data_paths();
                let width = label_width(rows.iter().map(|(path, _)| path.as_str()));
                for (path, id) in &rows {
                    blocks.push(self.row_block(Some(path), width, *id, options.width));
                }
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
                    let width = label_width(labels.iter().map(String::as_str));
                    for (label, &id) in labels.iter().zip(&group.members) {
                        blocks.push(self.row_block(Some(label), width, id, options.width));
                    }
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
    /// One node's row block: padded label, cells, indented
    /// continuation lines.
    fn row_block(
        &self,
        label: Option<&str>,
        label_width: usize,
        id: U32Id<BTreeGridNode>,
        width: Option<usize>,
    ) -> String {
        let node = self.node(id);
        let cells: Vec<Cell> = node
            .values
            .iter()
            .map(|value| Cell::render(self.cells(), node.format, value))
            .collect();
        let separator = Cell::separator(&cells);
        let indent = if label.is_some() { label_width + 1 } else { 0 };
        let segments = match width {
            // Leave room for at least one cell beside the label indent.
            Some(width) => wrap_cells(&cells, separator, width.saturating_sub(indent).max(1)),
            None => {
                let rendered: Vec<&str> = cells.iter().map(|cell| cell.rendered.as_str()).collect();
                vec![rendered.join(separator)]
            }
        };
        segments
            .iter()
            .enumerate()
            .map(|(line, segment)| {
                let prefix = match (line, label) {
                    (0, Some(label)) => format!("{} ", render::pad_right(label, label_width)),
                    (0, None) => String::new(),
                    _ => " ".repeat(indent),
                };
                format!("{prefix}{segment}").trim_end().to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn label_width<'a>(labels: impl Iterator<Item = &'a str>) -> usize {
    labels.map(render::visible_width).max().unwrap_or(0)
}

/// Greedily packs cells into segments of at most `budget` visible
/// columns, joined with `separator`; every segment gets at least one
/// cell.
fn wrap_cells(cells: &[Cell], separator: &str, budget: usize) -> Vec<String> {
    let separator_width = separator.chars().count();
    let mut segments: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut width = 0;
    for cell in cells {
        if !current.is_empty() && width + separator_width + cell.width > budget {
            segments.push(current.join(separator));
            current.clear();
            width = 0;
        }
        if !current.is_empty() {
            width += separator_width;
        }
        width += cell.width;
        current.push(&cell.rendered);
    }
    if !current.is_empty() {
        segments.push(current.join(separator));
    }
    segments
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGrid, TreeGridCellFormat, TreeGridHeaderOptions, TreeGridLabel, TreeGridLabelMode,
        TreeGridRenderRows, TreeGridRowsOptions, TreeGridValue,
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

    #[test]
    fn an_empty_grid_renders_the_empty_string() {
        assert_eq!(
            TreeGrid::new().render_rows(&TreeGridRowsOptions::default()),
            ""
        );
    }

    #[test]
    fn concat_labels_pad_to_the_longest_path() {
        assert_eq!(
            worked_example().render_rows(&TreeGridRowsOptions::default()),
            "0.\"baseColorFactor\"   #FF0000FF #00FF0080\n\
             \n\
             0.\"metallicFactor\"    1 0.2\n\
             \n\
             1.\"baseColorFactor\".a 255\n"
        );
    }

    #[test]
    fn none_labels_drop_the_label_column() {
        let options = TreeGridRowsOptions::default().with_label(TreeGridLabelMode::None);
        assert_eq!(
            worked_example().render_rows(&options),
            "#FF0000FF #00FF0080\n\
             \n\
             1 0.2\n\
             \n\
             255\n"
        );
    }

    #[test]
    fn header_labels_nest_headings_and_pad_per_group() {
        let options = TreeGridRowsOptions::default()
            .with_label(TreeGridLabelMode::Header(TreeGridHeaderOptions::default()));
        assert_eq!(
            worked_example().render_rows(&options),
            "# 0\n\
             \n\
             \"baseColorFactor\" #FF0000FF #00FF0080\n\
             \n\
             \"metallicFactor\"  1 0.2\n\
             \n\
             # 1\n\
             \n\
             ## \"baseColorFactor\"\n\
             \n\
             a 255\n"
        );
    }

    #[test]
    fn root_level_data_prints_first_with_no_heading() {
        let mut grid = TreeGrid::new();
        let count = grid.add_root(TreeGridLabel::bare("materialCount"));
        grid.push_value(count, TreeGridValue::int(2));
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let metallic = grid.add_child(palette, TreeGridLabel::quoted("metallicFactor"));
        grid.node_mut(metallic).format = Some(TreeGridCellFormat::Text);
        grid.push_value(metallic, TreeGridValue::unorm(0.2));

        let options = TreeGridRowsOptions::default()
            .with_label(TreeGridLabelMode::Header(TreeGridHeaderOptions::default()));
        assert_eq!(
            grid.render_rows(&options),
            "materialCount 2\n\
             \n\
             # 0\n\
             \n\
             \"metallicFactor\" 0.2\n"
        );
    }

    #[test]
    fn a_header_level_deepens_every_heading() {
        let options = TreeGridRowsOptions::default().with_label(TreeGridLabelMode::Header(
            TreeGridHeaderOptions::default().with_level(NonZeroU8::new(2).unwrap()),
        ));
        assert_eq!(
            worked_example().render_rows(&options),
            "## 0\n\
             \n\
             \"baseColorFactor\" #FF0000FF #00FF0080\n\
             \n\
             \"metallicFactor\"  1 0.2\n\
             \n\
             ## 1\n\
             \n\
             ### \"baseColorFactor\"\n\
             \n\
             a 255\n"
        );
    }

    #[test]
    fn a_heading_past_level_six_renders_bold() {
        let options = TreeGridRowsOptions::default().with_label(TreeGridLabelMode::Header(
            TreeGridHeaderOptions::default().with_level(NonZeroU8::new(6).unwrap()),
        ));
        assert_eq!(
            worked_example().render_rows(&options),
            "###### 0\n\
             \n\
             \"baseColorFactor\" #FF0000FF #00FF0080\n\
             \n\
             \"metallicFactor\"  1 0.2\n\
             \n\
             ###### 1\n\
             \n\
             **\"baseColorFactor\"**\n\
             \n\
             a 255\n"
        );
    }

    #[test]
    fn an_annotation_suffixes_and_widens_the_concat_label() {
        let mut grid = TreeGrid::new();
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let base = grid.add_child(palette, TreeGridLabel::quoted("baseColorFactor"));
        grid.node_mut(base).format = Some(TreeGridCellFormat::Text);
        grid.push_value(base, TreeGridValue::srgba8([255, 0, 0, 255]));
        let strength = grid.add_child(palette, TreeGridLabel::quoted("emissiveStrength"));
        grid.node_mut(strength).annotation = Some("(scalar)".to_string());
        grid.push_value(strength, TreeGridValue::float(2.0));

        assert_eq!(
            grid.render_rows(&TreeGridRowsOptions::default()),
            "0.\"baseColorFactor\"           #FF0000FF\n\
             \n\
             0.\"emissiveStrength\" (scalar) 2\n"
        );
    }

    #[test]
    fn an_annotated_branch_keeps_descendant_paths_bare() {
        let mut grid = TreeGrid::new();
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let tint = grid.add_child(palette, TreeGridLabel::quoted("tint"));
        grid.node_mut(tint).annotation = Some("(scalar)".to_string());
        grid.node_mut(tint).format = Some(TreeGridCellFormat::Text);
        grid.push_value(tint, TreeGridValue::srgba8([0, 255, 0, 128]));
        let alpha = grid.add_child(tint, TreeGridLabel::bare("a"));
        grid.node_mut(alpha).format = Some(TreeGridCellFormat::Text);
        grid.push_value(alpha, TreeGridValue::unorm8(128));

        assert_eq!(
            grid.render_rows(&TreeGridRowsOptions::default()),
            "0.\"tint\" (scalar) #00FF0080\n\
             \n\
             0.\"tint\".a        128\n"
        );
    }

    #[test]
    fn an_annotation_suffixes_the_header_label_and_heading() {
        let mut grid = TreeGrid::new();
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let tint = grid.add_child(palette, TreeGridLabel::quoted("tint"));
        grid.node_mut(tint).annotation = Some("(scalar)".to_string());
        grid.node_mut(tint).format = Some(TreeGridCellFormat::Text);
        grid.push_value(tint, TreeGridValue::srgba8([0, 255, 0, 128]));
        let alpha = grid.add_child(tint, TreeGridLabel::bare("a"));
        grid.node_mut(alpha).format = Some(TreeGridCellFormat::Text);
        grid.push_value(alpha, TreeGridValue::unorm8(128));

        let options = TreeGridRowsOptions::default()
            .with_label(TreeGridLabelMode::Header(TreeGridHeaderOptions::default()));
        assert_eq!(
            grid.render_rows(&options),
            "# 0\n\
             \n\
             \"tint\" (scalar) #00FF0080\n\
             \n\
             ## \"tint\" (scalar)\n\
             \n\
             a 128\n"
        );
    }

    #[test]
    fn a_width_wraps_cells_under_the_first_cell_column() {
        let mut grid = TreeGrid::new();
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let base = grid.add_child(palette, TreeGridLabel::quoted("baseColorFactor"));
        grid.node_mut(base).format = Some(TreeGridCellFormat::Text);
        grid.push_value(base, TreeGridValue::srgba8([255, 0, 0, 255]));
        grid.push_value(base, TreeGridValue::srgba8([0, 255, 0, 128]));

        // The 20-column label indent leaves room for one 9-wide hex
        // per line.
        let options = TreeGridRowsOptions::default().with_width(30);
        assert_eq!(
            grid.render_rows(&options),
            "0.\"baseColorFactor\" #FF0000FF\n                    #00FF0080\n"
        );
    }

    #[test]
    fn a_cell_wider_than_the_budget_takes_a_line_of_its_own() {
        let mut grid = TreeGrid::new();
        let node = grid.add_root(TreeGridLabel::bare("values"));
        grid.push_value(node, TreeGridValue::new("abcdef"));
        grid.push_value(node, TreeGridValue::new("gh"));

        let options = TreeGridRowsOptions::default()
            .with_label(TreeGridLabelMode::None)
            .with_width(5);
        assert_eq!(grid.render_rows(&options), "abcdef\ngh\n");
    }

    #[test]
    fn a_visual_row_abuts_into_a_strip() {
        let mut grid = TreeGrid::new();
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let base = grid.add_child(palette, TreeGridLabel::quoted("baseColorFactor"));
        grid.node_mut(base).format = Some(TreeGridCellFormat::Visual);
        grid.push_value(base, TreeGridValue::srgba8([255, 0, 0, 255]));
        grid.push_value(base, TreeGridValue::srgba8([0, 255, 0, 128]));

        assert_eq!(
            grid.render_rows(&TreeGridRowsOptions::default()),
            "0.\"baseColorFactor\" \x1b[48;2;255;0;0m  \x1b[0m\x1b[48;2;0;255;0m  \x1b[0m\n"
        );
    }

    #[test]
    fn a_visual_row_spaces_cells_with_no_visual() {
        let mut grid = TreeGrid::new();
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let shadows = grid.add_child(palette, TreeGridLabel::quoted("shadows"));
        grid.node_mut(shadows).format = Some(TreeGridCellFormat::Visual);
        grid.push_value(shadows, TreeGridValue::bool(true));
        grid.push_value(shadows, TreeGridValue::bool(false));

        // The space separator keeps bools from running into
        // `truefalse`.
        assert_eq!(
            grid.render_rows(&TreeGridRowsOptions::default()),
            "0.\"shadows\" true false\n"
        );
    }

    #[test]
    fn an_unset_format_decorates_colors_and_prints_numbers_plain() {
        let mut grid = TreeGrid::new();
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let base = grid.add_child(palette, TreeGridLabel::quoted("baseColorFactor"));
        grid.push_value(base, TreeGridValue::srgba8([255, 0, 0, 255]));
        grid.push_value(base, TreeGridValue::srgba8([0, 255, 0, 128]));
        let metallic = grid.add_child(palette, TreeGridLabel::quoted("metallicFactor"));
        grid.push_value(metallic, TreeGridValue::float(1.0));
        grid.push_value(metallic, TreeGridValue::float(0.2));

        assert_eq!(
            grid.render_rows(&TreeGridRowsOptions::default()),
            "0.\"baseColorFactor\" \x1b[48;2;255;0;0m  \x1b[0m #FF0000FF \x1b[48;2;0;255;0m  \x1b[0m #00FF0080\n\
             \n\
             0.\"metallicFactor\"  1 0.2\n"
        );
    }
}
