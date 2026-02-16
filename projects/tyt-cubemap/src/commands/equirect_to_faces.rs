use crate::{Dependencies, Result};
use clap::Parser;
use std::path::Path;

/// Converts an equirectangular panorama into six cube face images.
#[derive(Clone, Debug, Parser)]
pub struct EquirectToFaces {
    /// Base name for the input equirectangular image (`{base}.png`).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-cube`.
    #[arg(value_name = "out-base")]
    out_base: Option<String>,

    /// Side length in pixels for each output face.
    #[arg(value_name = "size", short, long, default_value_t = 512)]
    size: u32,
}

/// Face crop positions in the c3x2 layout: `(col, row, face_name)`.
const C3X2_FACES: &[(u32, u32, &str)] = &[
    (0, 0, "left"),
    (1, 0, "right"),
    (2, 0, "up"),
    (0, 1, "down"),
    (1, 1, "front"),
    (2, 1, "back"),
];

impl EquirectToFaces {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-cube", self.base));
        let tmp_dir = deps.create_temp_dir()?;
        let result = equirect_to_faces(&deps, &self.base, &out_base, self.size, &tmp_dir);
        deps.remove_dir_all(&tmp_dir)?;
        result?;
        deps.write_stdout(format!("Wrote: {out_base}-*.png\n").as_bytes())?;
        Ok(())
    }
}

fn equirect_to_faces(
    deps: &impl Dependencies,
    base: &str,
    out_base: &str,
    size: u32,
    tmp_dir: &Path,
) -> Result<()> {
    let c3x2_path = tmp_dir.join("c3x2.png");
    let c3x2_str = c3x2_path.to_string_lossy().into_owned();

    let vf = format!(
        "v360=input=equirect:output=c3x2,scale={}:{}:flags=neighbor",
        3 * size,
        2 * size
    );
    deps.exec_ffmpeg(["-y", "-i", &format!("{base}.png"), "-vf", &vf, &c3x2_str])?;

    for &(col, row, face) in C3X2_FACES {
        let crop = format!("crop={size}:{size}:{}:{}", col * size, row * size);
        let out_path = format!("{out_base}-{face}.png");
        deps.exec_ffmpeg([
            "-y",
            "-i",
            &c3x2_str,
            "-vf",
            &crop,
            "-frames:v",
            "1",
            &out_path,
        ])?;
    }

    Ok(())
}
