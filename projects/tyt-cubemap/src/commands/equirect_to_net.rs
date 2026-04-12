use crate::{Dependencies, Result, utilities};
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

    /// Use nearest-neighbor filtering on the final `--output-size` resize.
    #[arg(value_name = "point", long)]
    point: bool,

    /// Use nearest-neighbor (`interp=near`) on the v360 reprojection itself.
    #[arg(value_name = "point-reprojection", long)]
    point_reprojection: bool,

    /// Final side length for each face in the output net. When set, the net is
    /// resized to this resolution. Combine with `--point` for nearest-neighbor
    /// filtering that preserves hard edges.
    #[arg(value_name = "output-size", long)]
    output_size: Option<u32>,
}

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
            self.point,
            self.point_reprojection,
            self.output_size,
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
    point: bool,
    point_reprojection: bool,
    output_size: Option<u32>,
    tmp_dir: &Path,
) -> Result<()> {
    let c3x2_path = tmp_dir.join("c3x2.png");
    let c3x2_str = c3x2_path.to_string_lossy().into_owned();

    let vf = if point_reprojection {
        format!(
            "v360=input=equirect:output=c3x2:interp=near,scale={}:{}",
            3 * size,
            2 * size,
        )
    } else {
        format!(
            "v360=input=equirect:output=c3x2,scale={}:{}",
            3 * size,
            2 * size,
        )
    };
    deps.exec_ffmpeg(["-y", "-i", &format!("{base}.png"), "-vf", &vf, &c3x2_str])?;

    let cube_net_path =
        utilities::c3x2_to_cube_net(deps, &c3x2_str, size, tmp_dir, point, output_size)?;
    let cube_net_str = cube_net_path.to_string_lossy().into_owned();

    let out_path = format!("{out_base}.png");
    if do_square {
        utilities::square(deps, &cube_net_str, &out_path)?;
    } else {
        deps.rename_file(&cube_net_path, &out_path)?;
    }

    Ok(())
}
