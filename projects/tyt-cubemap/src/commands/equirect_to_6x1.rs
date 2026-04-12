use crate::{Dependencies, Result};
use clap::Parser;

/// Converts an equirectangular panorama into a c6x1 horizontal strip.
#[derive(Clone, Debug, Parser)]
pub struct EquirectTo6x1 {
    /// Base name for the input equirectangular image (`{base}.png`).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-6x1`.
    #[arg(value_name = "out-base")]
    out_base: Option<String>,

    /// Side length in pixels for each face in the strip.
    #[arg(value_name = "size", short, long, default_value_t = 512)]
    size: u32,

    /// Use point (nearest-neighbor) interpolation for v360 reprojection.
    #[arg(value_name = "point", long)]
    point: bool,

    /// Final side length for each face. When set, the strip is point-resized from
    /// `--size` to this dimension, preserving hard edges at a larger resolution.
    #[arg(value_name = "output-size", long)]
    output_size: Option<u32>,
}

impl EquirectTo6x1 {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-6x1", self.base));
        let out_path = format!("{out_base}.png");

        let vf = if self.point {
            format!(
                "v360=input=equirect:output=c6x1,scale={}:{}:flags=neighbor",
                6 * self.size,
                self.size,
            )
        } else {
            format!(
                "v360=input=equirect:output=c6x1,scale={}:{}",
                6 * self.size,
                self.size,
            )
        };
        deps.exec_ffmpeg([
            "-y",
            "-i",
            &format!("{}.png", self.base),
            "-vf",
            &vf,
            &out_path,
        ])?;

        if let Some(out_size) = self.output_size {
            let dimensions = format!("{}x{}", 6 * out_size, out_size);
            deps.exec_magick([
                out_path.as_str(),
                "-filter",
                "point",
                "-resize",
                &dimensions,
                &out_path,
            ])?;
        }

        deps.write_stdout(format!("Wrote: {out_path}\n").as_bytes())?;
        Ok(())
    }
}
