use crate::{Dependencies, Error, Result, utilities::RotUnit};
use clap::{ArgGroup, Args};
use std::{
    f64::consts::PI,
    ffi::OsString,
    io::{Error as IOError, ErrorKind},
    result::Result as StdResult,
    str,
};

/// Camera controls for the `render` command. Two mutually exclusive paths:
///
/// - **Auto-frame** (`--subject`, `--orbit-*`, `--zoom`, `--yaw` / `--pitch` /
///   `--roll`): computes a look-at pose from scene or subject bounds.
/// - **Explicit pose** (`--cam-pos-*`, `--cam-rot-*`, `--cam-xfm`): sets the
///   camera position and euler rotation directly.
///
/// `--rot-unit` controls the unit for every rotation-valued *input*.
/// `--print-camera` emits the resolved pose to stdout; it has its own
/// independent output unit.
#[derive(Clone, Debug, Args)]
#[command(group(
    ArgGroup::new("explicit_pose").multiple(true).conflicts_with("auto_frame"),
))]
#[command(group(
    ArgGroup::new("auto_frame").multiple(true),
))]
pub struct CameraArgs {
    /// Glob pattern selecting the object(s) the camera targets. The
    /// matched objects' combined world bounds drive auto-frame distance,
    /// look-at, and orbit pivot. `**/` is auto-prepended when the pattern
    /// does not already start with it.
    #[arg(value_name = "subject", long, group = "auto_frame")]
    pub subject: Option<String>,

    /// Horizontal orbit around the subject up axis. Unit from `--rot-unit`.
    #[arg(value_name = "orbit-h", long, group = "auto_frame")]
    pub orbit_h: Option<f64>,

    /// Vertical orbit (elevates/lowers the camera while still facing the
    /// subject). Unit from `--rot-unit`.
    #[arg(value_name = "orbit-v", long, group = "auto_frame")]
    pub orbit_v: Option<f64>,

    /// Distance multiplier applied to the auto-computed camera distance.
    /// `<1` closer, `>1` farther.
    #[arg(value_name = "zoom", long, group = "auto_frame")]
    pub zoom: Option<f64>,

    /// Camera-local rotation about the camera's up axis, applied after the
    /// look-at base rotation. Unit from `--rot-unit`.
    #[arg(value_name = "yaw", long, group = "auto_frame")]
    pub yaw: Option<f64>,

    /// Camera-local rotation about the camera's right axis, applied after
    /// the look-at base rotation. Unit from `--rot-unit`.
    #[arg(value_name = "pitch", long, group = "auto_frame")]
    pub pitch: Option<f64>,

    /// Camera-local rotation about the camera's forward axis, applied after
    /// the look-at base rotation. Unit from `--rot-unit`.
    #[arg(value_name = "roll", long, group = "auto_frame")]
    pub roll: Option<f64>,

    /// Sets the x position of the camera.
    #[arg(value_name = "x", long = "cam-pos-x", group = "explicit_pose")]
    pub cam_pos_x: Option<f64>,

    /// Sets the y position of the camera.
    #[arg(value_name = "y", long = "cam-pos-y", group = "explicit_pose")]
    pub cam_pos_y: Option<f64>,

    /// Sets the z position of the camera.
    #[arg(value_name = "z", long = "cam-pos-z", group = "explicit_pose")]
    pub cam_pos_z: Option<f64>,

    /// Sets the euler rotation of the camera around the x-axis. Unit from
    /// `--rot-unit`.
    #[arg(value_name = "x", long = "cam-rot-x", group = "explicit_pose")]
    pub cam_rot_x: Option<f64>,

    /// Sets the euler rotation of the camera around the y-axis. Unit from
    /// `--rot-unit`.
    #[arg(value_name = "y", long = "cam-rot-y", group = "explicit_pose")]
    pub cam_rot_y: Option<f64>,

    /// Sets the euler rotation of the camera around the z-axis. Unit from
    /// `--rot-unit`.
    #[arg(value_name = "z", long = "cam-rot-z", group = "explicit_pose")]
    pub cam_rot_z: Option<f64>,

