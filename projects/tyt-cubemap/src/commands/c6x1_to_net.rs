use crate::{Dependencies, Result, utilities};
use clap::Parser;
use std::path::Path;

/// Converts a c6x1 cube strip into a cube net cross layout.
#[derive(Clone, Debug, Parser)]
pub struct C6x1ToNet {
    /// Base name for the input strip (`{base}.png`).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-net`.
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

impl C6x1ToNet {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-net", self.base));
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

fn build_cube_net(
    deps: &impl Dependencies,
    base: &str,
    out_base: &str,
    do_square: bool,
    point: bool,
    output_size: Option<u32>,
    tmp_dir: &Path,
) -> Result<()> {
    let strip_path = format!("{base}.png");
    let size = utilities::identify_u32(deps, &strip_path, "%h")?;

    let cube_net_path =
        utilities::c6x1_to_cube_net(deps, &strip_path, size, tmp_dir, point, output_size)?;
    let cube_net_str = cube_net_path.to_string_lossy().into_owned();

    let out_path = format!("{out_base}.png");
    if do_square {
        utilities::square(deps, &cube_net_str, &out_path)?;
    } else {
        deps.rename_file(&cube_net_path, &out_path)?;
    }

    Ok(())
}
