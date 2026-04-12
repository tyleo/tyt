use crate::{Dependencies, Result, utilities};
use clap::Parser;

/// Converts six cube face images into a single equirectangular panorama.
#[derive(Clone, Debug, Parser)]
pub struct FacesToEquirect {
    /// Base name for input face files (`{base}-left.png`, etc.).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-equirect`.
    #[arg(value_name = "out-base")]
    out_base: Option<String>,

    /// Use nearest-neighbor interpolation for v360 reprojection.
    #[arg(value_name = "nearest", long)]
    nearest: bool,

    /// Pixelate (point-resize) the faces to the given height before converting.
    /// Implies `--nearest`.
    #[arg(value_name = "pixelate", long)]
    pixelate: Option<u32>,
}

impl FacesToEquirect {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-equirect", self.base));
        let nearest = self.nearest || self.pixelate.is_some();
        let tmp_dir = deps.create_temp_dir()?;

        let result = (|| {
            let equirect_base = if let Some(size) = self.pixelate {
                let tmp_base = tmp_dir.join("face");
                let tmp_base_str = tmp_base.to_string_lossy().into_owned();
                utilities::pixelate_faces(&deps, &self.base, &tmp_base_str, size)?;
                tmp_base_str
            } else {
                self.base.clone()
            };
            utilities::faces_to_equirect(&deps, &equirect_base, &out_base, &tmp_dir, nearest)
        })();

        deps.remove_dir_all(&tmp_dir)?;
        let out_path = result?;
        deps.write_stdout(format!("Wrote: {out_path}\n").as_bytes())?;
        Ok(())
    }
}
