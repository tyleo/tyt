use crate::{Dependencies, Error, Result, utilities};
use clap::Parser;
use std::{
    ffi::{OsStr, OsString},
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};

/// Renders the meshes in an FBX file from a specified camera position with
/// default 3-point lighting. The result is written to an image file, displayed
/// inline in the terminal (Kitty / iTerm2 / Sixel / ANSI fallback), or both.
#[derive(Clone, Debug, Parser)]
pub struct Render {
    /// The input FBX file to render.
    #[arg(value_name = "input-fbx")]
    input_fbx: PathBuf,

    /// Path to write the rendered PNG to. If omitted, the image is rendered to
    /// the terminal only.
    #[arg(value_name = "output-image", conflicts_with = "output_image_flag")]
    output_image_arg: Option<PathBuf>,

    /// Path to write the rendered PNG to. If omitted, the image is rendered to
    /// the terminal only.
    #[arg(
        value_name = "output-image",
        short = 'o',
        long = "output-image",
        conflicts_with = "output_image_arg"
    )]
    output_image_flag: Option<PathBuf>,

    /// Also display the rendered image in the terminal. Implied when
    /// `output-image` is omitted.
    #[arg(value_name = "terminal", long)]
    terminal: bool,

    /// Render width in pixels.
    #[arg(value_name = "resolution-x", long, default_value_t = 1920)]
    resolution_x: u32,

    /// Render height in pixels.
    #[arg(value_name = "resolution-y", long, default_value_t = 1080)]
    resolution_y: u32,

    /// Camera focal length in millimeters. Only valid with
    /// `--projection perspective`. Mutually exclusive with `--fov`.
    #[arg(value_name = "focal-length", long, conflicts_with = "fov")]
    focal_length: Option<f64>,

    /// Horizontal field of view in degrees. Only valid with
    /// `--projection perspective`. Mutually exclusive with `--focal-length`.
    #[arg(value_name = "fov", long)]
    fov: Option<f64>,

    /// Camera projection.
    #[arg(
        value_name = "projection",
        long,
        value_enum,
        default_value_t = utilities::Projection::Perspective,
    )]
    projection: utilities::Projection,

    /// Orthographic scale (world-units visible across the frame). Only valid
    /// with `--projection orthographic`. Defaults to the scene-bounds diagonal
    /// when omitted.
    #[arg(value_name = "ortho-scale", long)]
    ortho_scale: Option<f64>,

    /// Near clipping plane distance.
    #[arg(value_name = "near", long, default_value_t = 0.1)]
    near: f64,

    /// Far clipping plane distance.
    #[arg(value_name = "far", long, default_value_t = 1000.0)]
    far: f64,

    /// Render engine.
    #[arg(value_name = "renderer", long, value_enum, default_value_t = utilities::Renderer::Eevee)]
    renderer: utilities::Renderer,

    /// Render samples (AA / path-tracing samples depending on renderer).
    #[arg(value_name = "samples", long, default_value_t = 64)]
    samples: u32,

    #[command(flatten)]
    camera: utilities::CameraArgs,
}

impl Render {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Render {
            input_fbx,
            output_image_arg,
            output_image_flag,
            terminal,
            resolution_x,
            resolution_y,
            focal_length,
            fov,
            projection,
            ortho_scale,
            near,
            far,
            renderer,
            samples,
            camera,
        } = self;

        camera.validate()?;

        let output_image = output_image_arg.or(output_image_flag);
        let display_in_terminal = terminal || output_image.is_none();

        let (render_path, temp_dir) = match &output_image {
            Some(path) => (path.clone(), None),
            None => {
                let dir = dependencies.create_temp_dir()?;
                (dir.join("render.png"), Some(dir))
            }
        };

        match projection {
            utilities::Projection::Perspective => {
                if ortho_scale.is_some() {
                    return Err(Error::IO(IOError::new(
                        ErrorKind::InvalidInput,
                        "--ortho-scale is only valid with --projection orthographic",
                    )));
                }
            }
            utilities::Projection::Orthographic => {
                if focal_length.is_some() {
                    return Err(Error::IO(IOError::new(
                        ErrorKind::InvalidInput,
                        "--focal-length is only valid with --projection perspective",
                    )));
                }
                if fov.is_some() {
                    return Err(Error::IO(IOError::new(
                        ErrorKind::InvalidInput,
                        "--fov is only valid with --projection perspective",
                    )));
                }
            }
        }

        let (lens_mode, lens_value) = match (focal_length, fov) {
            (_, Some(fov)) => ("fov", fov),
            (Some(focal), _) => ("focal", focal),
            (None, None) => ("focal", 50.0),
        };
        let ortho_scale_value = ortho_scale.unwrap_or(0.0);

        let subject_names = match &camera.subject {
            Some(pattern) => resolve_subject_names(&dependencies, &input_fbx, pattern)?,
            None => Vec::new(),
        };

        let result = (|| -> Result<()> {
            let mut args: Vec<OsString> = vec![
                input_fbx.clone().into_os_string(),
                render_path.clone().into_os_string(),
                resolution_x.to_string().into(),
                resolution_y.to_string().into(),
                projection.as_blender_type().into(),
                lens_mode.into(),
                lens_value.to_string().into(),
                ortho_scale_value.to_string().into(),
                near.to_string().into(),
                far.to_string().into(),
                renderer.as_blender_engine().into(),
                samples.to_string().into(),
            ];
            args.extend(camera.to_python_args(&subject_names));

            let stdout = dependencies.exec_temp_blender_scripts(
                &utilities::FBX_RENDER_PY,
                [&utilities::COMMON_PY],
                &args,
            )?;

            if display_in_terminal {
                dependencies.display_image_in_terminal(&render_path)?;
            }

            camera.emit_print_camera(&dependencies, &stdout)?;

            Ok(())
        })();

        if let Some(dir) = temp_dir {
            let _ = dependencies.remove_dir_all(&dir);
        }

        result
    }
}

fn resolve_subject_names(
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

    let matched_names: Vec<String> = entries
        .iter()
        .zip(matched.iter())
        .filter(|&(_, &m)| m)
        .map(|((name, _, _), _)| name.clone())
        .collect();

    if matched_names.is_empty() {
        return Err(Error::IO(IOError::new(
            ErrorKind::NotFound,
            format!("no object matched pattern '{pattern}'"),
        )));
    }

    Ok(matched_names)
}
