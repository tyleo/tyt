use crate::{Dependencies, Result};
use clap::Parser;

/// Converts an equirectangular panorama into a c6x1 horizontal strip.
#[derive(Clone, Debug, Parser)]
pub struct EquirectToC6x1 {
    /// Base name for the input equirectangular image (`{base}.png`).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-c6x1`.
    #[arg(value_name = "out-base")]
    out_base: Option<String>,

    /// Side length in pixels for each face in the strip.
    #[arg(value_name = "size", short, long, default_value_t = 512)]
    size: u32,

    /// Use nearest-neighbor filtering on the final `--output-size` resize.
    #[arg(value_name = "point", long)]
    point: bool,

    /// Use nearest-neighbor (`interp=near`) on the v360 reprojection itself.
    #[arg(value_name = "point-reprojection", long)]
    point_reprojection: bool,

    /// Final side length for each face. When set, the strip is resized from
    /// `--size` to this dimension. Combine with `--point` for nearest-neighbor
    /// filtering that preserves hard edges.
    #[arg(value_name = "output-size", long)]
    output_size: Option<u32>,
}

impl EquirectToC6x1 {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-c6x1", self.base));
        let out_path = format!("{out_base}.png");

        let vf = if self.point_reprojection {
            format!(
                "v360=input=equirect:output=c6x1:interp=near,scale={}:{}",
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
            if self.point {
                deps.exec_magick([
                    out_path.as_str(),
                    "-filter",
                    "point",
                    "-resize",
                    &dimensions,
                    &out_path,
                ])?;
            } else {
                deps.exec_magick([out_path.as_str(), "-resize", &dimensions, &out_path])?;
            }
        }

        deps.write_stdout(format!("Wrote: {out_path}\n").as_bytes())?;
        Ok(())
    }
}
