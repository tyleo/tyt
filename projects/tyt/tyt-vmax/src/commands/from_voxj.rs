use crate::{ColorFormat, Dependencies, Result};
use clap::Parser;
use std::path::PathBuf;

/// Converts a Voxel Json document into a `.vmax` package directory.
#[derive(Clone, Debug, Parser)]
#[command(name = "from-voxj")]
pub struct FromVoxj {
    /// The input `.voxj` or `.voxjz` document.
    #[arg(value_name = "input-voxj")]
    input_voxj: PathBuf,

    /// The output `.vmax` package directory to create.
    #[arg(value_name = "output-vmax")]
    output_vmax: PathBuf,

    /// Where to store object colors: `png` (default) writes a 256x1
    /// `palette*.png` image and leaves the material `palette*.settings.vmaxpsb`
    /// without a `colors` table; `plist` writes the colors into that sidecar's
    /// `colors` table and emits no image (the `pal` reference still names the
    /// absent png); `all` writes both.
    #[arg(value_name = "color-format", long, default_value = "png")]
    color_format: ColorFormat,
}

impl FromVoxj {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let voxj_bytes = dependencies.read_file(&self.input_voxj)?;
        dependencies.write_vmax_package(&voxj_bytes, &self.output_vmax, self.color_format)?;
        Ok(())
    }
}
