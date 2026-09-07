use crate::{
    Dependencies, Result, VoxelInput, cli_value_parser,
    commands::{CameraView, convert, resolve_scene_camera},
};
use clap::Parser;
use std::path::PathBuf;
use voxconv::{
    WriteFormat,
    vmax::{VMaxColorFormat, VMaxWriteOptions},
};

/// Converts a voxel file to the Voxel Max format.
#[derive(Clone, Debug, Parser)]
#[command(name = "vmax")]
pub struct ToVmax {
    #[command(flatten)]
    input: VoxelInput,

    /// The output `.vmax` package directory to create. Defaults to the input
    /// path with a `.vmax` extension.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    /// Where to store object colors in the package.
    #[arg(
        value_name = "color-format",
        long,
        default_value = "png",
        value_parser = cli_value_parser::<VMaxColorFormat>()
    )]
    color_format: VMaxColorFormat,

    /// Which scene camera the rebuilt document opens with. Omitted, the input's
    /// `vmax` ext camera is kept when present, else the empty default.
    #[arg(value_name = "camera", long)]
    camera: Option<CameraView>,
}

impl ToVmax {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let from = self.input.resolve_format()?;

        let to = WriteFormat::VMax(VMaxWriteOptions {
            color_format: self.color_format,
            scene_camera: resolve_scene_camera(self.camera),
        });

        let output = self.input.output_path(self.output, to.extension());

        convert(&dependencies, &self.input.path, from, &output, &to)
    }
}
