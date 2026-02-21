use crate::{Dependencies, Error, Result};
use clap::Parser;
use std::path::{Path, PathBuf};

/// Creates an MSE png from material texture maps. The output png packs:
///   R = metalness (metal_rough red channel)
///   G = smoothness (1 - metal_rough alpha)
///   B = emissive (emissive alpha)
/// Optionally copies the albedo texture alongside.
#[derive(Clone, Debug, Parser)]
pub struct CreateMse {
    /// The output base path. Output files will be `{out_base}-mse.png` and
    /// `{out_base}-albedo.png`.
    #[arg(value_name = "out-base")]
    out_base: String,

    /// Search prefix for texture files. When set, searches for
    /// `{prefix}-metalness.png`, `{prefix}-emission.png`, `{prefix}-albedo.png`.
    #[arg(value_name = "prefix", long)]
    prefix: Option<String>,

    /// Explicit path to the metal_rough texture.
    #[arg(value_name = "metal-rough", long)]
    metal_rough: Option<PathBuf>,

    /// Explicit path to the emissive texture.
    #[arg(value_name = "emissive", long)]
    emissive: Option<PathBuf>,

    /// Explicit path to the albedo texture.
    #[arg(value_name = "albedo", long)]
    albedo: Option<PathBuf>,

    /// Skip the metal_rough channel (metalness and smoothness will be black).
    #[arg(
        value_name = "ignore-metal-rough",
        long,
        conflicts_with = "metal_rough"
    )]
    ignore_metal_rough: bool,

    /// Skip the emissive channel (emission will be black).
    #[arg(value_name = "ignore-emissive", long, conflicts_with = "emissive")]
    ignore_emissive: bool,

    /// Skip the albedo pass-through copy.
    #[arg(value_name = "ignore-albedo", long, conflicts_with = "albedo")]
    ignore_albedo: bool,
}

impl CreateMse {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let CreateMse {
            out_base,
            prefix,
            metal_rough,
            emissive,
            albedo,
            ignore_metal_rough,
            ignore_emissive,
            ignore_albedo,
        } = self;

        // ----------------------------------------------------------------
        // Resolve texture paths
        // ----------------------------------------------------------------
        let metal_rough_path = if ignore_metal_rough {
            None
        } else {
            Some(match metal_rough {
                Some(p) => coerce_png(p),
                None => match &prefix {
                    Some(pfx) => dependencies.glob_single_match(&format!("{pfx}-metalness.png"))?,
                    None => dependencies.glob_single_match("*metalness.png")?,
                },
            })
        };

        let emissive_path = if ignore_emissive {
            None
        } else {
            Some(match emissive {
                Some(p) => coerce_png(p),
                None => match &prefix {
                    Some(pfx) => dependencies.glob_single_match(&format!("{pfx}-emission.png"))?,
                    None => dependencies.glob_single_match("*emission.png")?,
                },
            })
        };

        let albedo_path = if ignore_albedo {
            None
        } else {
            Some(match albedo {
                Some(p) => coerce_png(p),
                None => match &prefix {
                    Some(pfx) => dependencies.glob_single_match(&format!("{pfx}-albedo.png"))?,
                    None => dependencies.glob_single_match("*albedo.png")?,
                },
            })
        };

        // ----------------------------------------------------------------
        // Determine base image for sizing
        // ----------------------------------------------------------------
        let base_img = metal_rough_path
            .as_ref()
            .or(emissive_path.as_ref())
            .or(albedo_path.as_ref())
            .ok_or_else(|| Error::Glob("all channels are ignored; nothing to do".into()))?;

        let size_output = dependencies.exec_magick([
            "identify".into(),
            "-ping".into(),
            "-format".into(),
            "%wx%h".into(),
            base_img.to_string_lossy().into_owned(),
        ])?;
        let size = String::from_utf8_lossy(&size_output).trim().to_string();
        if size.is_empty() {
            return Err(Error::Glob(format!(
                "couldn't determine image size for: {}",
                base_img.display()
            )));
        }

        // ----------------------------------------------------------------
        // Create temp dir for intermediate channel PNGs
        // ----------------------------------------------------------------
        let tmpdir = dependencies.create_temp_dir()?;
        let result = self::create_mse_inner(
            &dependencies,
            &metal_rough_path,
            &emissive_path,
            &albedo_path,
            &out_base,
            &size,
            &tmpdir,
        );
        dependencies.remove_dir_all(&tmpdir)?;
        result
    }
}

fn create_mse_inner(
    dependencies: &impl Dependencies,
    metal_rough_path: &Option<PathBuf>,
    emissive_path: &Option<PathBuf>,
    albedo_path: &Option<PathBuf>,
    out_base: &str,
    size: &str,
    tmpdir: &Path,
) -> Result<()> {
    let r_img = tmpdir.join("r.png");
    let g_img = tmpdir.join("g.png");
    let b_img = tmpdir.join("b.png");

    let r_str = r_img.to_string_lossy().into_owned();
    let g_str = g_img.to_string_lossy().into_owned();
    let b_str = b_img.to_string_lossy().into_owned();

    // R channel: metalness (red channel of metal_rough)
    match metal_rough_path {
        None => {
            dependencies.exec_magick(["-size", size, "xc:black", &r_str])?;
        }
        Some(mr) => {
            let mr_str = mr.to_string_lossy().into_owned();
            dependencies.exec_magick([
                &mr_str,
                "-colorspace",
                "sRGB",
                "-alpha",
                "on",
                "-channel",
                "R",
                "-separate",
                "+channel",
                "-resize",
                &format!("{size}!"),
                &r_str,
            ])?;
        }
    }

    // G channel: smoothness = 1 - roughness (alpha channel of metal_rough, inverted)
    match metal_rough_path {
        None => {
            dependencies.exec_magick(["-size", size, "xc:black", &g_str])?;
        }
        Some(mr) => {
            let mr_str = mr.to_string_lossy().into_owned();
            dependencies.exec_magick([
                &mr_str,
                "-colorspace",
                "sRGB",
                "-alpha",
                "on",
                "-alpha",
                "extract",
                "-fx",
                "u==0 ? 0 : 1-u",
                "-resize",
                &format!("{size}!"),
                &g_str,
            ])?;
        }
    }

    // B channel: emissive (alpha channel of emissive)
    match emissive_path {
        None => {
            dependencies.exec_magick(["-size", size, "xc:black", &b_str])?;
        }
        Some(em) => {
            let em_str = em.to_string_lossy().into_owned();
            dependencies.exec_magick([
                &em_str,
                "-colorspace",
                "sRGB",
                "-alpha",
                "on",
                "-alpha",
                "extract",
                "-resize",
                &format!("{size}!"),
                &b_str,
            ])?;
        }
    }

    // Combine R/G/B into MSE
    let mse_out = format!("{out_base}-mse.png");
    dependencies.exec_magick([
        &r_str,
        &g_str,
        &b_str,
        "-combine",
        "-colorspace",
        "sRGB",
        &mse_out,
    ])?;

    dependencies.write_stdout(format!("Wrote: {mse_out}\n").as_bytes())?;

    // Copy albedo if not ignored
    if let Some(albedo) = albedo_path {
        let albedo_out = format!("{out_base}-albedo.png");
        dependencies.copy_file(albedo, &albedo_out)?;
        dependencies.write_stdout(format!("Wrote: {albedo_out}\n").as_bytes())?;
    }

    Ok(())
}

/// If the path has no extension, assume `.png`.
fn coerce_png(p: PathBuf) -> PathBuf {
    if p.extension().is_none() {
        p.with_extension("png")
    } else {
        p
    }
}
