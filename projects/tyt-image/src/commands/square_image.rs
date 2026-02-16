use crate::{Dependencies, Result};
use clap::Parser;

/// Pads an image to a square canvas with transparent background.
#[derive(Clone, Debug, Parser)]
pub struct SquareImage {
    /// Base name for the input image (`{base}.png`).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-square`.
    #[arg(value_name = "out-base")]
    out_base: Option<String>,
}

impl SquareImage {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-square", self.base));
        let in_path = format!("{}.png", self.base);
        let out_path = format!("{out_base}.png");
        deps.exec_magick([
            in_path.as_str(),
            "-background",
            "none",
            "-gravity",
            "center",
            "-extent",
            "%[fx:max(w,h)]x%[fx:max(w,h)]",
            &out_path,
        ])?;
        deps.write_stdout(format!("Wrote: {out_path}\n").as_bytes())?;
        Ok(())
    }
}
