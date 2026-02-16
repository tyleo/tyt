use crate::{Dependencies, Result, blender};
use clap::Parser;

/// Collapses all mesh objects in the input FBX into a single joined mesh.
/// Clears parenting while keeping world transforms, deletes now-unused empties,
/// joins all meshes, and renames the result to `output-mesh-name`.
#[derive(Clone, Debug, Parser)]
pub struct Reduce {
    /// The input FBX file.
    #[arg(value_name = "input-fbx")]
    input_fbx: String,

    /// The name for the output mesh object and datablock.
    #[arg(value_name = "output-mesh-name")]
    output_mesh_name: String,

    /// The output FBX file to write. If not provided, the input file will be
    /// overwritten.
    #[arg(value_name = "output-fbx")]
    output_fbx: Option<String>,
}

impl Reduce {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Reduce {
            input_fbx,
            output_mesh_name,
            output_fbx,
        } = self;

        let output_fbx = output_fbx.as_ref().unwrap_or(&input_fbx);

        let args = [
            input_fbx.as_ref(),
            output_fbx.as_ref(),
            output_mesh_name.as_str(),
        ];

        dependencies.exec_temp_blender_scripts_with_stdout(
            &blender::FBX_REDUCE_TO_SINGLE_MESH_PY,
            [&blender::COMMON_PY],
            args,
        )?;

        Ok(())
    }
}