    /// Sets all three components of the camera's position.
    #[arg(
        value_names = ["x", "y", "z"],
        long = "cam-pos",
        num_args = 3,
        group = "explicit_pose",
        conflicts_with_all = ["cam_pos_x", "cam_pos_y", "cam_pos_z", "cam_xfm"],
    )]
    pub cam_pos: Option<Vec<f64>>,

    /// Sets all three components of the camera's euler rotation. Unit from
    /// `--rot-unit`.
    #[arg(
        value_names = ["x", "y", "z"],
        long = "cam-rot",
        num_args = 3,
        group = "explicit_pose",
        conflicts_with_all = ["cam_rot_x", "cam_rot_y", "cam_rot_z", "cam_xfm"],
    )]
    pub cam_rot: Option<Vec<f64>>,

    /// Sets the camera's full pose (position + euler rotation). Takes 6
    /// values in pos-x/y/z, rot-x/y/z order. Rotation unit from
    /// `--rot-unit`.
    #[arg(
        value_names = ["px", "py", "pz", "rx", "ry", "rz"],
        long = "cam-xfm",
        num_args = 6,
        group = "explicit_pose",
        conflicts_with_all = [
            "cam_pos", "cam_pos_x", "cam_pos_y", "cam_pos_z",
            "cam_rot", "cam_rot_x", "cam_rot_y", "cam_rot_z",
        ],
    )]
    pub cam_xfm: Option<Vec<f64>>,

    /// Unit used for every rotation-valued input (`--orbit-h`,
    /// `--orbit-v`, `--yaw`, `--pitch`, `--roll`, `--cam-rot-*`,
    /// `--cam-rot`, and the rotation portion of `--cam-xfm`). Does NOT
    /// affect the `--print-camera` output.
    #[arg(value_name = "rot-unit", long = "rot-unit", default_value = "rad")]
    pub rot_unit: RotUnit,

    /// Emits the resolved camera pose on stdout. Accepts up to three
    /// positional values: `[<format>] [<unit>] [<precision>]`.
    /// `format` is one of `pose` (JSON), `pose-args` (paste-able
    /// `--cam-pos` / `--cam-rot` args), or `target-args` (paste-able
    /// `--orbit-*` / `--yaw` / ... args). Default adapts: `target-args`
    /// when auto-framed, `pose-args` when explicit pose.
    /// `unit` is `rad` (default) or `deg`.
    /// `precision` is the number of decimal digits in emitted numbers
    /// (default `2`).
    #[arg(
        value_names = ["format", "unit", "precision"],
        long = "print-camera",
        num_args = 0..=3,
    )]
    pub print_camera: Option<Vec<String>>,
}

impl CameraArgs {
    /// Returns true if any explicit-pose flag was set.
    pub fn has_explicit_pose(&self) -> bool {
        self.cam_pos_x.is_some()
            || self.cam_pos_y.is_some()
            || self.cam_pos_z.is_some()
            || self.cam_rot_x.is_some()
            || self.cam_rot_y.is_some()
            || self.cam_rot_z.is_some()
            || self.cam_pos.is_some()
            || self.cam_rot.is_some()
            || self.cam_xfm.is_some()
    }

    /// Validates cross-cutting rules clap can't express: `--print-camera`
    /// format + unit + precision are well-formed, and `target-args` is
    /// not explicitly paired with an explicit pose.
    pub fn validate(&self) -> Result<()> {
        let Some(tokens) = &self.print_camera else {
            return Ok(());
        };

        if let Some(format_str) = tokens.first() {
            match format_str.as_str() {
                "pose" | "pose-args" | "target-args" => {}
                other => {
                    return Err(Error::IO(IOError::new(
                        ErrorKind::InvalidInput,
                        format!(
                            "--print-camera format must be 'pose', 'pose-args', or 'target-args'; got '{other}'"
                        ),
                    )));
                }
            }
            if format_str == "target-args" && self.has_explicit_pose() {
                return Err(Error::IO(IOError::new(
                    ErrorKind::InvalidInput,
                    "--print-camera target-args cannot describe an explicit pose",
                )));
            }
        }

        if let Some(unit_str) = tokens.get(1)
            && unit_str != "rad"
            && unit_str != "deg"
        {
            return Err(Error::IO(IOError::new(
                ErrorKind::InvalidInput,
                format!("--print-camera unit must be 'rad' or 'deg'; got '{unit_str}'"),
            )));
        }

        if let Some(precision_str) = tokens.get(2) {
            precision_str.parse::<u32>().map_err(|e| {
                IOError::new(
                    ErrorKind::InvalidInput,
                    format!("--print-camera precision must be a non-negative integer: {e}"),
                )
            })?;
        }

        Ok(())
    }

