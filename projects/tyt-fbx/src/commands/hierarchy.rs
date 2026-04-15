use crate::{Dependencies, Result, utilities};
use clap::Parser;
use std::{
    ffi::{OsStr, OsString},
    io::Error as IOError,
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
    /// as a nested subtree. Accepts up to three positional values:
    /// `[<space>] [<rot-unit>] [<precision>]`. `space` is `local` (default)
    /// or `world`. `rot-unit` is `rad` (default) or `deg`.
    /// `precision` is the decimal precision used to align vector components
    /// (default 2).
    #[arg(
        long = "show-transforms",
        value_names = ["space", "rot-unit", "precision"],
        num_args = 0..=3,
    )]
    show_transforms: Option<Vec<String>>,
}

impl Hierarchy {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Hierarchy {
            input_fbx,
            show_transforms,
        } = self;

        let transform_args: Option<(OsString, &'static OsStr, &'static OsStr)> = show_transforms
            .map(parse_transform_args)
            .transpose()?;

        let mut args: Vec<&OsStr> = Vec::with_capacity(4);
        args.push(input_fbx.as_ref());
        if let Some((precision, is_world, is_degrees)) = transform_args.as_ref() {
            args.push(precision.as_ref());
            args.push(is_world);
            args.push(is_degrees);
        }

        dependencies.exec_temp_blender_script_with_stdout(&utilities::FBX_HIERARCHY_PY, args)?;

        Ok(())
    }
}

fn parse_transform_args(
    values: Vec<String>,
) -> Result<(OsString, &'static OsStr, &'static OsStr)> {
    let space = values.first().map(String::as_str).unwrap_or("local");
    let is_world: &'static OsStr = match space {
        "local" => OsStr::new("false"),
        "world" => OsStr::new("true"),
        other => {
            return Err(IOError::other(format!(
                "space must be 'local' or 'world', got '{other}'"
            ))
            .into());
        }
    };

    let rot_unit = values.get(1).map(String::as_str).unwrap_or("rad");
    let is_degrees: &'static OsStr = match rot_unit {
        "rad" => OsStr::new("false"),
        "deg" => OsStr::new("true"),
        other => {
            return Err(IOError::other(format!(
                "rot-unit must be 'rad' or 'deg', got '{other}'"
            ))
            .into());
        }
    };

    let precision_str = values.get(2).map(String::as_str).unwrap_or("2");
    precision_str
        .parse::<u32>()
        .map_err(|e| IOError::other(format!("precision must be a non-negative integer: {e}")))?;

    Ok((OsString::from(precision_str), is_world, is_degrees))
}
