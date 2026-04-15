use crate::{Dependencies, Error, Result, utilities};
use clap::Parser;
use std::{
    ffi::{OsStr, OsString},
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};

/// Prints the FBX object hierarchy as a tree with box-drawing glyphs,
/// showing each object's name and type.
#[derive(Clone, Debug, Parser)]
pub struct Hierarchy {
    /// The input FBX file to inspect.
    #[arg(value_name = "input-fbx")]
    input_fbx: PathBuf,

    /// Optional glob pattern to match object hierarchy paths against. When
    /// set, only matched objects and their ancestors are printed (or only
    /// matched objects when `--collapse-ancestors` is used). `**/` is
    /// auto-prepended when the pattern does not already start with it.
    #[arg(value_name = "pattern")]
    pattern: Option<String>,

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

    /// When set with a `pattern`, the ancestor chain above each matched
    /// object is hidden and replaced with an `(ANCESTORS)` marker printed
    /// directly above the matched object (omitted when the matched object
    /// is a scene root). Has no effect when `pattern` is omitted.
    #[arg(value_name = "collapse-ancestors", long = "collapse-ancestors")]
    collapse_ancestors: bool,

    /// When set with a `pattern`, the descendants of each matched object
    /// are hidden and replaced with a `(DESCENDANTS)` marker printed as a
    /// child subtree of the matched object (omitted when the matched
    /// object has no descendants). Has no effect when `pattern` is omitted.
    #[arg(value_name = "collapse-descendants", long = "collapse-descendants")]
    collapse_descendants: bool,
}

impl Hierarchy {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Hierarchy {
            input_fbx,
            pattern,
            show_transforms,
            show_bounds,
            show_extents,
            collapse_ancestors,
            collapse_descendants,
        } = self;

        let (show_xfm, xfm_prec, xfm_world, xfm_deg) = pack_transforms(show_transforms)?;
        let (show_bnd, bnd_prec, bnd_world, bnd_scale) = pack_bounds(show_bounds)?;
        let (show_ext, ext_prec, ext_world, ext_scale) = pack_bounds(show_extents)?;

        let matched_paths = match pattern {
            Some(pattern) => resolve_matched_paths(&dependencies, &input_fbx, &pattern)?,
            None => Vec::new(),
        };

        let collapse_anc_arg = bool_arg(collapse_ancestors);
        let collapse_desc_arg = bool_arg(collapse_descendants);
        let num_paths_arg = OsString::from(matched_paths.len().to_string());

        let mut args: Vec<&OsStr> = Vec::with_capacity(16 + matched_paths.len());
        args.extend_from_slice(&[
            input_fbx.as_ref(),
            show_xfm, xfm_prec.as_ref(), xfm_world, xfm_deg,
            show_bnd, bnd_prec.as_ref(), bnd_world, bnd_scale,
            show_ext, ext_prec.as_ref(), ext_world, ext_scale,
            collapse_anc_arg, collapse_desc_arg, num_paths_arg.as_ref(),
        ]);
        for p in &matched_paths {
            args.push(OsStr::new(p.as_str()));
        }

        dependencies.exec_temp_blender_script_with_stdout(&utilities::FBX_HIERARCHY_PY, args)?;

        Ok(())
    }
}

fn bool_arg(value: bool) -> &'static OsStr {
    if value {
        OsStr::new("true")
    } else {
        OsStr::new("false")
    }
}

fn resolve_matched_paths(
    dependencies: &impl Dependencies,
    input_fbx: &PathBuf,
    pattern: &str,
) -> Result<Vec<String>> {
    let args: [&OsStr; 1] = [input_fbx.as_ref()];
    let stdout = dependencies.exec_temp_blender_script(&utilities::FBX_HIERARCHY_JSON_PY, args)?;
    let json = utilities::extract_json(&stdout, b'[', b']')?;
    let entries = dependencies.parse_hierarchy_json(json)?;

    let pattern = if pattern.starts_with("**/") {
        pattern.to_owned()
    } else {
        format!("**/{pattern}")
    };

    let candidate_paths: Vec<&str> = entries.iter().map(|(_, path, _)| path.as_str()).collect();
    let matched = dependencies.match_glob(&pattern, &candidate_paths)?;

    let matched_paths: Vec<String> = entries
        .iter()
        .zip(matched.iter())
        .filter(|&(_, &m)| m)
        .map(|((_, path, _), _)| path.clone())
        .collect();

    if matched_paths.is_empty() {
        return Err(Error::IO(IOError::new(
            ErrorKind::NotFound,
            format!("no object matched pattern '{pattern}'"),
        )));
    }

    Ok(matched_paths)
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
