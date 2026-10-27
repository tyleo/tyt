use crate::{
    TreeGrid, TreeGridCells, TreeGridTableLabelMode, TreeGridTableShapeKind,
    render::TreeGridRenderFramedTables, render_box_tables::BoxTableFrame,
};

/// The `box-tables` render.
pub trait TreeGridRenderBoxTables {
    /// Renders the `box-tables` layout: box-glyph tables in `shape`, each
    /// section headed by a bare line carrying its full path.
    fn render_box_tables(&self, shape: TreeGridTableShapeKind) -> String;
}

impl<C: TreeGridCells> TreeGridRenderBoxTables for TreeGrid<C> {
    fn render_box_tables(&self, shape: TreeGridTableShapeKind) -> String {
        self.render_framed_tables(shape, TreeGridTableLabelMode::Concat, &BoxTableFrame)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGrid, TreeGridCellFormat, TreeGridLabel, TreeGridRenderBoxTables,
        TreeGridTableShapeKind, TreeGridValue,
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

    #[test]
    fn an_empty_grid_renders_the_empty_string() {
        for shape in [
            TreeGridTableShapeKind::Flat,
            TreeGridTableShapeKind::Nested,
            TreeGridTableShapeKind::Records,
        ] {
            assert_eq!(TreeGrid::new().render_box_tables(shape), "");
        }
    }

    #[test]
    fn nested_sections_head_with_bare_full_paths() {
        assert_eq!(
            worked_example().render_box_tables(TreeGridTableShapeKind::Nested),
            "0\n\
             \n\
             ┌───┬───────────────────┬──────────────────┐\n\
             │ # │ \"baseColorFactor\" │ \"metallicFactor\" │\n\
             ├───┼───────────────────┼──────────────────┤\n\
             │ 0 │ #FF0000FF         │ 1                │\n\
             ├───┼───────────────────┼──────────────────┤\n\
             │ 1 │ #00FF0080         │ 0.2              │\n\
             └───┴───────────────────┴──────────────────┘\n\
             \n\
             1\n\
             \n\
             1.\"baseColorFactor\"\n\
             \n\
             ┌───┬─────┐\n\
             │ # │ a   │\n\
             ├───┼─────┤\n\
             │ 0 │ 255 │\n\
             └───┴─────┘\n"
        );
    }

    #[test]
    fn flat_tables_compare_across_the_forest() {
        assert_eq!(
            worked_example().render_box_tables(TreeGridTableShapeKind::Flat),
            "┌───┬─────────────────────┬────────────────────┬───────────────────────┐\n\
             │ # │ 0.\"baseColorFactor\" │ 0.\"metallicFactor\" │ 1.\"baseColorFactor\".a │\n\
             ├───┼─────────────────────┼────────────────────┼───────────────────────┤\n\
             │ 0 │ #FF0000FF           │ 1                  │ 255                   │\n\
             ├───┼─────────────────────┼────────────────────┼───────────────────────┤\n\
             │ 1 │ #00FF0080           │ 0.2                │                       │\n\
             └───┴─────────────────────┴────────────────────┴───────────────────────┘\n"
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
            grid.render_box_tables(TreeGridTableShapeKind::Records),
            "palettes\n\
             \n\
             ┌───────┬───────────────┬────────────────────────────────┐\n\
             │ label │ materialCount │ properties                     │\n\
             ├───────┼───────────────┼────────────────────────────────┤\n\
             │ 0     │ 2             │ baseColorFactor metallicFactor │\n\
             ├───────┼───────────────┼────────────────────────────────┤\n\
             │ 1     │ 1             │ baseColorFactor                │\n\
             └───────┴───────────────┴────────────────────────────────┘\n"
        );
    }

    #[test]
    fn labels_and_cells_keep_pipes() {
        let mut grid = TreeGrid::new();
        let palette = grid.retain_root(TreeGridLabel::bare("0"));
        let attribute = grid.retain_child(palette, TreeGridLabel::quoted("a|b"));
        grid.node_mut(attribute).format = Some(TreeGridCellFormat::Text);
        grid.push_value(attribute, TreeGridValue::new("x|y"));

        assert_eq!(
            grid.render_box_tables(TreeGridTableShapeKind::Nested),
            "0\n\
             \n\
             ┌───┬───────┐\n\
             │ # │ \"a|b\" │\n\
             ├───┼───────┤\n\
             │ 0 │ x|y   │\n\
             └───┴───────┘\n"
        );
    }

    #[test]
    fn a_visual_cell_pads_by_its_declared_width() {
        let mut grid = TreeGrid::new();
        let palette = grid.retain_root(TreeGridLabel::bare("0"));
        let color = grid.retain_child(palette, TreeGridLabel::quoted("c"));
        grid.push_value(color, TreeGridValue::srgba8([255, 0, 0, 255]));

        assert_eq!(
            grid.render_box_tables(TreeGridTableShapeKind::Nested),
            "0\n\
             \n\
             ┌───┬──────────────┐\n\
             │ # │ \"c\"          │\n\
             ├───┼──────────────┤\n\
             │ 0 │ \x1b[48;2;255;0;0m  \x1b[0m #FF0000FF │\n\
             └───┴──────────────┘\n"
        );
    }
}
