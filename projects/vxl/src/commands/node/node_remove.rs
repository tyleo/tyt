use crate::{Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document};
use clap::Parser;
use voxcore::BVoxHierarchyNode;
use voxsmith::operations::node::remove_nodes;

/// Removes nodes, then each descendant node and object left with no parent.
#[derive(Clone, Debug, Parser)]
#[command(name = "remove")]
pub struct NodeRemove {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxHierarchyNode>,
}

impl NodeRemove {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let node_ids = self.selection.resolve_nodes(main)?;

            Ok(remove_nodes(main, &node_ids)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::NodeRemove;
    use clap::Parser;

    #[test]
    fn parses_the_selectors() {
        assert!(
            NodeRemove::try_parse_from(["remove", "scene.voxj", "--select-index", "0-2"]).is_ok()
        );
        assert!(NodeRemove::try_parse_from(["remove", "scene.voxj"]).is_err());
    }
}
