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
}

impl FacesToEquirect {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-equirect", self.base));
        let tmp_dir = deps.create_temp_dir()?;
        let result = utilities::faces_to_equirect(&deps, &self.base, &out_base, &tmp_dir);
        deps.remove_dir_all(&tmp_dir)?;
        let out_path = result?;
        deps.write_stdout(format!("Wrote: {out_path}\n").as_bytes())?;
        Ok(())
    }
}
