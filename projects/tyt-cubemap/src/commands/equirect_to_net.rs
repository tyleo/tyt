use crate::{Dependencies, Result, commands::square_image::square};
use clap::Parser;
use std::path::Path;

/// Converts an equirectangular panorama into a cube net cross layout.
#[derive(Clone, Debug, Parser)]
pub struct EquirectToNet {
    /// Base name for the input equirectangular image (`{base}.png`).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-cube-net`.
    #[arg(value_name = "out-base")]
    out_base: Option<String>,

    /// Side length in pixels for each cube face.
    #[arg(value_name = "size", short, long, default_value_t = 512)]
    size: u32,

    /// Pad the output to a square canvas.
    #[arg(value_name = "square", long)]
    square: bool,
}

/// Face crop positions in the c3x2 layout used by the cube net: `(col, row, face_name)`.
const C3X2_NET_FACES: &[(u32, u32, &str)] = &[
    (0, 0, "left"),
    (1, 0, "front"),
    (2, 0, "right"),
    (0, 1, "down"),
    (1, 1, "back"),
    (2, 1, "up"),
];

impl EquirectToNet {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-cube-net", self.base));
        let tmp_dir = deps.create_temp_dir()?;
        let result = build_cube_net(
            &deps,
            &self.base,
            &out_base,
            self.size,
            self.square,
            &tmp_dir,
        );
        deps.remove_dir_all(&tmp_dir)?;
        result?;
        deps.write_stdout(format!("Wrote: {out_base}.png\n").as_bytes())?;
        Ok(())
    }
}

fn build_cube_net(
    deps: &impl Dependencies,
    base: &str,
    out_base: &str,
    size: u32,
    do_square: bool,
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

    for &(col, row, face) in C3X2_NET_FACES {
        let crop = format!("crop={size}:{size}:{}:{}", col * size, row * size);
        let out_path = tmp_dir.join(format!("{face}.png"));
        let out_str = out_path.to_string_lossy().into_owned();
        deps.exec_ffmpeg([
            "-y",
            "-i",
            &c3x2_str,
            "-vf",
            &crop,
            "-frames:v",
            "1",
            &out_str,
        ])?;
    }

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

    let out_path = format!("{out_base}.png");
    if do_square {
        square(deps, &cube_net_str, &out_path)?;
    } else {
        deps.rename_file(&cube_net_path, &out_path)?;
    }

    Ok(())
}
