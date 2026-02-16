use crate::{Dependencies, Result, blender};
use clap::Parser;
use std::{ffi::OsStr, path::PathBuf};

/// Extracts the first direct child mesh under `parent_mesh_name` from the input
/// FBX file, unparents it, keeping the world transform, and deletes everything
/// else so the file only contains the extracted mesh. Finally, renames the mesh
/// object and its datablock to `output-mesh-name`.
#[derive(Clone, Debug, Parser)]
pub struct Extract {
    /// The input FBX file to extract from.
    #[arg(value_name = "input-fbx")]
    input_fbx: PathBuf,

    /// The name of the mesh's parent.
    #[arg(value_name = "parent-mesh-name")]
    parent_mesh_name: String,

    /// The output FBX file to write the extracted data to. If not provided,
    /// the input file will be overwritten.
    #[arg(value_name = "output-fbx")]
    output_fbx: Option<PathBuf>,

    /// The name of the output mesh to write. If not provided, the original
    /// mesh name will be used.
    #[arg(
        value_name = "output-mesh-name",
        short = 'o',
        long = "output-mesh-name",
        conflicts_with = "output_mesh_name_arg"
    )]
    output_mesh_name_flag: Option<String>,

    /// The name of the output mesh to write. If not provided, the original
    /// mesh name will be used.
    #[arg(
        value_name = "output-mesh-name",
        conflicts_with = "output_mesh_name_flag"
    )]
    output_mesh_name_arg: Option<String>,
}

impl Extract {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Extract {
            input_fbx,
            parent_mesh_name,
            output_fbx,
            output_mesh_name_flag,
            output_mesh_name_arg,
        } = self;

        let output_fbx = output_fbx.as_ref().unwrap_or(&input_fbx);

        let output_mesh_name = output_mesh_name_flag
            .as_ref()
            .or(output_mesh_name_arg.as_ref())
            .unwrap_or(&parent_mesh_name);

        let args: [&OsStr; 4] = [
            input_fbx.as_ref(),
            parent_mesh_name.as_ref(),
            output_fbx.as_ref(),
            output_mesh_name.as_ref(),
        ];

        dependencies.exec_temp_blender_scripts_with_stdout(
            &blender::FBX_EXTRACT_MESH_PY,
            [&blender::COMMON_PY],
            args,
        )?;

        Ok(())
    }
}
