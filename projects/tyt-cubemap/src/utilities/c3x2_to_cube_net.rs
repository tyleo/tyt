use crate::{Dependencies, Result};
use std::path::{Path, PathBuf};

/// Face crop positions in the c3x2 layout used by the cube net: `(col, row, face_name)`.
const C3X2_NET_FACES: &[(u32, u32, &str)] = &[
    (0, 0, "left"),
    (1, 0, "front"),
    (2, 0, "right"),
    (0, 1, "down"),
    (1, 1, "back"),
    (2, 1, "up"),
];

/// Assembles a cube-net cross layout from a c3x2 grid into `{tmp_dir}/cube-net.png`.
/// When `output_size` is set, the result is resized; `point` selects nearest-neighbor
/// for that resize. Returns the path to the assembled net.
pub fn c3x2_to_cube_net(
    deps: &impl Dependencies,
    c3x2_path: &str,
    size: u32,
    tmp_dir: &Path,
    point: bool,
    output_size: Option<u32>,
) -> Result<PathBuf> {
    // Extract individual faces from the c3x2 grid.
    for &(col, row, face) in C3X2_NET_FACES {
        let crop = format!("crop={size}:{size}:{}:{}", col * size, row * size);
        let out_path = tmp_dir.join(format!("{face}.png"));
        let out_str = out_path.to_string_lossy().into_owned();
        deps.exec_ffmpeg([
            "-y",
            "-i",
            c3x2_path,
            "-vf",
            &crop,
            "-frames:v",
            "1",
            &out_str,
        ])?;
    }

    // Assemble the cross layout from extracted faces.
    let cube_net_path = tmp_dir.join("cube-net.png");
    let cube_net_str = cube_net_path.to_string_lossy().into_owned();

    let canvas = format!("{}x{}", 4 * size, 3 * size);
    let right_path = tmp_dir.join("right.png").to_string_lossy().into_owned();
    let up_path = tmp_dir.join("up.png").to_string_lossy().into_owned();
    let front_path = tmp_dir.join("front.png").to_string_lossy().into_owned();
    let back_path = tmp_dir.join("back.png").to_string_lossy().into_owned();
    let left_path = tmp_dir.join("left.png").to_string_lossy().into_owned();
    let down_path = tmp_dir.join("down.png").to_string_lossy().into_owned();

    deps.exec_magick([
        "-size",
        &canvas,
        "xc:transparent",
        "(",
        &right_path,
        "-rotate",
        "270",
        ")",
        "-geometry",
        &format!("+{}+0", size),
        "-composite",
        &up_path,
        "-geometry",
        &format!("+0+{}", size),
        "-composite",
        &front_path,
        "-geometry",
        &format!("+{}+{}", size, size),
        "-composite",
        &back_path,
        "-geometry",
        &format!("+{}+{}", 2 * size, size),
        "-composite",
        &left_path,
        "-geometry",
        &format!("+{}+{}", 3 * size, size),
        "-composite",
        "(",
        &down_path,
        "-rotate",
        "90",
        ")",
        "-geometry",
        &format!("+{}+{}", size, 2 * size),
        "-composite",
        &cube_net_str,
    ])?;

    if let Some(out_size) = output_size {
        let resized = tmp_dir.join("cube-net-resized.png");
        let resized_str = resized.to_string_lossy().into_owned();
        let dimensions = format!("{}x{}", 4 * out_size, 3 * out_size);
        if point {
            deps.exec_magick([
                cube_net_str.as_str(),
                "-filter",
                "point",
                "-resize",
                &dimensions,
                &resized_str,
            ])?;
        } else {
            deps.exec_magick([cube_net_str.as_str(), "-resize", &dimensions, &resized_str])?;
        }
        deps.rename_file(&resized, &cube_net_str)?;
    }

    Ok(cube_net_path)
}