    /// Collapses `--cam-pos-*` / `--cam-pos` / `--cam-rot-*` / `--cam-rot` /
    /// `--cam-xfm` into six `Option<f64>` slots (pos x/y/z, rot x/y/z). Returns
    /// `None` when no explicit-pose flag was set. Rotation values remain in
    /// `self.rot_unit`.
    fn explicit_pose_slots(&self) -> Option<[Option<f64>; 6]> {
        if !self.has_explicit_pose() {
            return None;
        }

        let mut slots = [
            self.cam_pos_x,
            self.cam_pos_y,
            self.cam_pos_z,
            self.cam_rot_x,
            self.cam_rot_y,
            self.cam_rot_z,
        ];

        if let Some(v) = &self.cam_pos {
            slots[0] = Some(v[0]);
            slots[1] = Some(v[1]);
            slots[2] = Some(v[2]);
        }

        if let Some(v) = &self.cam_rot {
            slots[3] = Some(v[0]);
            slots[4] = Some(v[1]);
            slots[5] = Some(v[2]);
        }

        if let Some(v) = &self.cam_xfm {
            slots[0] = Some(v[0]);
            slots[1] = Some(v[1]);
            slots[2] = Some(v[2]);
            slots[3] = Some(v[3]);
            slots[4] = Some(v[4]);
            slots[5] = Some(v[5]);
        }

        Some(slots)
    }

    /// Serializes all modifier args for the Blender script. Rotations are
    /// converted from `self.rot_unit` to radians. `subject_names` is
    /// appended as a count + variable-length tail.
    pub fn to_python_args(&self, subject_names: &[String]) -> Vec<OsString> {
        let mut args: Vec<OsString> = Vec::new();

        let slots = match self.explicit_pose_slots() {
            Some([px, py, pz, rx, ry, rz]) => [
                px,
                py,
                pz,
                rx.map(|v| self.rot_unit.to_radians(v)),
                ry.map(|v| self.rot_unit.to_radians(v)),
                rz.map(|v| self.rot_unit.to_radians(v)),
            ],
            None => [None; 6],
        };
        for slot in slots {
            args.push(match slot {
                Some(v) => OsString::from(v.to_string()),
                None => OsString::from("none"),
            });
        }

        let orbit_h_rad = self.rot_unit.to_radians(self.orbit_h.unwrap_or(0.0));
        let orbit_v_rad = self.rot_unit.to_radians(self.orbit_v.unwrap_or(0.0));
        let zoom = self.zoom.unwrap_or(1.0);
        args.push(orbit_h_rad.to_string().into());
        args.push(orbit_v_rad.to_string().into());
        args.push(zoom.to_string().into());

        let yaw_rad = self.rot_unit.to_radians(self.yaw.unwrap_or(0.0));
        let pitch_rad = self.rot_unit.to_radians(self.pitch.unwrap_or(0.0));
        let roll_rad = self.rot_unit.to_radians(self.roll.unwrap_or(0.0));
        args.push(yaw_rad.to_string().into());
        args.push(pitch_rad.to_string().into());
        args.push(roll_rad.to_string().into());

        let emit_camera = if self.print_camera.is_some() {
            "true"
        } else {
            "false"
        };
        args.push(emit_camera.into());

        args.push(subject_names.len().to_string().into());
        for name in subject_names {
            args.push(OsString::from(name));
        }

        args
    }

    /// Parses the `===CAMERA===` / `===/CAMERA===` block emitted by the
    /// Blender script and writes the formatted output to stdout. No-op when
    /// `--print-camera` wasn't set.
    pub fn emit_print_camera(
        &self,
        dependencies: &impl Dependencies,
        blender_stdout: &[u8],
    ) -> Result<()> {
        let Some(tokens) = &self.print_camera else {
            return Ok(());
        };

        let (position, rotation_rad) = parse_camera_block(blender_stdout)?;

        let requested_format = tokens.first().map(String::as_str);
        let unit_str = tokens.get(1).map(String::as_str).unwrap_or("rad");
        let precision: usize = tokens
            .get(2)
            .map(String::as_str)
            .unwrap_or("2")
            .parse()
            .map_err(|e| {
                IOError::new(
                    ErrorKind::InvalidInput,
                    format!("--print-camera precision must be a non-negative integer: {e}"),
                )
            })?;
        let rad_to_out = if unit_str == "deg" { 180.0 / PI } else { 1.0 };
        let format = match requested_format {
            Some(f) => f,
            None => {
                if self.has_explicit_pose() {
                    "pose-args"
                } else {
                    "target-args"
                }
            }
        };

        let rot_out = [
            rotation_rad[0] * rad_to_out,
            rotation_rad[1] * rad_to_out,
            rotation_rad[2] * rad_to_out,
        ];

        let output = match format {
            "pose" => format!(
                "{{\"Position\": {{\"X\": {:.*}, \"Y\": {:.*}, \"Z\": {:.*}}}, \"Rotation\": {{\"X\": {:.*}, \"Y\": {:.*}, \"Z\": {:.*}}}}}\n",
                precision,
                position[0],
                precision,
                position[1],
                precision,
                position[2],
                precision,
                rot_out[0],
                precision,
                rot_out[1],
                precision,
                rot_out[2],
            ),
            "pose-args" => format!(
                "--cam-pos {:.prec$} {:.prec$} {:.prec$} --cam-rot {:.prec$} {:.prec$} {:.prec$} --rot-unit {unit}\n",
                position[0],
                position[1],
                position[2],
                rot_out[0],
                rot_out[1],
                rot_out[2],
                unit = unit_str,
                prec = precision,
            ),
            "target-args" => format_target_args(self, unit_str, precision),
            _ => unreachable!("format validated earlier"),
        };

        dependencies.write_stdout(output.as_bytes())?;
        Ok(())
    }
}

