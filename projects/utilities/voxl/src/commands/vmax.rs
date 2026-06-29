use crate::{CameraView, ColorFormat, Dependencies, Format, Result};
use clap::Parser;
use std::path::PathBuf;

/// Converts a voxel file to the Voxel Max format.
#[derive(Clone, Debug, Parser)]
#[command(name = "vmax")]
pub struct Vmax {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// The output `.vmax` package directory to create.
    #[arg(value_name = "output")]
    output: PathBuf,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,

    /// Where to store object colors in the package.
    #[arg(value_name = "color-format", long, default_value = "png")]
    color_format: ColorFormat,

    /// Which scene camera the rebuilt document opens with. Omitted, the input's
    /// `voxel-max` ext camera is kept when present, else the empty default.
    #[arg(value_name = "camera", long)]
    camera: Option<CameraView>,
}

impl Vmax {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.to_vmax(
            &self.input,
            self.from,
            &self.output,
            self.color_format,
            self.camera,
        )
    }
}
