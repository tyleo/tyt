use crate::{Dependencies, Result};
use clap::Parser;

/// Converts a c6x1 cube strip into an equirectangular panorama.
#[derive(Clone, Debug, Parser)]
pub struct C6x1ToEquirect {
    /// Base name for the input strip (`{base}.png`).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-equirect`.
    #[arg(value_name = "out-base")]
    out_base: Option<String>,

    /// Use point (nearest-neighbor) interpolation for v360 reprojection.
    #[arg(value_name = "point", long)]
    point: bool,

    /// Final output height in pixels. When set, the equirectangular image is
    /// point-resized up to this height, preserving hard edges at a larger resolution.
    #[arg(value_name = "output-size", long)]
    output_size: Option<u32>,
}

impl C6x1ToEquirect {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-equirect", self.base));
        let out_path = format!("{out_base}.png");

        let vf = if self.point {
            "v360=c6x1:e:flags=neighbor"
        } else {
            "v360=c6x1:e"
        };
        deps.exec_ffmpeg([
            "-y",
            "-loglevel",
            "error",
            "-i",
            &format!("{}.png", self.base),
            "-vf",
            vf,
            &out_path,
        ])?;

        if let Some(output_size) = self.output_size {
            deps.exec_magick([
                out_path.as_str(),
                "-filter",
                "point",
                "-resize",
                &format!("x{output_size}"),
                &out_path,
            ])?;
        }

        deps.write_stdout(format!("Wrote: {out_path}\n").as_bytes())?;
        Ok(())
    }
}
