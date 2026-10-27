use crate::{
    Dependencies, ParentSelection, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document,
};
use clap::Parser;
use voxcore::BVoxHierarchyNode;
use voxsmith::operations::node::unlink_nodes;

/// Removes nodes from one parent node, or from the roots, leaving a node with no
/// parents unplaced. Errors when the edge does not exist.
#[derive(Clone, Debug, Parser)]
#[command(name = "unlink")]
pub struct NodeUnlink {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxHierarchyNode>,

    #[command(flatten)]
    parent: ParentSelection,
}

impl NodeUnlink {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let node_ids = self.selection.resolve_nodes(main)?;

            let parent_id = self.parent.resolve(main)?;

            Ok(unlink_nodes(main, &node_ids, parent_id)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::NodeUnlink;
    use clap::Parser;

    #[test]
    fn the_parent_is_optional() {
        assert!(
            NodeUnlink::try_parse_from([
                "unlink",
                "scene.voxj",
                "--select",
                "house/door",
                "--select-parent",
                "garage",
            ])
            .is_ok()
        );
        assert!(NodeUnlink::try_parse_from(["unlink", "scene.voxj", "--select", "shed"]).is_ok());
        assert!(NodeUnlink::try_parse_from(["unlink", "scene.voxj"]).is_err());
    }
}
