use crate::{Dependencies, Result, utilities};
use clap::Parser;
use std::{ffi::OsStr, path::PathBuf};

/// Renames mesh objects and their datablocks in the input FBX file. If exactly
/// one mesh exists it is renamed to `output-mesh-name`; if multiple exist they
/// are renamed to `output-mesh-name`-001, -002, etc.
#[derive(Clone, Debug, Parser)]
pub struct Rename {
    /// The input FBX file.
    #[arg(value_name = "input-fbx")]
    input_fbx: PathBuf,

    /// The base name for the output mesh object(s) and datablock(s).
    #[arg(value_name = "output-mesh-name")]
    output_mesh_name: String,

    /// The output FBX file to write. If not provided, the input file will be
    /// overwritten.
    #[arg(value_name = "output-fbx")]
    output_fbx: Option<PathBuf>,
}

impl Rename {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Rename {
            input_fbx,
            output_mesh_name,
            output_fbx,
        } = self;

        let output_fbx = output_fbx.as_ref().unwrap_or(&input_fbx);

        let args: [&OsStr; 3] = [
            input_fbx.as_ref(),
            output_fbx.as_ref(),
            output_mesh_name.as_ref(),
        ];

        dependencies.exec_temp_blender_scripts_with_stdout(
            &utilities::FBX_RENAME_MESHES_PY,
            [&utilities::COMMON_PY],
            args,
        )?;

        Ok(())
    }
}
