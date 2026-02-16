use crate::{Dependencies, Result};
use clap::Parser;
use std::path::Path;

const FACES: &[&str] = &["left", "right", "up", "down", "front", "back"];

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
        let result = faces_to_equirect(&deps, &self.base, &out_base, &tmp_dir);
        deps.remove_dir_all(&tmp_dir)?;
        let out_path = result?;
        deps.write_stdout(format!("Wrote: {out_path}\n").as_bytes())?;
        Ok(())
    }
}

pub(crate) fn faces_to_equirect(
    deps: &impl Dependencies,
    base: &str,
    out_base: &str,
    tmp_dir: &Path,
) -> Result<String> {
    let strip_path = tmp_dir.join("strip.png");
    let strip_str = strip_path.to_string_lossy().into_owned();
    let out_path = format!("{out_base}.png");

    let mut magick_args: Vec<String> = FACES
        .iter()
        .map(|face| format!("{base}-{face}.png"))
        .collect();
    magick_args.push("+append".into());
    magick_args.push(strip_str.clone());
    deps.exec_magick(magick_args)?;

    deps.exec_ffmpeg([
        "-y",
        "-loglevel",
        "error",
        "-i",
        &strip_str,
        "-vf",
        "v360=c6x1:e",
        &out_path,
    ])?;

    Ok(out_path)
}