fn format_target_args(args: &CameraArgs, unit_str: &str, precision: usize) -> String {
    let rad_to_out = if unit_str == "deg" { 180.0 / PI } else { 1.0 };
    let angle_out = |value: f64| args.rot_unit.to_radians(value) * rad_to_out;

    let mut parts: Vec<String> = Vec::new();
    if let Some(subject) = &args.subject {
        parts.push(format!("--subject '{subject}'"));
    }
    if let Some(orbit_h) = args.orbit_h {
        parts.push(format!("--orbit-h {:.*}", precision, angle_out(orbit_h)));
    }
    if let Some(orbit_v) = args.orbit_v {
        parts.push(format!("--orbit-v {:.*}", precision, angle_out(orbit_v)));
    }
    if let Some(zoom) = args.zoom {
        parts.push(format!("--zoom {:.*}", precision, zoom));
    }
    if let Some(yaw) = args.yaw {
        parts.push(format!("--yaw {:.*}", precision, angle_out(yaw)));
    }
    if let Some(pitch) = args.pitch {
        parts.push(format!("--pitch {:.*}", precision, angle_out(pitch)));
    }
    if let Some(roll) = args.roll {
        parts.push(format!("--roll {:.*}", precision, angle_out(roll)));
    }
    parts.push(format!("--rot-unit {unit_str}"));
    format!("{}\n", parts.join(" "))
}

fn parse_camera_block(bytes: &[u8]) -> Result<([f64; 3], [f64; 3])> {
    const OPEN: &[u8] = b"===CAMERA===";
    const CLOSE: &[u8] = b"===/CAMERA===";

    let start = find_subslice(bytes, OPEN)
        .ok_or_else(|| IOError::new(ErrorKind::InvalidData, "camera block marker not found"))?;
    let body_start = start + OPEN.len();
    let close_rel = find_subslice(&bytes[body_start..], CLOSE).ok_or_else(|| {
        IOError::new(
            ErrorKind::InvalidData,
            "camera block close marker not found",
        )
    })?;
    let body = &bytes[body_start..body_start + close_rel];

    let text = str::from_utf8(body).map_err(|e| {
        IOError::new(
            ErrorKind::InvalidData,
            format!("camera block not valid utf-8: {e}"),
        )
    })?;

    let mut position: Option<[f64; 3]> = None;
    let mut rotation: Option<[f64; 3]> = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("POSITION") {
            position = Some(parse_three_floats(rest.trim())?);
        } else if let Some(rest) = line.strip_prefix("ROTATION") {
            rotation = Some(parse_three_floats(rest.trim())?);
        }
    }

    let position = position.ok_or_else(|| {
        IOError::new(ErrorKind::InvalidData, "camera block missing POSITION line")
    })?;
    let rotation = rotation.ok_or_else(|| {
        IOError::new(ErrorKind::InvalidData, "camera block missing ROTATION line")
    })?;
    Ok((position, rotation))
}

fn parse_three_floats(s: &str) -> Result<[f64; 3]> {
    let values: Vec<f64> = s
        .split_whitespace()
        .map(|t| t.parse::<f64>())
        .collect::<StdResult<Vec<_>, _>>()
        .map_err(|e| {
            IOError::new(
                ErrorKind::InvalidData,
                format!("camera block: invalid float: {e}"),
            )
        })?;
    if values.len() != 3 {
        return Err(Error::IO(IOError::new(
            ErrorKind::InvalidData,
            format!("camera block: expected 3 floats, got {}", values.len()),
        )));
    }
    Ok([values[0], values[1], values[2]])
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    (0..=haystack.len() - needle.len()).find(|&i| &haystack[i..i + needle.len()] == needle)
}
