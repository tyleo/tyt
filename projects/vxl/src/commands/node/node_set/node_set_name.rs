use crate::{Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document};
use clap::Parser;
use voxcore::BVoxHierarchyNode;
use voxsmith::Error as VoxsmithError;

/// Renames one node.
#[derive(Clone, Debug, Parser)]
#[command(name = "name")]
pub struct NodeSetName {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxHierarchyNode>,

    /// The node's new name.
    #[arg(value_name = "name", long)]
    name: String,
}

impl NodeSetName {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let node_id = self.selection.resolve_one_node(main)?;

            Ok(main
                .set_hierarchy_node_name(node_id, self.name)
                .map_err(VoxsmithError::from)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::NodeSetName;
    use clap::Parser;

    #[test]
    fn parses_the_name() {
        let set = NodeSetName::try_parse_from([
            "name",
            "scene.voxj",
            "--select",
            "house/door",
            "--name",
            "gate",
        ])
        .unwrap();

        assert_eq!(set.name, "gate");
    }
}
