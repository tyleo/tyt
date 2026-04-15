use crate::{Dependencies, Result, utilities};
use clap::Parser;
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

/// Prints the FBX object hierarchy as a tree with box-drawing glyphs,
/// showing each object's name and type.
#[derive(Clone, Debug, Parser)]
pub struct Hierarchy {
    /// The input FBX file to inspect.
    #[arg(value_name = "input-fbx")]
    input_fbx: PathBuf,

    /// If set, prepend each object's transform (position, rotation, scale)
    /// as a nested subtree. The integer specifies decimal precision so that
    /// vector components align.
    #[arg(value_name = "precision", long = "show-transforms")]
    show_transforms: Option<u32>,
}

impl Hierarchy {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Hierarchy {
            input_fbx,
            show_transforms,
        } = self;

        let precision_arg: Option<OsString> = show_transforms.map(|p| OsString::from(p.to_string()));

        let mut args: Vec<&OsStr> = Vec::with_capacity(2);
        args.push(input_fbx.as_ref());
        if let Some(precision) = precision_arg.as_ref() {
            args.push(precision.as_ref());
        }

        dependencies.exec_temp_blender_script_with_stdout(&utilities::FBX_HIERARCHY_PY, args)?;

        Ok(())
    }
}
