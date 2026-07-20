use crate::{
    BTreeGridNode, TreeGrid, TreeGridCells, TreeGridHierarchyOptions,
    render::{CONNECTOR_LAST, CONNECTOR_MID, Cell, EXTENSION_LAST, EXTENSION_MID},
};
use branded_id::U32Id;

impl<C: TreeGridCells> TreeGrid<C> {
    /// Renders the `hierarchy` layout: a box-glyph tree with one line
    /// per node.
    ///
    /// A node line is the rendered label, then the annotation joined
    /// with one space when present, then `: ` and the node's cells
    /// when it has values. With
    /// [`value_children`](TreeGridHierarchyOptions::value_children)
    /// set, a data node prints its label alone and each value takes
    /// its own connector line beneath, before the node's children.
    /// With [`bare_roots`](TreeGridHierarchyOptions::bare_roots) set,
    /// each root prints on an unprefixed line and successive root
    /// sections separate with one blank line; unset, roots take
    /// connectors as siblings of one another. Every line ends with a
    /// newline; an empty grid renders as the empty string.
    pub fn render_hierarchy(&self, options: &TreeGridHierarchyOptions) -> String {
        let mut output = String::new();
        if options.bare_roots {
            for (index, &root) in self.roots().iter().enumerate() {
                if index > 0 {
                    output.push('\n');
                }
                let line = self.node_line(root, options);
                output.push_str(&format!("{line}\n"));
                self.render_under(&mut output, root, "", options);
            }
        } else {
            let total = self.roots().len();
            for (index, &root) in self.roots().iter().enumerate() {
                self.render_subtree(&mut output, root, "", index + 1 == total, options);
            }
        }
        output
    }

    /// Appends `id`'s connector line and everything beneath it, under
    /// `prefix`.
    fn render_subtree(
        &self,
        output: &mut String,
        id: U32Id<BTreeGridNode>,
        prefix: &str,
        last: bool,
        options: &TreeGridHierarchyOptions,
    ) {
        let connector = if last { CONNECTOR_LAST } else { CONNECTOR_MID };
        let line = self.node_line(id, options);
        output.push_str(&format!("{prefix}{connector} {line}\n"));

        let extension = if last { EXTENSION_LAST } else { EXTENSION_MID };
        self.render_under(output, id, &format!("{prefix}{extension}"), options);
    }

    /// Appends the lines below `id`'s own: its value lines then its
    /// child subtrees, the last of the combined list taking the
    /// last-child connector.
    fn render_under(
        &self,
        output: &mut String,
        id: U32Id<BTreeGridNode>,
        prefix: &str,
        options: &TreeGridHierarchyOptions,
    ) {
        let node = self.node(id);
        let values: &[C::Value] = if options.value_children {
            &node.values
        } else {
            &[]
        };
        let total = values.len() + node.children().len();

        for (index, value) in values.iter().enumerate() {
            let connector = if index + 1 == total {
                CONNECTOR_LAST
            } else {
                CONNECTOR_MID
            };
            let cell = Cell::render(self.cells(), node.format, value);
            output.push_str(&format!("{prefix}{connector} {}\n", cell.rendered));
        }

        for (index, &child) in node.children().iter().enumerate() {
            let last = values.len() + index + 1 == total;
            self.render_subtree(output, child, prefix, last, options);
        }
    }

    /// The node's line content: label, annotation, then the inline
    /// cells; the cells are omitted when values print as child lines
    /// instead.
    fn node_line(&self, id: U32Id<BTreeGridNode>, options: &TreeGridHierarchyOptions) -> String {
        let node = self.node(id);
        let mut line = node.label.render();
        if let Some(annotation) = &node.annotation {
            line.push(' ');
            line.push_str(annotation);
        }
        if !options.value_children && !node.values.is_empty() {
            let cells: Vec<Cell> = node
                .values
                .iter()
                .map(|value| Cell::render(self.cells(), node.format, value))
                .collect();
            let rendered: Vec<&str> = cells.iter().map(|cell| cell.rendered.as_str()).collect();
            line.push_str(": ");
            line.push_str(&rendered.join(Cell::separator(&cells)));
        }
        line
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGrid, TreeGridCellFormat, TreeGridHierarchyOptions, TreeGridLabel, TreeGridValue,
    };

    #[test]
    fn an_empty_grid_renders_the_empty_string() {
        assert_eq!(
            TreeGrid::new().render_hierarchy(&TreeGridHierarchyOptions::default()),
            ""
        );
    }

