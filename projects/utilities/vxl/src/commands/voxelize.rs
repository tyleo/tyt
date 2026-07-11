use crate::{
    Dependencies, FillMode, GridResolutionOptions, MaterialMode, MaxPaletteMaterials, MeshFormat,
    PaletteReductionOptions, Result, VoxjEncodingOptions,
};
use clap::Parser;
use std::{
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
    result::Result as StdResult,
};

/// Rasterizes a mesh into a voxel grid, the inverse of `mesh`.
#[derive(Clone, Debug, Parser)]
#[command(name = "voxelize")]
pub struct Voxelize {
    /// The input glTF (`.gltf`) or GLB (`.glb`) mesh.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// The output `.voxj` or `.voxjz` document to write. Defaults to the input
    /// path with a `.voxj` extension, or `.voxjz` when `--format zip`.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    /// Source mesh format. Inferred from the input extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<MeshFormat>,

    #[command(flatten)]
    resolution_options: GridResolutionOptions,

    /// How the mesh fills the grid, independent of `--material-mode`.
    #[arg(value_name = "fill-mode", long, default_value = "solid")]
    fill_mode: FillMode,

    /// Where each voxel's color comes from, independent of `--fill-mode`.
    #[arg(value_name = "material-mode", long, default_value = "auto")]
    material_mode: MaterialMode,

    /// Fill color as a `#RRGGBBAA` hex. Under `--material-mode flat` it paints
    /// every voxel, white when omitted; under `--fill-mode solid` it paints a
    /// body's interior, the nearest surface color when omitted. Rejected on a
    /// sampling-mode surface, which samples every voxel.
    #[arg(value_name = "fill-color", long, value_parser = parse_hex_rgba)]
    fill_color: Option<[u8; 4]>,

    /// Name for the voxelized object. Defaults to the mesh's own name, else the
    /// input file stem.
    #[arg(value_name = "name", long)]
    name: Option<String>,

    /// Most materials the palette may hold; sampling over this reduces to it.
    /// `none` disables the cap.
    #[arg(value_name = "max-palette-materials", long, default_value = "256")]
    max_palette_materials: MaxPaletteMaterials,

    #[command(flatten)]
    reduction_options: PaletteReductionOptions,

    #[command(flatten)]
    encoding_options: VoxjEncodingOptions,
}

impl Voxelize {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let resolution = self.resolution_options.resolve()?;

        self.validate_fill_color()?;

        let (format, output) = self
            .encoding_options
            .resolve_output(&self.input, self.output);

        let encoding = self.encoding_options.encoding();

        let color_format = self.encoding_options.color_format();

        let reduction = self.reduction_options.resolve(self.max_palette_materials);

        dependencies.voxelize(
            &self.input,
            self.from,
            &output,
            resolution,
            self.fill_mode,
            self.material_mode,
            self.fill_color,
            self.name.as_deref(),
            reduction,
            encoding,
            format,
            color_format,
        )
    }

    /// Rejects a `--fill-color` that a sampling-mode surface shell would drop.
    fn validate_fill_color(&self) -> Result<()> {
        if self.fill_color.is_some()
            && self.fill_mode == FillMode::Surface
            && self.material_mode != MaterialMode::Flat
        {
            return Err(usage(
                "--fill-color has no effect with --fill-mode surface and a sampling \
                 --material-mode; it applies under --material-mode flat or --fill-mode solid",
            ));
        }

        Ok(())
    }
}

/// A usage error for a rule clap cannot express, exiting non-zero with a
/// message.
fn usage(message: &str) -> crate::Error {
    IOError::new(ErrorKind::InvalidInput, message).into()
}

/// Parses a `#RRGGBB` or `#RRGGBBAA` hex `--fill-color` into straight RGBA
/// bytes, a missing alpha defaulting to opaque.
fn parse_hex_rgba(value: &str) -> StdResult<[u8; 4], String> {
    let hex = value
        .strip_prefix('#')
        .filter(|hex| hex.len() == 6 || hex.len() == 8)
        .ok_or_else(|| format!("`{value}` is not a color; use a #RRGGBB or #RRGGBBAA hex"))?;

    let byte = |index: usize| {
        hex.get(index * 2..index * 2 + 2)
            .and_then(|pair| u8::from_str_radix(pair, 16).ok())
            .ok_or_else(|| format!("`{value}` is not a valid hex color"))
    };

    Ok([
        byte(0)?,
        byte(1)?,
        byte(2)?,
        if hex.len() == 8 { byte(3)? } else { 255 },
    ])
}

#[cfg(test)]
mod tests {
    use super::Voxelize;
    use clap::Parser;

    /// Parses a `voxelize` invocation with a valid resolution already set, so a
    /// test only supplies the flags it exercises.
    fn parse(args: &[&str]) -> Voxelize {
        let mut argv = vec!["voxelize", "model.glb", "--resolution", "32"];
        argv.extend_from_slice(args);
        Voxelize::try_parse_from(argv).unwrap()
    }

    #[test]
    fn a_fill_color_is_rejected_on_a_sampled_surface() {
        let voxelize = parse(&[
            "--fill-mode",
            "surface",
            "--material-mode",
            "per-texel",
            "--fill-color",
            "#ff0000",
        ]);
        assert!(voxelize.validate_fill_color().is_err());
    }

    #[test]
    fn a_flat_surface_accepts_a_fill_color() {
        let voxelize = parse(&[
            "--fill-mode",
            "surface",
            "--material-mode",
            "flat",
            "--fill-color",
            "#ff0000",
        ]);
        assert!(voxelize.validate_fill_color().is_ok());
    }

    #[test]
    fn a_solid_body_accepts_a_fill_color() {
        let voxelize = parse(&[
            "--fill-mode",
            "solid",
            "--material-mode",
            "per-texel",
            "--fill-color",
            "#ff0000",
        ]);
        assert!(voxelize.validate_fill_color().is_ok());
    }

    #[test]
    fn a_sampled_surface_without_a_fill_color_is_fine() {
        let voxelize = parse(&["--fill-mode", "surface", "--material-mode", "per-texel"]);
        assert!(voxelize.validate_fill_color().is_ok());
    }

    #[test]
    fn an_omitted_fill_color_is_none() {
        assert_eq!(parse(&[]).fill_color, None);
    }

    #[test]
    fn a_hex_fill_color_parses_to_rgba() {
        // Six digits default the alpha to opaque; eight carry it through.
        assert_eq!(super::parse_hex_rgba("#0A0B0C"), Ok([10, 11, 12, 255]));
        assert_eq!(super::parse_hex_rgba("#0A0B0C0D"), Ok([10, 11, 12, 13]));
    }

    #[test]
    fn an_invalid_fill_color_is_rejected() {
        // A color name, a missing `#`, a bad length, and non-hex digits all fail.
        assert!(super::parse_hex_rgba("white").is_err());
        assert!(super::parse_hex_rgba("ff0000").is_err());
        assert!(super::parse_hex_rgba("#12345").is_err());
        assert!(super::parse_hex_rgba("#GG0000").is_err());
    }
}
