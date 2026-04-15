use crate::{Dependencies, Error, Result, utilities};
use clap::Parser;
use std::{
    ffi::OsString,
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

    /// Camera position in world space (three space-separated floats). If
    /// omitted, a position is auto-computed from the scene bounds to frame the
    /// imported meshes.
    #[arg(value_name = "camera-position", long, num_args = 3)]
    camera_position: Vec<f64>,

    /// Point the camera looks at in world space (three space-separated floats).
    #[arg(
        value_name = "camera-target",
        long,
        num_args = 3,
        default_values_t = [0.0_f64, 0.0, 0.0],
    )]
    camera_target: Vec<f64>,

    /// Camera up direction hint (three space-separated floats).
    #[arg(
        value_name = "camera-up",
        long,
        num_args = 3,
        default_values_t = [0.0_f64, 0.0, 1.0],
    )]
    camera_up: Vec<f64>,

    /// Render width in pixels.
    #[arg(value_name = "resolution-x", long, default_value_t = 1920)]
    resolution_x: u32,

    /// Render height in pixels.
    #[arg(value_name = "resolution-y", long, default_value_t = 1080)]
    resolution_y: u32,

    /// Camera focal length in millimeters.
    #[arg(value_name = "focal-length", long, default_value_t = 50.0)]
    focal_length: f64,

    /// Render engine.
    #[arg(value_name = "renderer", long, value_enum, default_value_t = utilities::Renderer::Eevee)]
    renderer: utilities::Renderer,

    /// Render samples (AA / path-tracing samples depending on renderer).
    #[arg(value_name = "samples", long, default_value_t = 64)]
    samples: u32,
}

impl Render {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Render {
            input_fbx,
            output_image_arg,
            output_image_flag,
            terminal,
            camera_position,
            camera_target,
            camera_up,
            resolution_x,
            resolution_y,
            focal_length,
            renderer,
            samples,
        } = self;

        let output_image = output_image_arg.or(output_image_flag);
        let display_in_terminal = terminal || output_image.is_none();

        let (render_path, temp_dir) = match &output_image {
            Some(path) => (path.clone(), None),
            None => {
                let dir = dependencies.create_temp_dir()?;
                (dir.join("render.png"), Some(dir))
            }
        };

        let auto_frame = camera_position.is_empty();
        let camera_position = if auto_frame {
            vec![0.0, 0.0, 0.0]
        } else {
            camera_position
        };

        let result = (|| -> Result<()> {
            let args: [OsString; 17] = [
                input_fbx.clone().into_os_string(),
                render_path.clone().into_os_string(),
                vec_float(&camera_position, 0)?,
                vec_float(&camera_position, 1)?,
                vec_float(&camera_position, 2)?,
                vec_float(&camera_target, 0)?,
                vec_float(&camera_target, 1)?,
                vec_float(&camera_target, 2)?,
                vec_float(&camera_up, 0)?,
                vec_float(&camera_up, 1)?,
                vec_float(&camera_up, 2)?,
                resolution_x.to_string().into(),
                resolution_y.to_string().into(),
                focal_length.to_string().into(),
                renderer.as_blender_engine().into(),
                samples.to_string().into(),
                if auto_frame { "1" } else { "0" }.into(),
            ];

            dependencies.exec_temp_blender_scripts(
                &utilities::FBX_RENDER_PY,
                [&utilities::COMMON_PY],
                args,
            )?;

            if display_in_terminal {
                dependencies.display_image_in_terminal(&render_path)?;
            }

            Ok(())
        })();

        if let Some(dir) = temp_dir {
            let _ = dependencies.remove_dir_all(&dir);
        }

        result
    }
}

fn vec_float(values: &[f64], index: usize) -> Result<OsString> {
    values
        .get(index)
        .map(|v| OsString::from(v.to_string()))
        .ok_or_else(|| {
            Error::IO(IOError::new(
                ErrorKind::InvalidInput,
                format!("expected 3 floats, got {}", values.len()),
            ))
        })
}
