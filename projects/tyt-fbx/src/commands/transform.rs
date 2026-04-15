use crate::{Dependencies, Error, Result, utilities};
use clap::{ArgGroup, Parser, ValueEnum};
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
#[command(group(
    ArgGroup::new("set_args").multiple(true).conflicts_with("mod_args"),
))]
#[command(group(
    ArgGroup::new("mod_args").multiple(true),
))]
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

    /// The transform space used for the set-* and mod-* values. When not set,
    /// Blender's default (local) is used.
    #[arg(value_name = "space", long = "space")]
    space: Option<Space>,

    /// The unit used for the set-rot-* and mod-rot-* values.
    #[arg(value_name = "rot-unit", long = "rot-unit", default_value = "radians")]
    rot_unit: RotUnit,

    /// Sets the x position of the matched objects.
    #[arg(value_name = "x", long = "set-pos-x", group = "set_args")]
    set_pos_x: Option<f64>,

    /// Sets the y position of the matched objects.
    #[arg(value_name = "y", long = "set-pos-y", group = "set_args")]
    set_pos_y: Option<f64>,

    /// Sets the z position of the matched objects.
    #[arg(value_name = "z", long = "set-pos-z", group = "set_args")]
    set_pos_z: Option<f64>,

    /// Sets the rotation of the matched objects around the x-axis.
    #[arg(value_name = "x", long = "set-rot-x", group = "set_args")]
    set_rot_x: Option<f64>,

    /// Sets the rotation of the matched objects around the y-axis.
    #[arg(value_name = "y", long = "set-rot-y", group = "set_args")]
    set_rot_y: Option<f64>,

    /// Sets the rotation of the matched objects around the z-axis.
    #[arg(value_name = "z", long = "set-rot-z", group = "set_args")]
    set_rot_z: Option<f64>,

    /// Sets the scale of the matched objects in the x-axis.
    #[arg(value_name = "x", long = "set-scl-x", group = "set_args")]
    set_scl_x: Option<f64>,

    /// Sets the scale of the matched objects in the y-axis.
    #[arg(value_name = "y", long = "set-scl-y", group = "set_args")]
    set_scl_y: Option<f64>,

    /// Sets the scale of the matched objects in the z-axis.
    #[arg(value_name = "z", long = "set-scl-z", group = "set_args")]
    set_scl_z: Option<f64>,

    /// Sets all components of position on the matched objects.
    #[arg(
        value_names = ["x", "y", "z"],
        long = "set-pos",
        num_args = 3,
        group = "set_args",
        conflicts_with_all = ["set_pos_x", "set_pos_y", "set_pos_z", "set_xfm"],
    )]
    set_pos: Option<Vec<f64>>,

    /// Sets all components of rotation on the matched objects.
    #[arg(
        value_names = ["x", "y", "z"],
        long = "set-rot",
        num_args = 3,
        group = "set_args",
        conflicts_with_all = ["set_rot_x", "set_rot_y", "set_rot_z", "set_xfm"],
    )]
    set_rot: Option<Vec<f64>>,

    /// Sets all components of scale on the matched objects.
    #[arg(
        value_names = ["x", "y", "z"],
        long = "set-scl",
        num_args = 3,
        group = "set_args",
        conflicts_with_all = ["set_scl_x", "set_scl_y", "set_scl_z", "set_xfm"],
    )]
    set_scl: Option<Vec<f64>>,

    /// Sets the full transform of the matched objects. Takes 9 values in
    /// pos-x/y/z, rot-x/y/z, scl-x/y/z order.
    #[arg(
        value_names = ["px", "py", "pz", "rx", "ry", "rz", "sx", "sy", "sz"],
        long = "set-xfm",
        num_args = 9,
        group = "set_args",
        conflicts_with_all = [
            "set_pos", "set_pos_x", "set_pos_y", "set_pos_z",
            "set_rot", "set_rot_x", "set_rot_y", "set_rot_z",
            "set_scl", "set_scl_x", "set_scl_y", "set_scl_z",
        ],
    )]
    set_xfm: Option<Vec<f64>>,

    /// Adds to the x position of the matched objects.
    #[arg(value_name = "x", long = "mod-pos-x", group = "mod_args")]
    mod_pos_x: Option<f64>,

    /// Adds to the y position of the matched objects.
    #[arg(value_name = "y", long = "mod-pos-y", group = "mod_args")]
    mod_pos_y: Option<f64>,

    /// Adds to the z position of the matched objects.
    #[arg(value_name = "z", long = "mod-pos-z", group = "mod_args")]
    mod_pos_z: Option<f64>,

    /// Adds to the rotation of the matched objects around the x-axis.
    #[arg(value_name = "x", long = "mod-rot-x", group = "mod_args")]
    mod_rot_x: Option<f64>,

    /// Adds to the rotation of the matched objects around the y-axis.
    #[arg(value_name = "y", long = "mod-rot-y", group = "mod_args")]
    mod_rot_y: Option<f64>,

    /// Adds to the rotation of the matched objects around the z-axis.
    #[arg(value_name = "z", long = "mod-rot-z", group = "mod_args")]
    mod_rot_z: Option<f64>,

    /// Adds to the scale of the matched objects in the x-axis.
    #[arg(value_name = "x", long = "mod-scl-x", group = "mod_args")]
    mod_scl_x: Option<f64>,

    /// Adds to the scale of the matched objects in the y-axis.
    #[arg(value_name = "y", long = "mod-scl-y", group = "mod_args")]
    mod_scl_y: Option<f64>,

    /// Adds to the scale of the matched objects in the z-axis.
    #[arg(value_name = "z", long = "mod-scl-z", group = "mod_args")]
    mod_scl_z: Option<f64>,

    /// Adds to all components of position on the matched objects.
    #[arg(
        value_names = ["x", "y", "z"],
        long = "mod-pos",
        num_args = 3,
        group = "mod_args",
        conflicts_with_all = ["mod_pos_x", "mod_pos_y", "mod_pos_z", "mod_xfm"],
    )]
    mod_pos: Option<Vec<f64>>,

    /// Adds to all components of rotation on the matched objects.
    #[arg(
        value_names = ["x", "y", "z"],
        long = "mod-rot",
        num_args = 3,
        group = "mod_args",
        conflicts_with_all = ["mod_rot_x", "mod_rot_y", "mod_rot_z", "mod_xfm"],
    )]
    mod_rot: Option<Vec<f64>>,

    /// Adds to all components of scale on the matched objects.
    #[arg(
        value_names = ["x", "y", "z"],
        long = "mod-scl",
        num_args = 3,
        group = "mod_args",
        conflicts_with_all = ["mod_scl_x", "mod_scl_y", "mod_scl_z", "mod_xfm"],
    )]
    mod_scl: Option<Vec<f64>>,

    /// Adds to the full transform of the matched objects. Takes 9 values in
    /// pos-x/y/z, rot-x/y/z, scl-x/y/z order.
    #[arg(
        value_names = ["px", "py", "pz", "rx", "ry", "rz", "sx", "sy", "sz"],
        long = "mod-xfm",
        num_args = 9,
        group = "mod_args",
        conflicts_with_all = [
            "mod_pos", "mod_pos_x", "mod_pos_y", "mod_pos_z",
            "mod_rot", "mod_rot_x", "mod_rot_y", "mod_rot_z",
            "mod_scl", "mod_scl_x", "mod_scl_y", "mod_scl_z",
        ],
    )]
    mod_xfm: Option<Vec<f64>>,

    /// Bakes the x position of the matched objects: zeros pos x on the final
    /// transform and displaces mesh verts so the object appears unmoved.
    #[arg(value_name = "bak-pos-x", long = "bak-pos-x")]
    bak_pos_x: bool,

    /// Bakes the y position of the matched objects.
    #[arg(value_name = "bak-pos-y", long = "bak-pos-y")]
    bak_pos_y: bool,

    /// Bakes the z position of the matched objects.
    #[arg(value_name = "bak-pos-z", long = "bak-pos-z")]
    bak_pos_z: bool,

    /// Bakes the rotation of the matched objects around the x-axis.
    #[arg(value_name = "bak-rot-x", long = "bak-rot-x")]
    bak_rot_x: bool,

    /// Bakes the rotation of the matched objects around the y-axis.
    #[arg(value_name = "bak-rot-y", long = "bak-rot-y")]
    bak_rot_y: bool,

    /// Bakes the rotation of the matched objects around the z-axis.
    #[arg(value_name = "bak-rot-z", long = "bak-rot-z")]
    bak_rot_z: bool,

    /// Bakes the scale of the matched objects in the x-axis.
    #[arg(value_name = "bak-scl-x", long = "bak-scl-x")]
    bak_scl_x: bool,

    /// Bakes the scale of the matched objects in the y-axis.
    #[arg(value_name = "bak-scl-y", long = "bak-scl-y")]
    bak_scl_y: bool,

    /// Bakes the scale of the matched objects in the z-axis.
    #[arg(value_name = "bak-scl-z", long = "bak-scl-z")]
    bak_scl_z: bool,

    /// Bakes all components of position on the matched objects to 0,
    /// displacing mesh verts so the object appears unmoved.
    #[arg(
        value_name = "bak-pos",
        long = "bak-pos",
        conflicts_with_all = ["bak_pos_x", "bak_pos_y", "bak_pos_z", "bak_xfm"],
    )]
    bak_pos: bool,

    /// Bakes all components of rotation on the matched objects to 0.
    #[arg(
        value_name = "bak-rot",
        long = "bak-rot",
        conflicts_with_all = ["bak_rot_x", "bak_rot_y", "bak_rot_z", "bak_xfm"],
    )]
    bak_rot: bool,

    /// Bakes all components of scale on the matched objects to 1.
    #[arg(
        value_name = "bak-scl",
        long = "bak-scl",
        conflicts_with_all = ["bak_scl_x", "bak_scl_y", "bak_scl_z", "bak_xfm"],
    )]
    bak_scl: bool,

    /// Bakes the full transform of the matched objects to identity
    /// (pos=0, rot=0, scl=1).
    #[arg(
        value_name = "bak-xfm",
        long = "bak-xfm",
        conflicts_with_all = [
            "bak_pos", "bak_pos_x", "bak_pos_y", "bak_pos_z",
            "bak_rot", "bak_rot_x", "bak_rot_y", "bak_rot_z",
            "bak_scl", "bak_scl_x", "bak_scl_y", "bak_scl_z",
        ],
    )]
    bak_xfm: bool,
}

