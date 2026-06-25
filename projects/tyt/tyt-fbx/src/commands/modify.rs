use crate::{Dependencies, Error, Result, utilities};
use clap::Parser;
use std::{
    ffi::{OsStr, OsString},
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};

/// Applies mutating operations to matched objects in an FBX.
#[derive(Clone, Debug, Parser)]
pub struct Modify {
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

    /// Removes all materials from matched mesh objects and deletes any
    /// material datablocks left with no users.
    #[arg(value_name = "clear-materials", long)]
    clear_materials: bool,
}

impl Modify {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Modify {
            input_fbx,
            pattern,
            output_fbx,
            clear_materials,
        } = self;

        let output_fbx = output_fbx.as_ref().unwrap_or(&input_fbx);

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

        let clear_materials_arg = if clear_materials { "true" } else { "false" };
        let num_objects_arg = OsString::from(matched_names.len().to_string());

        let mut args: Vec<&OsStr> = Vec::with_capacity(4 + matched_names.len());
        args.push(input_fbx.as_ref());
        args.push(output_fbx.as_ref());
        args.push(OsStr::new(clear_materials_arg));
        args.push(num_objects_arg.as_ref());
        for name in &matched_names {
            args.push(OsStr::new(*name));
        }

        dependencies.exec_temp_blender_scripts_with_stdout(
            &utilities::FBX_MODIFY_PY,
            [&utilities::COMMON_PY],
            args,
        )?;

        Ok(())
    }
}
