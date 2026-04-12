use crate::{Dependencies, Result, utilities};
use clap::Parser;

/// Combines six cube face images into a c6x1 horizontal strip.
#[derive(Clone, Debug, Parser)]
pub struct FacesToC6x1 {
    /// Base name for input face files (`{base}-left.png`, etc.).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-c6x1`.
    #[arg(value_name = "out-base")]
    out_base: Option<String>,

    /// Use point (nearest-neighbor) interpolation when resizing to `--output-size`.
    #[arg(value_name = "point", long)]
    point: bool,

    /// Final side length for each face in the output strip. When set, the strip is
    /// resized to this resolution. Combine with `--point` for nearest-neighbor
    /// filtering that preserves hard edges.
    #[arg(value_name = "output-size", long)]
    output_size: Option<u32>,
}

impl FacesToC6x1 {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-c6x1", self.base));
        let out_path = format!("{out_base}.png");

        let mut magick_args: Vec<String> = utilities::C6X1_FACES
            .iter()
            .map(|face| format!("{}-{face}.png", self.base))
            .collect();
        magick_args.push("+append".into());
        magick_args.push(out_path.clone());
        deps.exec_magick(magick_args)?;

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
