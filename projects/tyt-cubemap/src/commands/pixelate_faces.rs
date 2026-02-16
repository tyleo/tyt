use crate::{Dependencies, Result};
use clap::Parser;

const FACES: &[&str] = &["left", "right", "up", "down", "front", "back"];

/// Pixelates (point-resizes) six cube face images.
#[derive(Clone, Debug, Parser)]
pub struct PixelateFaces {
    /// Base name for input face files (`{base}-left.png`, etc.).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-px`.
    #[arg(value_name = "out-base")]
    out_base: Option<String>,

    /// Target height in pixels for each face.
    #[arg(value_name = "size", short, long, default_value_t = 256)]
    size: u32,
}

impl PixelateFaces {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self.out_base.unwrap_or_else(|| format!("{}-px", self.base));
        pixelate_faces(&deps, &self.base, &out_base, self.size)?;
        deps.write_stdout(format!("Wrote resized faces: {out_base}-*.png\n").as_bytes())?;
        Ok(())
    }
}

pub(crate) fn pixelate_faces(
    deps: &impl Dependencies,
    base: &str,
    out_base: &str,
    size: u32,
) -> Result<()> {
    for face in FACES {
        let in_path = format!("{base}-{face}.png");
        let out_path = format!("{out_base}-{face}.png");
        deps.exec_magick([
            in_path.as_str(),
            "-filter",
            "point",
            "-resize",
            &format!("x{size}"),
            &out_path,
        ])?;
    }
    Ok(())
}
