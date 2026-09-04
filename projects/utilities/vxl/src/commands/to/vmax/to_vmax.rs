use crate::{
    Dependencies, Format, Result,
    commands::{CameraView, ColorFormat},
};
use clap::Parser;
use std::path::PathBuf;

/// Converts a voxel file to the Voxel Max format.
#[derive(Clone, Debug, Parser)]
#[command(name = "vmax")]
pub struct ToVmax {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// The output `.vmax` package directory to create. Defaults to the input
    /// path with a `.vmax` extension.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,

    /// Where to store object colors in the package.
    #[arg(value_name = "color-format", long, default_value = "png")]
    color_format: ColorFormat,

    /// Which scene camera the rebuilt document opens with. Omitted, the input's
    /// `vmax` ext camera is kept when present, else the empty default.
    #[arg(value_name = "camera", long)]
    camera: Option<CameraView>,
}

impl ToVmax {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let output = self
            .output
            .unwrap_or_else(|| self.input.with_extension("vmax"));
        dependencies.to_vmax(
            &self.input,
            self.from,
            &output,
            self.color_format,
            self.camera,
        )
    }
}
