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

    /// If set, prepend each object's local-space transform (position,
    /// rotation, scale) as a nested subtree. The integer specifies decimal
    /// precision so that vector components align.
    #[arg(
        value_name = "precision",
        long = "show-local-transforms",
        conflicts_with = "show_world_transforms"
    )]
    show_local_transforms: Option<u32>,

    /// If set, prepend each object's world-space transform (position,
    /// rotation, scale) as a nested subtree. The integer specifies decimal
    /// precision so that vector components align.
    #[arg(value_name = "precision", long = "show-world-transforms")]
    show_world_transforms: Option<u32>,
}

impl Hierarchy {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Hierarchy {
            input_fbx,
            show_local_transforms,
            show_world_transforms,
        } = self;

        let transform_args: Option<(OsString, &'static OsStr)> = show_local_transforms
            .map(|p| (OsString::from(p.to_string()), OsStr::new("false")))
            .or_else(|| {
                show_world_transforms.map(|p| (OsString::from(p.to_string()), OsStr::new("true")))
            });

        let mut args: Vec<&OsStr> = Vec::with_capacity(3);
        args.push(input_fbx.as_ref());
        if let Some((precision, is_world)) = transform_args.as_ref() {
            args.push(precision.as_ref());
            args.push(is_world);
        }

        dependencies.exec_temp_blender_script_with_stdout(&utilities::FBX_HIERARCHY_PY, args)?;

        Ok(())
    }
}