impl Transform {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Transform {
            input_fbx,
            pattern,
            output_fbx,
            space,
            rot_unit,
            set_pos_x,
            set_pos_y,
            set_pos_z,
            set_rot_x,
            set_rot_y,
            set_rot_z,
            set_scl_x,
            set_scl_y,
            set_scl_z,
            set_pos,
            set_rot,
            set_scl,
            set_xfm,
            mod_pos_x,
            mod_pos_y,
            mod_pos_z,
            mod_rot_x,
            mod_rot_y,
            mod_rot_z,
            mod_scl_x,
            mod_scl_y,
            mod_scl_z,
            mod_pos,
            mod_rot,
            mod_scl,
            mod_xfm,
            bak_pos_x,
            bak_pos_y,
            bak_pos_z,
            bak_rot_x,
            bak_rot_y,
            bak_rot_z,
            bak_scl_x,
            bak_scl_y,
            bak_scl_z,
            bak_pos,
            bak_rot,
            bak_scl,
            bak_xfm,
        } = self;

        let bake_all_pos = bak_pos || bak_xfm;
        let bake_all_rot = bak_rot || bak_xfm;
        let bake_all_scl = bak_scl || bak_xfm;

        let bak_pos_x = (bake_all_pos || bak_pos_x).then_some(0.0);
        let bak_pos_y = (bake_all_pos || bak_pos_y).then_some(0.0);
        let bak_pos_z = (bake_all_pos || bak_pos_z).then_some(0.0);
        let bak_rot_x = (bake_all_rot || bak_rot_x).then_some(0.0);
        let bak_rot_y = (bake_all_rot || bak_rot_y).then_some(0.0);
        let bak_rot_z = (bake_all_rot || bak_rot_z).then_some(0.0);
        let bak_scl_x = (bake_all_scl || bak_scl_x).then_some(1.0);
        let bak_scl_y = (bake_all_scl || bak_scl_y).then_some(1.0);
        let bak_scl_z = (bake_all_scl || bak_scl_z).then_some(1.0);

