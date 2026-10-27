use crate::{
    Dependencies, ParentSelection, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document,
};
use clap::Parser;
use voxcore::BVoxHierarchyNode;
use voxsmith::operations::node::link_nodes;

/// Places nodes under one more parent node, or in the roots. Errors when the
/// edge exists or would close a cycle.
#[derive(Clone, Debug, Parser)]
#[command(name = "link")]
pub struct NodeLink {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxHierarchyNode>,

    #[command(flatten)]
    parent: ParentSelection,
}

impl NodeLink {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let node_ids = self.selection.resolve_nodes(main)?;

            let parent_id = self.parent.resolve(main)?;

            Ok(link_nodes(main, &node_ids, parent_id)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::NodeLink;
    use clap::Parser;

    #[test]
    fn the_parent_is_optional() {
        assert!(
            NodeLink::try_parse_from([
                "link",
                "scene.voxj",
                "--select",
                "house/door",
                "--select-parent",
                "garage",
            ])
            .is_ok()
        );
        assert!(NodeLink::try_parse_from(["link", "scene.voxj", "--select", "shed"]).is_ok());
        assert!(NodeLink::try_parse_from(["link", "scene.voxj"]).is_err());
    }
}
