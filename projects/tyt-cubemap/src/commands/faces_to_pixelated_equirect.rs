use crate::{
    Dependencies, Result,
    commands::{faces_to_equirect::faces_to_equirect, pixelate_faces::pixelate_faces},
};
use clap::Parser;

/// Pixelates cube face images and then converts them to an equirectangular panorama.
#[derive(Clone, Debug, Parser)]
pub struct FacesToPixelatedEquirect {
    /// Base name for input face files (`{base}-left.png`, etc.).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-px-equirect`.
    #[arg(value_name = "out-base")]
    out_base: Option<String>,

    /// Target height in pixels for pixelation (halved internally).
    #[arg(value_name = "size", short, long, default_value_t = 256)]
    size: u32,
}

impl FacesToPixelatedEquirect {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-px-equirect", self.base));
        let half_size = self.size / 2;

        let tmp_dir = deps.create_temp_dir()?;
        let tmp_base = tmp_dir.join("face");
        let tmp_base_str = tmp_base.to_string_lossy().into_owned();

        let result = (|| {
            pixelate_faces(&deps, &self.base, &tmp_base_str, half_size)?;
            faces_to_equirect(&deps, &tmp_base_str, &out_base, &tmp_dir)
        })();

        deps.remove_dir_all(&tmp_dir)?;
        let out_path = result?;
        deps.write_stdout(format!("Wrote: {out_path}\n").as_bytes())?;
        Ok(())
    }
}
