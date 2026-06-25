use crate::{Dependencies, Error, Result, utilities};
use clap::Parser;
use std::{
    ffi::{OsStr, OsString},
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};

/// Renames every object whose hierarchy path matches `pattern`. The new name
/// for each matched object is composed as
/// `{prefix}{name-or-old}{suffix}{suffix-num}`, where any omitted piece is
/// treated as empty and `name-or-old` is `--name` when set, otherwise the
/// object's existing name. Matches any object type (MESH, ARMATURE, EMPTY,
/// LIGHT, CAMERA, ...).
#[derive(Clone, Debug, Parser)]
pub struct Rename {
    /// The input FBX file.
    #[arg(value_name = "input-fbx")]
    input_fbx: PathBuf,

    /// Glob pattern to match object hierarchy paths against.
    #[arg(value_name = "pattern")]
    pattern: String,

    /// The output FBX file to write. If not provided, the input file will be
    /// overwritten.
    #[arg(value_name = "output-fbx")]
    output_fbx: Option<PathBuf>,

    /// Replaces the existing object name with this value as the `base` in the
    /// composed new name. If omitted, each object's current name is used.
    #[arg(value_name = "name", long = "name")]
    name: Option<String>,

    /// Prepended to the composed new name.
    #[arg(value_name = "prefix", long = "prefix")]
    prefix: Option<String>,

    /// Appended to the composed new name before the numeric suffix.
    #[arg(value_name = "suffix", long = "suffix")]
    suffix: Option<String>,

    /// Appends a numeric suffix to each matched object. Takes 0-2 values:
    /// `[start] [pad]`. `start` (u32, default 0) is the first number used.
    /// `pad` (bool, default false) zero-pads every number to the width of
    /// the largest number produced when true. Digits only - no implicit
    /// separator; use `--suffix "-"` for `name-0, name-1, ...`.
    #[arg(
        value_names = ["start", "pad"],
        long = "suffix-num",
        num_args = 0..=2,
    )]
    suffix_num: Option<Vec<String>>,
}

impl Rename {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Rename {
            input_fbx,
            pattern,
            output_fbx,
            name,
            prefix,
            suffix,
            suffix_num,
        } = self;

        let output_fbx = output_fbx.as_ref().unwrap_or(&input_fbx);

        let suffix_num = match suffix_num {
            None => None,
            Some(v) => {
                let start = match v.first() {
                    Some(s) => s.parse::<u32>().map_err(|e| {
                        Error::IO(IOError::new(
                            ErrorKind::InvalidInput,
                            format!("invalid --suffix-num start '{s}': {e}"),
                        ))
                    })?,
                    None => 0,
                };
                let pad = match v.get(1) {
                    Some(p) => p.parse::<bool>().map_err(|e| {
                        Error::IO(IOError::new(
                            ErrorKind::InvalidInput,
                            format!("invalid --suffix-num pad '{p}': {e}"),
                        ))
                    })?,
                    None => false,
                };
                Some((start, pad))
            }
        };

        // Phase 1: get hierarchy JSON from Blender.
        let args: [&OsStr; 1] = [input_fbx.as_ref()];
        let stdout =
            dependencies.exec_temp_blender_script(&utilities::FBX_HIERARCHY_JSON_PY, args)?;

        let json = utilities::extract_json(&stdout, b'[', b']')?;
        let entries = dependencies.parse_hierarchy_json(json)?;

        // Auto-prepend `**/` unless already present.
        let pattern = if pattern.starts_with("**/") {
            pattern
        } else {
            format!("**/{pattern}")
        };

        let candidate_paths: Vec<&str> = entries.iter().map(|(_, path, _)| path.as_str()).collect();
        let matched = dependencies.match_glob(&pattern, &candidate_paths)?;

        let matched_names: Vec<&str> = entries
            .iter()
            .zip(matched.iter())
            .filter(|&(_, &m)| m)
            .map(|((name, _, _), _)| name.as_str())
            .collect();

        if matched_names.is_empty() {
            return Err(Error::IO(IOError::new(
                ErrorKind::NotFound,
                format!("no object matched pattern '{pattern}'"),
            )));
        }

        let width = match suffix_num {
            Some((start, true)) => (start + matched_names.len() as u32 - 1).to_string().len(),
            _ => 0,
        };

        let prefix_str = prefix.as_deref().unwrap_or("");
        let suffix_str = suffix.as_deref().unwrap_or("");

        let new_names: Vec<String> = matched_names
            .iter()
            .enumerate()
            .map(|(i, old)| {
                let base = name.as_deref().unwrap_or(old);
                let num_str = match suffix_num {
                    Some((start, _)) => format!("{:0>width$}", start + i as u32, width = width),
                    None => String::new(),
                };
                format!("{prefix_str}{base}{suffix_str}{num_str}")
            })
            .collect();

        let num_pairs_arg = OsString::from(matched_names.len().to_string());

        let mut args: Vec<&OsStr> = Vec::with_capacity(3 + 2 * matched_names.len());
        args.push(input_fbx.as_ref());
        args.push(output_fbx.as_ref());
        args.push(num_pairs_arg.as_ref());
        for (old, new) in matched_names.iter().zip(new_names.iter()) {
            args.push(OsStr::new(*old));
            args.push(OsStr::new(new.as_str()));
        }

        dependencies.exec_temp_blender_scripts_with_stdout(
            &utilities::FBX_RENAME_OBJECTS_PY,
            [&utilities::COMMON_PY],
            args,
        )?;

        Ok(())
    }
}
