use crate::{Dependencies, Result, blender};
use clap::Parser;
use std::{ffi::OsStr, path::PathBuf};

/// Prints the FBX object hierarchy as a tree with box-drawing glyphs,
/// showing each object's name and type.
#[derive(Clone, Debug, Parser)]
pub struct Hierarchy {
    /// The input FBX file to inspect.
    #[arg(value_name = "input-fbx")]
    input_fbx: PathBuf,
}

impl Hierarchy {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let args: [&OsStr; 1] = [self.input_fbx.as_ref()];

        dependencies.exec_temp_blender_script_with_stdout(&blender::FBX_HIERARCHY_PY, args)?;

        Ok(())
    }
}
