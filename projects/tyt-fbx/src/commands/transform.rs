use crate::{Dependencies, Error, Result, utilities};
use clap::{Parser, ValueEnum};
use std::{
    ffi::{OsStr, OsString},
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum RotUnit {
    Radians,
    Degrees,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Space {
    Local,
    World,
}

/// Overwrites individual position, rotation, and scale components on every
/// object whose hierarchy path matches `pattern`. Unset components are left
/// untouched.
#[derive(Clone, Debug, Parser)]
pub struct Transform {
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

    /// Sets the x position of the matched objects.
    #[arg(value_name = "x", long = "set-pos-x")]
    set_pos_x: Option<f64>,

    /// Sets the y position of the matched objects.
    #[arg(value_name = "y", long = "set-pos-y")]
    set_pos_y: Option<f64>,

    /// Sets the z position of the matched objects.
    #[arg(value_name = "z", long = "set-pos-z")]
    set_pos_z: Option<f64>,

    /// Sets the rotation of the matched objects around the x-axis.
    #[arg(value_name = "x", long = "set-rot-x")]
    set_rot_x: Option<f64>,

    /// Sets the rotation of the matched objects around the y-axis.
    #[arg(value_name = "y", long = "set-rot-y")]
    set_rot_y: Option<f64>,

    /// Sets the rotation of the matched objects around the z-axis.
    #[arg(value_name = "z", long = "set-rot-z")]
    set_rot_z: Option<f64>,

    /// Sets the scale of the matched objects in the x-axis.
    #[arg(value_name = "x", long = "set-scl-x")]
    set_scl_x: Option<f64>,

    /// Sets the scale of the matched objects in the y-axis.
    #[arg(value_name = "y", long = "set-scl-y")]
    set_scl_y: Option<f64>,

    /// Sets the scale of the matched objects in the z-axis.
    #[arg(value_name = "z", long = "set-scl-z")]
    set_scl_z: Option<f64>,

    /// Adds to the x position of the matched objects.
    #[arg(value_name = "x", long = "mod-pos-x")]
    mod_pos_x: Option<f64>,

    /// Adds to the y position of the matched objects.
    #[arg(value_name = "y", long = "mod-pos-y")]
    mod_pos_y: Option<f64>,

    /// Adds to the z position of the matched objects.
    #[arg(value_name = "z", long = "mod-pos-z")]
    mod_pos_z: Option<f64>,

    /// Adds to the rotation of the matched objects around the x-axis.
    #[arg(value_name = "x", long = "mod-rot-x")]
    mod_rot_x: Option<f64>,

    /// Adds to the rotation of the matched objects around the y-axis.
    #[arg(value_name = "y", long = "mod-rot-y")]
    mod_rot_y: Option<f64>,

    /// Adds to the rotation of the matched objects around the z-axis.
    #[arg(value_name = "z", long = "mod-rot-z")]
    mod_rot_z: Option<f64>,

    /// Adds to the scale of the matched objects in the x-axis.
    #[arg(value_name = "x", long = "mod-scl-x")]
    mod_scl_x: Option<f64>,

    /// Adds to the scale of the matched objects in the y-axis.
    #[arg(value_name = "y", long = "mod-scl-y")]
    mod_scl_y: Option<f64>,

    /// Adds to the scale of the matched objects in the z-axis.
    #[arg(value_name = "z", long = "mod-scl-z")]
    mod_scl_z: Option<f64>,

    /// The unit used for the set-rot-* and mod-rot-* values.
    #[arg(value_name = "rot-unit", long = "rot-unit", default_value = "radians")]
    rot_unit: RotUnit,

    /// The transform space used for the set-* and mod-* values. When not set,
    /// Blender's default (local) is used.
    #[arg(value_name = "space", long = "space")]
    space: Option<Space>,
}

impl Transform {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Transform {
            input_fbx,
            pattern,
            output_fbx,
            set_pos_x,
            set_pos_y,
            set_pos_z,
            set_rot_x,
            set_rot_y,
            set_rot_z,
            set_scl_x,
            set_scl_y,
            set_scl_z,
            mod_pos_x,
            mod_pos_y,
            mod_pos_z,
            mod_rot_x,
            mod_rot_y,
            mod_rot_z,
            mod_scl_x,
            mod_scl_y,
            mod_scl_z,
            rot_unit,
            space,
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

        let to_radians = |v: f64| match rot_unit {
            RotUnit::Radians => v,
            RotUnit::Degrees => v.to_radians(),
        };

        let transform_slots = [
            set_pos_x,
            set_pos_y,
            set_pos_z,
            set_rot_x.map(to_radians),
            set_rot_y.map(to_radians),
            set_rot_z.map(to_radians),
            set_scl_x,
            set_scl_y,
            set_scl_z,
            mod_pos_x,
            mod_pos_y,
            mod_pos_z,
            mod_rot_x.map(to_radians),
            mod_rot_y.map(to_radians),
            mod_rot_z.map(to_radians),
            mod_scl_x,
            mod_scl_y,
            mod_scl_z,
        ];

        let transform_args: Vec<OsString> = transform_slots
            .iter()
            .map(|slot| match slot {
                Some(v) => OsString::from(v.to_string()),
                None => OsString::from("none"),
            })
            .collect();

        let space_arg = OsString::from(match space {
            Some(Space::Local) => "local",
            Some(Space::World) => "world",
            None => "default",
        });

        let num_objects_arg = OsString::from(matched_names.len().to_string());

        let mut args: Vec<&OsStr> = Vec::with_capacity(22 + matched_names.len());
        args.push(input_fbx.as_ref());
        args.push(output_fbx.as_ref());
        for transform_arg in &transform_args {
            args.push(transform_arg.as_ref());
        }
        args.push(space_arg.as_ref());
        args.push(num_objects_arg.as_ref());
        for name in &matched_names {
            args.push(OsStr::new(*name));
        }

        dependencies.exec_temp_blender_scripts_with_stdout(
            &utilities::FBX_TRANSFORM_PY,
            [&utilities::COMMON_PY],
            args,
        )?;

        Ok(())
    }
}
