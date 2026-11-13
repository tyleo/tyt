//! voxrender-bench's box view as a headless desktop app. Each subcommand
//! draws or checks one view of a scene.

use clap::{Args, Parser, Subcommand, value_parser};
use png::{BitDepth, ColorType, Encoder, SrgbRenderingIntent};
use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
    process,
};
use voxrender::{RenderBloom, RenderScene};
use voxrender_bench::{
    Error as BenchError, bench_scene, lit_check, look_from, model_scene, retain_headset_views,
    shape_view, view_check, view_image,
};

// Internal API

mod error;
mod headless_gpu;
mod result;

pub(crate) use error::*;
pub(crate) use headless_gpu::*;
pub(crate) use result::*;

/// Draws and checks voxrender-bench's box view.
#[derive(Clone, Debug, Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// Draws the first view of a bench scene or of a model as a shaded box
    /// raster and writes the image as a PNG.
    #[command(name = "view")]
    View {
        /// The start of a bench scene's name, such as `S1`, or the path of
        /// a Voxel Json model.
        #[arg(value_name = "scene")]
        scene: String,

        /// The PNG to write.
        #[arg(value_name = "output-png")]
        output: PathBuf,

        /// The width of the image.
        #[arg(
            value_name = "pixels",
            long,
            default_value_t = 1680,
            value_parser = value_parser!(u32).range(1..=8192)
        )]
        width: u32,

        /// The height of the image.
        #[arg(
            value_name = "pixels",
            long,
            default_value_t = 1760,
            value_parser = value_parser!(u32).range(1..=8192)
        )]
        height: u32,

        #[command(flatten)]
        look: Look,
    },

    /// Compares the shaded view of a bench scene or of a model with the
    /// reference's image of it and prints how far they differ.
    #[command(name = "view-check")]
    ViewCheck {
        /// The start of a bench scene's name, such as `S1`, or the path of
        /// a Voxel Json model.
        #[arg(value_name = "scene")]
        scene: String,

        /// The side of the square image.
        #[arg(
            value_name = "pixels",
            long,
            default_value_t = 256,
            value_parser = value_parser!(u32).range(1..=4096)
        )]
        size: u32,

        /// Looks through a Quest 3's two eyes at the view's pose in place
        /// of the view.
        #[arg(value_name = "headset", long)]
        headset: bool,

        #[command(flatten)]
        look: Look,
    },

    /// Compares the GPU's fill of the corner shadows of a bench scene or of
    /// a model with the fill through the reference's walk and prints how far
    /// they differ.
    #[command(name = "lit-check")]
    LitCheck {
        /// The start of a bench scene's name, such as `S1`, or the path of
        /// a Voxel Json model.
        #[arg(value_name = "scene")]
        scene: String,
    },
}

/// Where a view looks from in place of the scene's first view.
#[derive(Clone, Debug, Args)]
struct Look {
    /// The point the view looks from, in the scene's meters.
    #[arg(
        value_name = "meters",
        long,
        num_args = 3,
        allow_negative_numbers = true,
        requires = "look_at"
    )]
    eye: Option<Vec<f64>>,

    /// The point the view looks at, with +Y up.
    #[arg(
        value_name = "meters",
        long,
        num_args = 3,
        allow_negative_numbers = true,
        requires = "eye"
    )]
    look_at: Option<Vec<f64>>,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::View {
            scene,
            output,
            width,
            height,
            look,
        } => view(&scene, &look, &output, width, height).map(|()| String::new()),

        Command::ViewCheck {
            scene,
            size,
            headset,
            look,
        } => check(&scene, &look, size, headset),

        Command::LitCheck { scene } => check_lit(&scene),
    };

    match result {
        Ok(report) => print!("{report}"),

        Err(error) => {
            eprintln!("error: {error}");
            process::exit(1);
        }
    }
}

/// The model at the path `scene` under its bloom, or else the bench scene
/// whose name starts with `scene` under none.
fn scene_of(scene: &str) -> Result<(RenderScene, RenderBloom)> {
    let path = Path::new(scene);

    if path.is_file() {
        return model_scene(path).map_err(Error::Bench);
    }

    bench_scene(scene)
        .map(|scene| (scene, RenderBloom::default()))
        .ok_or_else(|| Error::NoScene(scene.to_owned()))
}

fn looked(scene: &str, look: &Look) -> Result<(RenderScene, RenderBloom)> {
    let (mut render_scene, bloom) = scene_of(scene)?;

    if let (Some(eye), Some(look_at)) = (&look.eye, &look.look_at) {
        let point = |values: &[f64]| [values[0], values[1], values[2]];

        look_from(&mut render_scene, point(eye), point(look_at)).map_err(Error::Bench)?;
    }

    Ok((render_scene, bloom))
}

fn check(scene: &str, look: &Look, size: u32, headset: bool) -> Result<String> {
    let (mut render_scene, bloom) = looked(scene, look)?;

    let view_ids = if headset {
        retain_headset_views(&mut render_scene).map_err(Error::Bench)?
    } else {
        let (view_id, _) = render_scene
            .iter_views()
            .next()
            .ok_or(Error::Bench(BenchError::NoView))?;

        [view_id; 2]
    };

    let gpu = headless_gpu()?;

    view_check(&gpu, &render_scene, bloom, view_ids, size, size).map_err(Error::Bench)
}

fn check_lit(scene: &str) -> Result<String> {
    let (render_scene, _) = scene_of(scene)?;
    let gpu = headless_gpu()?;

    lit_check(&gpu, &render_scene).map_err(Error::Bench)
}

fn view(scene: &str, look: &Look, output: &Path, width: u32, height: u32) -> Result<()> {
    let (mut render_scene, bloom) = looked(scene, look)?;

    shape_view(&mut render_scene, width, height).map_err(Error::Bench)?;

    let (view_id, _) = render_scene
        .iter_views()
        .next()
        .ok_or(Error::Bench(BenchError::NoView))?;

    let gpu = headless_gpu()?;

    let [texels, _] = view_image(&gpu, &render_scene, bloom, [view_id; 2], width, height)
        .map_err(Error::Bench)?;

    let file = File::create(output).map_err(Error::Write)?;
    let mut encoder = Encoder::new(BufWriter::new(file), width, height);

    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_source_srgb(SrgbRenderingIntent::Perceptual);

    encoder
        .write_header()
        .and_then(|mut writer| writer.write_image_data(&texels))
        .map_err(Error::Png)
}