    #[test]
    fn a_bare_root_prints_alone_above_its_connectored_subtree() {
        let mut grid = TreeGrid::new();
        let palettes = grid.add_root(TreeGridLabel::bare("palettes"));
        let first = grid.add_child(palettes, TreeGridLabel::bare("0"));
        let count = grid.add_child(first, TreeGridLabel::bare("materialCount"));
        grid.push_value(count, TreeGridValue::int(2));
        let attributes = grid.add_child(first, TreeGridLabel::bare("attributes"));
        grid.add_child(attributes, TreeGridLabel::bare("baseColorFactor"));
        grid.add_child(attributes, TreeGridLabel::bare("metallicFactor"));
        let objects = grid.add_child(first, TreeGridLabel::bare("objects"));
        grid.add_child(objects, TreeGridLabel::bare("a"));
        grid.add_child(objects, TreeGridLabel::bare("b"));
        let second = grid.add_child(palettes, TreeGridLabel::bare("1"));
        let count = grid.add_child(second, TreeGridLabel::bare("materialCount"));
        grid.push_value(count, TreeGridValue::int(1));

        let options = TreeGridHierarchyOptions::default().with_bare_roots(true);
        assert_eq!(
            grid.render_hierarchy(&options),
            "palettes\n\
             ├ 0\n\
             │ ├ materialCount: 2\n\
             │ ├ attributes\n\
             │ │ ├ baseColorFactor\n\
             │ │ └ metallicFactor\n\
             │ └ objects\n\
             │   ├ a\n\
             │   └ b\n\
             └ 1\n\
             \u{20}\u{20}└ materialCount: 1\n"
        );
    }

    #[test]
    fn connectored_roots_render_as_annotated_siblings() {
        let mut grid = TreeGrid::new();
        let group = grid.add_root(TreeGridLabel::bare("energy-tank"));
        grid.node_mut(group).annotation = Some("(Group)".to_owned());
        let object = grid.add_child(group, TreeGridLabel::bare("energy-tank-1"));
        grid.node_mut(object).annotation = Some("(Object)".to_owned());
        let reactor = grid.add_root(TreeGridLabel::bare("energy-reactor"));
        grid.node_mut(reactor).annotation = Some("(Object)".to_owned());

        assert_eq!(
            grid.render_hierarchy(&TreeGridHierarchyOptions::default()),
            "├ energy-tank (Group)\n\
             │ └ energy-tank-1 (Object)\n\
             └ energy-reactor (Object)\n"
        );
    }

    #[test]
    fn a_marker_root_takes_a_connector_like_a_collapsed_ancestors_list() {
        let mut grid = TreeGrid::new();
        let ancestors = grid.add_root(TreeGridLabel::bare("ancestors"));
        let hand = grid.add_child(ancestors, TreeGridLabel::quoted("hand"));
        grid.push_value(hand, TreeGridValue::new("{node: 0}"));
        let mesh = grid.add_child(hand, TreeGridLabel::quoted("handMesh"));
        grid.push_value(mesh, TreeGridValue::new("{object: 0}"));

        assert_eq!(
            grid.render_hierarchy(&TreeGridHierarchyOptions::default()),
            "└ ancestors\n\
             \u{20}\u{20}└ \"hand\": {node: 0}\n\
             \u{20}\u{20}\u{20}\u{20}└ \"handMesh\": {object: 0}\n"
        );
    }

    #[test]
    fn bare_root_sections_separate_with_a_blank_line() {
        let mut grid = TreeGrid::new();
        let root = grid.add_root(TreeGridLabel::bare("root"));
        let node = grid.add_child(root, TreeGridLabel::quoted("root"));
        grid.push_value(node, TreeGridValue::new("{node: 0}"));
        let body = grid.add_child(node, TreeGridLabel::quoted("body"));
        grid.push_value(body, TreeGridValue::new("{object: 0}"));
        let unplaced = grid.add_root(TreeGridLabel::bare("unplaced"));
        let spare = grid.add_child(unplaced, TreeGridLabel::quoted("spareNode"));
        grid.push_value(spare, TreeGridValue::new("{node: 1}"));
        let spare_child = grid.add_child(spare, TreeGridLabel::quoted("spareChild"));
        grid.push_value(spare_child, TreeGridValue::new("{object: 2}"));
        let loose = grid.add_child(unplaced, TreeGridLabel::quoted("looseMesh"));
        grid.push_value(loose, TreeGridValue::new("{object: 1}"));

        let options = TreeGridHierarchyOptions::default().with_bare_roots(true);
        assert_eq!(
            grid.render_hierarchy(&options),
            "root\n\
             └ \"root\": {node: 0}\n\
             \u{20}\u{20}└ \"body\": {object: 0}\n\
             \n\
             unplaced\n\
             ├ \"spareNode\": {node: 1}\n\
             │ └ \"spareChild\": {object: 2}\n\
             └ \"looseMesh\": {object: 1}\n"
        );
    }

