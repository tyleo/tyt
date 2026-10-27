use crate::{
    Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document,
    parse_finite_f64, vector3_f64,
};
use clap::{ArgAction, Parser};
use voxcore::BVoxHierarchyNode;
use voxsmith::operations::node::set_node_positions;

/// Sets nodes' positions relative to their parents.
#[derive(Clone, Debug, Parser)]
#[command(name = "position")]
pub struct NodeSetPosition {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxHierarchyNode>,

    /// The node's new position.
    #[arg(
        value_names = ["x", "y", "z"],
        long,
        num_args = 3,
        required = true,
        allow_negative_numbers = true,
        action = ArgAction::Set,
        value_parser = parse_finite_f64
    )]
    position: Vec<f64>,
}

impl NodeSetPosition {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let position = vector3_f64(&self.position);

        edit_document(&dependencies, &self.input, self.output, |main| {
            let node_ids = self.selection.resolve_nodes(main)?;

            Ok(set_node_positions(main, &node_ids, position)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::NodeSetPosition;
    use clap::{Error as ClapError, Parser};

    fn parse(position: [&str; 3]) -> Result<NodeSetPosition, ClapError> {
        let mut args = vec!["position", "scene.voxj", "--select", "door", "--position"];
        args.extend(position);

        NodeSetPosition::try_parse_from(args)
    }

    #[test]
    fn parses_a_negative_position() {
        assert_eq!(
            parse(["-1.5", "0", "2"]).unwrap().position,
            [-1.5, 0.0, 2.0]
        );
        assert!(parse(["nan", "0", "0"]).is_err());
    }
}
