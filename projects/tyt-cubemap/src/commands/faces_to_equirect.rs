use crate::{Dependencies, Result, utilities};
use clap::Parser;

/// Converts six cube face images into a single equirectangular panorama.
#[derive(Clone, Debug, Parser)]
pub struct FacesToEquirect {
    /// Base name for input face files (`{base}-left.png`, etc.).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-equirect`.
    #[arg(value_name = "out-base")]
    out_base: Option<String>,

    /// Use nearest-neighbor filtering on the final `--output-size` resize.
    #[arg(value_name = "point", long)]
    point: bool,

    /// Use nearest-neighbor (`interp=near`) on the v360 reprojection itself.
    #[arg(value_name = "point-reprojection", long)]
    point_reprojection: bool,

    /// Pixelate (point-resize) the faces to the given height before converting.
    /// Implies `--point-reprojection` so the downscaled pixel edges survive the reprojection.
    #[arg(value_name = "pixelate", long)]
    pixelate: Option<u32>,

    /// Final output height in pixels. When set, the equirectangular image is
    /// resized to this height. Combine with `--point` for nearest-neighbor
    /// filtering that preserves hard edges.
    #[arg(value_name = "output-size", long)]
    output_size: Option<u32>,
}

impl FacesToEquirect {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-equirect", self.base));
        let point_reprojection = self.point_reprojection || self.pixelate.is_some();
        let tmp_dir = deps.create_temp_dir()?;

        let result: Result<String> = (|| {
            let equirect_base = if let Some(size) = self.pixelate {
                let tmp_base = tmp_dir.join("face");
                let tmp_base_str = tmp_base.to_string_lossy().into_owned();
                utilities::pixelate_faces(&deps, &self.base, &tmp_base_str, size, None)?;
                tmp_base_str
            } else {
                self.base.clone()
            };
            let out_path = utilities::faces_to_equirect(
                &deps,
                &equirect_base,
                &out_base,
                &tmp_dir,
                point_reprojection,
            )?;
            if let Some(output_size) = self.output_size {
                let resize = format!("x{output_size}");
                if self.point {
                    deps.exec_magick([
                        out_path.as_str(),
                        "-filter",
                        "point",
                        "-resize",
                        &resize,
                        &out_path,
                    ])?;
                } else {
                    deps.exec_magick([out_path.as_str(), "-resize", &resize, &out_path])?;
                }
            }
            Ok(out_path)
        })();

        deps.remove_dir_all(&tmp_dir)?;
        let out_path = result?;
        deps.write_stdout(format!("Wrote: {out_path}\n").as_bytes())?;
        Ok(())
    }
}