        // Expand set/mod aggregates into per-axis Options. Clap already
        // rejects combinations like `--set-pos X Y Z --set-pos-x W`.
        let expand_triple = |triple: Option<Vec<f64>>| -> [Option<f64>; 3] {
            match triple {
                Some(v) => [Some(v[0]), Some(v[1]), Some(v[2])],
                None => [None, None, None],
            }
        };

        let [sp_x, sp_y, sp_z] = expand_triple(set_pos);
        let [sr_x, sr_y, sr_z] = expand_triple(set_rot);
        let [ss_x, ss_y, ss_z] = expand_triple(set_scl);
        let [mp_x, mp_y, mp_z] = expand_triple(mod_pos);
        let [mr_x, mr_y, mr_z] = expand_triple(mod_rot);
        let [ms_x, ms_y, ms_z] = expand_triple(mod_scl);

        let (set_xfm_pos, set_xfm_rot, set_xfm_scl) = match set_xfm {
            Some(v) => (
                [Some(v[0]), Some(v[1]), Some(v[2])],
                [Some(v[3]), Some(v[4]), Some(v[5])],
                [Some(v[6]), Some(v[7]), Some(v[8])],
            ),
            None => ([None; 3], [None; 3], [None; 3]),
        };

        let (mod_xfm_pos, mod_xfm_rot, mod_xfm_scl) = match mod_xfm {
            Some(v) => (
                [Some(v[0]), Some(v[1]), Some(v[2])],
                [Some(v[3]), Some(v[4]), Some(v[5])],
                [Some(v[6]), Some(v[7]), Some(v[8])],
            ),
            None => ([None; 3], [None; 3], [None; 3]),
        };

