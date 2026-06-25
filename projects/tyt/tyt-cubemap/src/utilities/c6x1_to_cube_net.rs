use crate::{Dependencies, Result, utilities};
use std::path::{Path, PathBuf};

/// Assembles a cube-net cross layout from a c6x1 strip into `{tmp_dir}/cube-net.png`.
/// Returns the path to the assembled net.
pub fn c6x1_to_cube_net(
    deps: &impl Dependencies,
    strip_path: &str,
    size: u32,
    tmp_dir: &Path,
    point: bool,
    output_size: Option<u32>,
) -> Result<PathBuf> {
    // Convert c6x1 → c3x2 via ffmpeg v360.
    let c3x2_path = tmp_dir.join("c3x2.png");
    let c3x2_str = c3x2_path.to_string_lossy().into_owned();
    // `interp=near` ensures the nominally-lossless cube→cube rearrangement
    // stays pixel-perfect at face-boundary seams. `flags=neighbor` on the scale
    // is redundant: with equal-area cubemap layouts the scale is a no-op.
    let vf = format!(
        "v360=input=c6x1:output=c3x2:interp=near,scale={}:{}",
        3 * size,
        2 * size,
    );
    deps.exec_ffmpeg(["-y", "-i", strip_path, "-vf", &vf, &c3x2_str])?;

    utilities::c3x2_to_cube_net(deps, &c3x2_str, size, tmp_dir, point, output_size)
}