    #[test]
    fn a_branch_with_values_keeps_its_children_beneath() {
        let mut grid = TreeGrid::new();
        let root = grid.add_root(TreeGridLabel::bare("root"));
        let tank = grid.add_child(root, TreeGridLabel::quoted("energy-tank-1"));
        grid.push_value(tank, TreeGridValue::new("{node: 0}"));
        let transform = grid.add_child(tank, TreeGridLabel::bare("transform"));
        let position = grid.add_child(transform, TreeGridLabel::bare("position"));
        grid.push_value(position, TreeGridValue::new("[12.5, 0.5, 10.0]"));
        let scale = grid.add_child(transform, TreeGridLabel::bare("scale"));
        grid.push_value(scale, TreeGridValue::new("[1.0, 1.0, 1.0]"));
        let object = grid.add_child(tank, TreeGridLabel::quoted("energy-tank-1"));
        grid.push_value(object, TreeGridValue::new("{object: 0, instance: 0}"));

        let options = TreeGridHierarchyOptions::default().with_bare_roots(true);
        assert_eq!(
            grid.render_hierarchy(&options),
            "root\n\
             └ \"energy-tank-1\": {node: 0}\n\
             \u{20}\u{20}├ transform\n\
             \u{20}\u{20}│ ├ position: [12.5, 0.5, 10.0]\n\
             \u{20}\u{20}│ └ scale: [1.0, 1.0, 1.0]\n\
             \u{20}\u{20}└ \"energy-tank-1\": {object: 0, instance: 0}\n"
        );
    }

    #[test]
    fn value_children_print_one_cell_per_line_before_the_child_nodes() {
        let mut grid = TreeGrid::new();
        let palette = grid.add_root(TreeGridLabel::bare("0"));
        let attribute = grid.add_child(palette, TreeGridLabel::quoted("baseColorFactor"));
        grid.node_mut(attribute).format = Some(TreeGridCellFormat::Text);
        grid.push_value(attribute, TreeGridValue::srgba8([255, 0, 0, 255]));
        grid.push_value(attribute, TreeGridValue::srgba8([0, 255, 0, 128]));
        let component = grid.add_child(attribute, TreeGridLabel::bare("a"));
        grid.node_mut(component).format = Some(TreeGridCellFormat::Text);
        grid.push_value(component, TreeGridValue::unorm8(255));

        let options = TreeGridHierarchyOptions::default().with_value_children(true);
        assert_eq!(
            grid.render_hierarchy(&options),
            "└ 0\n\
             \u{20}\u{20}└ \"baseColorFactor\"\n\
             \u{20}\u{20}\u{20}\u{20}├ #FF0000FF\n\
             \u{20}\u{20}\u{20}\u{20}├ #00FF0080\n\
             \u{20}\u{20}\u{20}\u{20}└ a\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}└ 255\n"
        );
    }

    #[test]
    fn an_annotated_data_node_renders_label_annotation_then_cells() {
        let mut grid = TreeGrid::new();
        let node = grid.add_root(TreeGridLabel::bare("energy-tank"));
        grid.node_mut(node).annotation = Some("(Group)".to_owned());
        grid.push_value(node, TreeGridValue::int(3));

        assert_eq!(
            grid.render_hierarchy(&TreeGridHierarchyOptions::default()),
            "└ energy-tank (Group): 3\n"
        );
    }

    #[test]
    fn an_inline_visual_series_abuts_into_a_strip() {
        let mut grid = TreeGrid::new();
        let attribute = grid.add_root(TreeGridLabel::quoted("baseColorFactor"));
        grid.node_mut(attribute).format = Some(TreeGridCellFormat::Visual);
        grid.push_value(attribute, TreeGridValue::srgba8([255, 0, 0, 255]));
        grid.push_value(attribute, TreeGridValue::srgba8([0, 255, 0, 128]));

        assert_eq!(
            grid.render_hierarchy(&TreeGridHierarchyOptions::default()),
            "└ \"baseColorFactor\": \
             \x1b[48;2;255;0;0m  \x1b[0m\x1b[48;2;0;255;0m  \x1b[0m\n"
        );
    }
}