        let set_pos_x = set_pos_x.or(sp_x).or(set_xfm_pos[0]);
        let set_pos_y = set_pos_y.or(sp_y).or(set_xfm_pos[1]);
        let set_pos_z = set_pos_z.or(sp_z).or(set_xfm_pos[2]);
        let set_rot_x = set_rot_x.or(sr_x).or(set_xfm_rot[0]);
        let set_rot_y = set_rot_y.or(sr_y).or(set_xfm_rot[1]);
        let set_rot_z = set_rot_z.or(sr_z).or(set_xfm_rot[2]);
        let set_scl_x = set_scl_x.or(ss_x).or(set_xfm_scl[0]);
        let set_scl_y = set_scl_y.or(ss_y).or(set_xfm_scl[1]);
        let set_scl_z = set_scl_z.or(ss_z).or(set_xfm_scl[2]);

        let mod_pos_x = mod_pos_x.or(mp_x).or(mod_xfm_pos[0]);
        let mod_pos_y = mod_pos_y.or(mp_y).or(mod_xfm_pos[1]);
        let mod_pos_z = mod_pos_z.or(mp_z).or(mod_xfm_pos[2]);
        let mod_rot_x = mod_rot_x.or(mr_x).or(mod_xfm_rot[0]);
        let mod_rot_y = mod_rot_y.or(mr_y).or(mod_xfm_rot[1]);
        let mod_rot_z = mod_rot_z.or(mr_z).or(mod_xfm_rot[2]);
        let mod_scl_x = mod_scl_x.or(ms_x).or(mod_xfm_scl[0]);
        let mod_scl_y = mod_scl_y.or(ms_y).or(mod_xfm_scl[1]);
        let mod_scl_z = mod_scl_z.or(ms_z).or(mod_xfm_scl[2]);

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
            bak_pos_x,
            bak_pos_y,
            bak_pos_z,
            bak_rot_x.map(to_radians),
            bak_rot_y.map(to_radians),
            bak_rot_z.map(to_radians),
            bak_scl_x,
            bak_scl_y,
            bak_scl_z,
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

        let mut args: Vec<&OsStr> = Vec::with_capacity(31 + matched_names.len());
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
