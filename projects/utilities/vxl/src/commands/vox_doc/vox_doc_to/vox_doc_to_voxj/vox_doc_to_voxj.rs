use crate::{
    Dependencies, ObjectSelection, PositiveF64, Result, VoxelInput, VoxjOutput, commands::convert,
};
use clap::Parser;

/// Converts a voxel file to the Voxel Json format.
#[derive(Clone, Debug, Parser)]
#[command(name = "voxj")]
pub struct VoxDocToVoxj {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    /// Multiplies each root node's position and scale by this factor.
    /// Defaults to `1.0`. Pass the inverse of a `vox-doc to vmax
    /// --scene-scale` factor to read that file back in meters.
    #[arg(value_name = "scene-scale", long)]
    scene_scale: Option<PositiveF64>,

    #[command(flatten)]
    selection: ObjectSelection,
}

impl VoxDocToVoxj {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let from = self.input.resolve_format()?;

        let (to, output) = self.output.resolve(&self.input.path);

        convert(
            &dependencies,
            &self.input.path,
            from,
            &output,
            &to,
            &self.selection,
            self.scene_scale.map(|scale| scale.0),
        )
    }
}
