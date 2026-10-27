use crate::{
    BTreeGridNode, TreeGrid, TreeGridCells, TreeGridMdListsLabelMode, TreeGridMdListsOptions,
    render::{self, Cell},
};
use branded_id::U32Id;

/// The `md-lists` render.
pub trait TreeGridRenderMdLists {
    /// Renders the `md-lists` layout: each data node's values as a
    /// numbered list, in pre-order, under the headings the label mode
    /// emits, with a blank line between blocks.
    fn render_md_lists(&self, options: &TreeGridMdListsOptions) -> String;
}

impl<C: TreeGridCells> TreeGridRenderMdLists for TreeGrid<C> {
    fn render_md_lists(&self, options: &TreeGridMdListsOptions) -> String {
        let mut blocks: Vec<String> = Vec::new();
        for &root in self.roots() {
            collect_list_blocks(self, root, 0, options, &mut blocks);
        }
        if blocks.is_empty() {
            String::new()
        } else {
            format!("{}\n", blocks.join("\n\n"))
        }
    }
}

/// The blocks of `id`'s subtree: a heading when the label mode
/// emits one and the node bears or leads to data, a list when the
/// node bears data, then each child's blocks.
fn collect_list_blocks<C: TreeGridCells>(
    grid: &TreeGrid<C>,
    id: U32Id<BTreeGridNode>,
    depth: usize,
    options: &TreeGridMdListsOptions,
    blocks: &mut Vec<String>,
) {
    let node = grid.node(id);
    let bears_data = !node.values.is_empty();
    if !bears_data && !grid.leads_to_data(id) {
        return;
    }
    if let TreeGridMdListsLabelMode::Header(header) = options.label {
        blocks.push(render::heading(
            header.level,
            depth,
            &node.annotated_label(),
        ));
    }
    if bears_data {
        blocks.push(list_block(grid, id));
    }
    for &child in node.children() {
        collect_list_blocks(grid, child, depth + 1, options, blocks);
    }
}

/// One node's list block: its values as `1.`-numbered lines.
fn list_block<C: TreeGridCells>(grid: &TreeGrid<C>, id: U32Id<BTreeGridNode>) -> String {
    let node = grid.node(id);
    node.values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let cell = Cell::render(grid.cells(), node.format, value);
            format!("{}. {}", index + 1, cell.rendered)
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGrid, TreeGridCellFormat, TreeGridHeaderOptions, TreeGridLabel,
        TreeGridMdListsLabelMode, TreeGridMdListsOptions, TreeGridRenderMdLists, TreeGridValue,
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

    #[test]
    fn an_empty_grid_renders_the_empty_string() {
        assert_eq!(
            TreeGrid::new().render_md_lists(&TreeGridMdListsOptions::default()),
            ""
        );
    }

    #[test]
    fn header_labels_head_every_node_on_the_way_to_data() {
        assert_eq!(
            worked_example().render_md_lists(&TreeGridMdListsOptions::default()),
            "# 0\n\
             \n\
             ## \"baseColorFactor\"\n\
             \n\
             1. #FF0000FF\n\
             2. #00FF0080\n\
             \n\
             ## \"metallicFactor\"\n\
             \n\
             1. 1\n\
             2. 0.2\n\
             \n\
             # 1\n\
             \n\
             ## \"baseColorFactor\"\n\
             \n\
             ### a\n\
             \n\
             1. 255\n"
        );
    }

    #[test]
    fn none_labels_leave_the_lists_alone() {
        let options = TreeGridMdListsOptions::default().with_label(TreeGridMdListsLabelMode::None);
        assert_eq!(
            worked_example().render_md_lists(&options),
            "1. #FF0000FF\n\
             2. #00FF0080\n\
             \n\
             1. 1\n\
             2. 0.2\n\
             \n\
             1. 255\n"
        );
    }

    #[test]
    fn root_data_heads_its_list_at_the_shallowest_level() {
        let mut grid = TreeGrid::new();
        let built_in = grid.retain_root(TreeGridLabel::bare("built in"));
        grid.push_value(built_in, TreeGridValue::new("albedo"));
        grid.push_value(built_in, TreeGridValue::new("orm"));
        let config = grid.retain_root(TreeGridLabel::bare("/repo/.vxlconfig"));
        grid.node_mut(config).annotation = Some("(local)".to_owned());
        grid.push_value(config, TreeGridValue::new("matte"));
        grid.retain_root(TreeGridLabel::bare("empty"));

        let options =
            TreeGridMdListsOptions::default().with_label(TreeGridMdListsLabelMode::Header(
                TreeGridHeaderOptions::default().with_level(NonZeroU8::new(2).unwrap()),
            ));
        assert_eq!(
            grid.render_md_lists(&options),
            "## built in\n\
             \n\
             1. albedo\n\
             2. orm\n\
             \n\
             ## /repo/.vxlconfig (local)\n\
             \n\
             1. matte\n"
        );
    }

    #[test]
    fn a_node_with_values_and_children_lists_before_its_subsections() {
        let mut grid = TreeGrid::new();
        let root = grid.retain_root(TreeGridLabel::bare("root"));
        grid.push_value(root, TreeGridValue::new("own"));
        let child = grid.retain_child(root, TreeGridLabel::bare("child"));
        grid.push_value(child, TreeGridValue::new("deep"));

        assert_eq!(
            grid.render_md_lists(&TreeGridMdListsOptions::default()),
            "# root\n\
             \n\
             1. own\n\
             \n\
             ## child\n\
             \n\
             1. deep\n"
        );
    }
}
