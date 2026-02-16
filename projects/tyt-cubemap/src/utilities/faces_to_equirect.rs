use crate::{Dependencies, Result};
use std::path::Path;

const FACES: &[&str] = &["left", "right", "up", "down", "front", "back"];

/// Appends six cube face images into a horizontal strip and converts to equirectangular.
pub fn faces_to_equirect(
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
