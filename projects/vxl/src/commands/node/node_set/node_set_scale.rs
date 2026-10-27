use crate::{
    Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document,
    parse_finite_f64, vector3_f64,
};
use clap::{ArgAction, Parser};
use voxcore::BVoxHierarchyNode;
use voxsmith::operations::node::set_node_scales;

/// Sets nodes' scales. A negative component mirrors that axis.
#[derive(Clone, Debug, Parser)]
#[command(name = "scale")]
pub struct NodeSetScale {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxHierarchyNode>,

    /// The node's new scale. No component may be zero.
    #[arg(
        value_names = ["x", "y", "z"],
        long,
        num_args = 3,
        required = true,
        allow_negative_numbers = true,
        action = ArgAction::Set,
        value_parser = parse_finite_f64
    )]
    scale: Vec<f64>,
}

impl NodeSetScale {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let scale = vector3_f64(&self.scale);

        edit_document(&dependencies, &self.input, self.output, |main| {
            let node_ids = self.selection.resolve_nodes(main)?;

            Ok(set_node_scales(main, &node_ids, scale)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::NodeSetScale;
    use clap::Parser;

    #[test]
    fn parses_a_mirroring_scale() {
        let set = NodeSetScale::try_parse_from([
            "scale",
            "scene.voxj",
            "--select-index",
            "0",
            "--scale",
            "-1",
            "1",
            "0.5",
        ])
        .unwrap();

        assert_eq!(set.scale, [-1.0, 1.0, 0.5]);
    }
}
