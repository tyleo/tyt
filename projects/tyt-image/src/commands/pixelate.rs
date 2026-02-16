use crate::{Dependencies, Result};
use clap::Parser;

/// Pixelates (point-resizes) an image.
#[derive(Clone, Debug, Parser)]
pub struct Pixelate {
    /// Base name for the input image (`{base}.png`).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-px`.
    #[arg(value_name = "out-base")]
    out_base: Option<String>,

    /// Target height in pixels.
    #[arg(value_name = "size", short, long, default_value_t = 256)]
    size: u32,
}

impl Pixelate {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-px", self.base));
        let in_path = format!("{}.png", self.base);
        let out_path = format!("{out_base}.png");
        deps.exec_magick([
            in_path.as_str(),
            "-filter",
            "point",
            "-resize",
            &format!("x{}", self.size),
            &out_path,
        ])?;
        deps.write_stdout(format!("Wrote: {out_path}\n").as_bytes())?;
        Ok(())
    }
}
