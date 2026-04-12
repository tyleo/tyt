use crate::{Dependencies, Result, utilities};
use std::path::Path;

/// Appends six cube face images into a horizontal strip and converts to equirectangular.
pub fn faces_to_equirect(
    deps: &impl Dependencies,
    base: &str,
    out_base: &str,
    tmp_dir: &Path,
    point: bool,
) -> Result<String> {
    let strip_path = tmp_dir.join("strip.png");
    let strip_str = strip_path.to_string_lossy().into_owned();
    let out_path = format!("{out_base}.png");

    let mut magick_args: Vec<String> = utilities::C6X1_FACES
        .iter()
        .map(|face| format!("{base}-{face}.png"))
        .collect();
    magick_args.push("+append".into());
    magick_args.push(strip_str.clone());
    deps.exec_magick(magick_args)?;

    let vf = if point {
        "v360=c6x1:e:flags=neighbor"
    } else {
        "v360=c6x1:e"
    };
    deps.exec_ffmpeg([
        "-y",
        "-loglevel",
        "error",
        "-i",
        &strip_str,
        "-vf",
        vf,
        &out_path,
    ])?;

    Ok(out_path)
}
