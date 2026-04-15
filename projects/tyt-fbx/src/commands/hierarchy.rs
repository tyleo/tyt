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

    /// If set, append an axis-aligned bounding-box subtree for each object,
    /// aggregating the object's own mesh with all descendant meshes.
    /// Accepts up to three positional values:
    /// `[<space>] [<precision>] [<scale>]`. `space` is `local` (default) or
    /// `world`. `precision` is the decimal precision (default 2). `scale` is
    /// `no-scale` (default) or `scale`; when `scale`, the object's local
    /// scale is baked into local-space output (no effect in world space).
    #[arg(
        long = "show-bounds",
        value_names = ["space", "precision", "scale"],
        num_args = 0..=3,
    )]
    show_bounds: Option<Vec<String>>,

    /// If set, append an AABB-extents subtree (`max - min`) for each object,
    /// aggregating the object's own mesh with all descendant meshes.
    /// Accepts up to three positional values:
    /// `[<space>] [<precision>] [<scale>]`. Same semantics as `--show-bounds`.
    #[arg(
        long = "show-extents",
        value_names = ["space", "precision", "scale"],
        num_args = 0..=3,
    )]
    show_extents: Option<Vec<String>>,
}

impl Hierarchy {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Hierarchy {
            input_fbx,
            show_transforms,
            show_bounds,
            show_extents,
        } = self;

        let (show_xfm, xfm_prec, xfm_world, xfm_deg) = pack_transforms(show_transforms)?;
        let (show_bnd, bnd_prec, bnd_world, bnd_scale) = pack_bounds(show_bounds)?;
        let (show_ext, ext_prec, ext_world, ext_scale) = pack_bounds(show_extents)?;

        let args: Vec<&OsStr> = vec![
            input_fbx.as_ref(),
            show_xfm, xfm_prec.as_ref(), xfm_world, xfm_deg,
            show_bnd, bnd_prec.as_ref(), bnd_world, bnd_scale,
            show_ext, ext_prec.as_ref(), ext_world, ext_scale,
        ];

        dependencies.exec_temp_blender_script_with_stdout(&utilities::FBX_HIERARCHY_PY, args)?;

        Ok(())
    }
}

fn pack_transforms(
    values: Option<Vec<String>>,
) -> Result<(&'static OsStr, OsString, &'static OsStr, &'static OsStr)> {
    match values {
        Some(values) => {
            let (p, w, d) = parse_transform_args(values)?;
            Ok((OsStr::new("true"), p, w, d))
        }
        None => Ok((
            OsStr::new("false"),
            OsString::from("2"),
            OsStr::new("false"),
            OsStr::new("false"),
        )),
    }
}

fn pack_bounds(
    values: Option<Vec<String>>,
) -> Result<(&'static OsStr, OsString, &'static OsStr, &'static OsStr)> {
    match values {
        Some(values) => {
            let (p, w, s) = parse_bounds_args(values)?;
            Ok((OsStr::new("true"), p, w, s))
        }
        None => Ok((
            OsStr::new("false"),
            OsString::from("2"),
            OsStr::new("false"),
            OsStr::new("false"),
        )),
    }
}

fn parse_transform_args(
    values: Vec<String>,
) -> Result<(OsString, &'static OsStr, &'static OsStr)> {
    let space = values.first().map(String::as_str).unwrap_or("local");
    let is_world = parse_space(space)?;

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
    parse_precision(precision_str)?;

    Ok((OsString::from(precision_str), is_world, is_degrees))
}

fn parse_bounds_args(
    values: Vec<String>,
) -> Result<(OsString, &'static OsStr, &'static OsStr)> {
    let space = values.first().map(String::as_str).unwrap_or("local");
    let is_world = parse_space(space)?;

    let precision_str = values.get(1).map(String::as_str).unwrap_or("2");
    parse_precision(precision_str)?;

    let scale = values.get(2).map(String::as_str).unwrap_or("no-scale");
    let apply_scale: &'static OsStr = match scale {
        "no-scale" => OsStr::new("false"),
        "scale" => OsStr::new("true"),
        other => {
            return Err(IOError::other(format!(
                "scale must be 'no-scale' or 'scale', got '{other}'"
            ))
            .into());
        }
    };

    Ok((OsString::from(precision_str), is_world, apply_scale))
}

fn parse_space(space: &str) -> Result<&'static OsStr> {
    match space {
        "local" => Ok(OsStr::new("false")),
        "world" => Ok(OsStr::new("true")),
        other => Err(IOError::other(format!(
            "space must be 'local' or 'world', got '{other}'"
        ))
        .into()),
    }
}

fn parse_precision(s: &str) -> Result<()> {
    s.parse::<u32>()
        .map_err(|e| IOError::other(format!("precision must be a non-negative integer: {e}")))?;
    Ok(())
}
