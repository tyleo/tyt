use crate::{Dependencies, Result, utilities};
use clap::Parser;
use std::{
    io::{Error as IOError, ErrorKind},
    path::Path,
};

/// Converts six cube face images into a cube net cross layout.
#[derive(Clone, Debug, Parser)]
pub struct FacesToNet {
    /// Base name for input face files (`{base}-left.png`, etc.).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-cube-net`.
    #[arg(value_name = "out-base")]
    out_base: Option<String>,

    /// Pad the output to a square canvas.
    #[arg(value_name = "square", long)]
    square: bool,

    /// Use point (nearest-neighbor) interpolation when resizing to `--output-size`.
    #[arg(value_name = "point", long)]
    point: bool,

    /// Final side length for each face in the output net. When set, the net is
    /// resized to this resolution. Combine with `--point` for nearest-neighbor
    /// filtering that preserves hard edges.
    #[arg(value_name = "output-size", long)]
    output_size: Option<u32>,
}

/// Face order for the c6x1 strip fed to ffmpeg (must match `faces_to_equirect`).
const C6X1_FACES: &[&str] = &["left", "right", "up", "down", "front", "back"];

/// Face crop positions in the c3x2 layout produced by ffmpeg: `(col, row, face_name)`.
const C3X2_NET_FACES: &[(u32, u32, &str)] = &[
    (0, 0, "left"),
    (1, 0, "front"),
    (2, 0, "right"),
    (0, 1, "down"),
    (1, 1, "back"),
    (2, 1, "up"),
];

impl FacesToNet {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-cube-net", self.base));
        let tmp_dir = deps.create_temp_dir()?;
        let result = build_cube_net(
            &deps,
            &self.base,
            &out_base,
            self.square,
            self.point,
            self.output_size,
            &tmp_dir,
        );
        deps.remove_dir_all(&tmp_dir)?;
        result?;
        deps.write_stdout(format!("Wrote: {out_base}.png\n").as_bytes())?;
        Ok(())
    }
}

fn face_size(deps: &impl Dependencies, base: &str) -> Result<u32> {
    let output = deps.exec_magick(["identify", "-format", "%w", &format!("{base}-front.png")])?;
    let text = String::from_utf8_lossy(&output);
    text.trim().parse::<u32>().map_err(|_| {
        IOError::new(
            ErrorKind::InvalidData,
            format!("invalid face width: {text}"),
        )
        .into()
    })
}

fn build_cube_net(
    deps: &impl Dependencies,
    base: &str,
    out_base: &str,
    do_square: bool,
    point: bool,
    output_size: Option<u32>,
    tmp_dir: &Path,
) -> Result<()> {
    let size = face_size(deps, base)?;

    // Build c6x1 strip from face images (same order as faces_to_equirect).
    let strip_path = tmp_dir.join("strip.png");
    let strip_str = strip_path.to_string_lossy().into_owned();
    let mut magick_args: Vec<String> = C6X1_FACES
        .iter()
        .map(|face| format!("{base}-{face}.png"))
        .collect();
    magick_args.push("+append".into());
    magick_args.push(strip_str.clone());
    deps.exec_magick(magick_args)?;

    // Convert c6x1 → c3x2 via ffmpeg v360.
    let c3x2_path = tmp_dir.join("c3x2.png");
    let c3x2_str = c3x2_path.to_string_lossy().into_owned();
    let vf = format!(
        "v360=input=c6x1:output=c3x2,scale={}:{}:flags=neighbor",
        3 * size,
        2 * size,
    );
    deps.exec_ffmpeg(["-y", "-i", &strip_str, "-vf", &vf, &c3x2_str])?;

    // Extract individual faces from the c3x2 grid.
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

    let out_path = format!("{out_base}.png");
    if do_square {
        utilities::square(deps, &cube_net_str, &out_path)?;
    } else {
        deps.rename_file(&cube_net_path, &out_path)?;
    }

    Ok(())
}
