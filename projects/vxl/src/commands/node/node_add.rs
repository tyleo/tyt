use crate::{Dependencies, ParentSelection, Result, VoxelInput, VoxjOutput, edit_document};
use clap::Parser;
use voxsmith::operations::node::add_node;

/// Adds an empty node with an identity transform under a parent node, or as
/// the last root.
#[derive(Clone, Debug, Parser)]
#[command(name = "add")]
pub struct NodeAdd {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    parent: ParentSelection,

    /// The new node's name.
    #[arg(value_name = "name", long)]
    name: String,
}

impl NodeAdd {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let parent_id = self.parent.resolve(main)?;

            add_node(main, self.name, parent_id)?;

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::NodeAdd;
    use clap::Parser;

    #[test]
    fn parses_the_name_without_a_selector() {
        let add = NodeAdd::try_parse_from(["add", "scene.voxj", "--name", "garage"]).unwrap();

        assert_eq!(add.name, "garage");
        assert!(NodeAdd::try_parse_from(["add", "scene.voxj"]).is_err());
        assert!(
            NodeAdd::try_parse_from(["add", "scene.voxj", "--name", "a", "--select", "b"]).is_err()
        );
    }
}
