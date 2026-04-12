use crate::{Dependencies, Result, utilities};
use clap::Parser;

/// Extracts six cube face images from a c6x1 cube strip.
#[derive(Clone, Debug, Parser)]
pub struct C6x1ToFaces {
    /// Base name for the input strip (`{base}.png`).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-cube`.
    #[arg(value_name = "out-base")]
    out_base: Option<String>,

    /// Final side length for each output face. When set, faces are point-resized to this dimension,
    /// preserving hard edges at a larger resolution.
    #[arg(value_name = "output-size", long)]
    output_size: Option<u32>,
}

impl C6x1ToFaces {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-cube", self.base));
        let strip_path = format!("{}.png", self.base);
        let size = utilities::identify_u32(&deps, &strip_path, "%h")?;

        for (i, face) in utilities::C6X1_FACES.iter().enumerate() {
            let x = i as u32 * size;
            let vf = if let Some(out_size) = self.output_size {
                format!("crop={size}:{size}:{x}:0,scale={out_size}:{out_size}:flags=neighbor")
            } else {
                format!("crop={size}:{size}:{x}:0")
            };
            let out_path = format!("{out_base}-{face}.png");
            deps.exec_ffmpeg([
                "-y",
                "-i",
                &strip_path,
                "-vf",
                &vf,
                "-frames:v",
                "1",
                &out_path,
            ])?;
        }

        deps.write_stdout(format!("Wrote: {out_base}-*.png\n").as_bytes())?;
        Ok(())
    }
}
