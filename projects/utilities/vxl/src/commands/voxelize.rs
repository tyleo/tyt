use crate::{Dependencies, FillMode, MeshFormat, Result, VoxjEncodingOptions};
use clap::{ArgGroup, Parser};
use std::{
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};

/// Rasterizes a mesh into a voxel grid, the inverse of `mesh`.
#[derive(Clone, Debug, Parser)]
#[command(
    name = "voxelize",
    group(ArgGroup::new("resolution").required(true).args(["side_length", "scale"]))
)]
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

    /// Grid resolution in voxels along the longest axis; the other axes are
    /// sized to preserve aspect. Mutually exclusive with `--scale`.
    #[arg(value_name = "side-length", long)]
    side_length: Option<u32>,

    /// Edge length of one voxel in meters; each axis count is the mesh extent on
    /// that axis divided by this, rounded up. Recorded as the placing node's
    /// scale so the model keeps its source size. Mutually exclusive with
    /// `--side-length`.
    #[arg(value_name = "scale", long)]
    scale: Option<f64>,

    /// How the mesh fills the grid.
    #[arg(value_name = "fill-mode", long, default_value = "solid")]
    fill_mode: FillMode,

    /// Color of every voxel under `--fill-mode solid`, a `#RRGGBBAA` hex or a
    /// name like `white` (the default). Rejected with `--fill-mode surface`,
    /// which samples the mesh's own color.
    #[arg(value_name = "fill-color", long, value_parser = parse_fill_color)]
    fill_color: Option<[u8; 4]>,

    #[command(flatten)]
    encoding_options: VoxjEncodingOptions,
}

impl Voxelize {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        if self.fill_mode == FillMode::Surface && self.fill_color.is_some() {
            return Err(usage(
                "--fill-color cannot be used with --fill-mode surface, which samples the mesh's own color",
            ));
        }
        if let Some(meters) = self.scale
            && (meters <= 0.0 || meters.is_nan())
        {
            return Err(usage("--scale must be greater than 0"));
        }
        if self.side_length == Some(0) {
            return Err(usage("--side-length must be at least 1"));
        }
        let fill_color = self.fill_color.unwrap_or([255, 255, 255, 255]);

        let format = self.encoding_options.resolve_format(self.output.as_deref());
        let output = self
            .output
            .unwrap_or_else(|| self.input.with_extension(format.extension()));
        let encoding = self.encoding_options.encoding();

        dependencies.voxelize(
            &self.input,
            self.from,
            &output,
            self.side_length,
            self.scale,
            self.fill_mode,
            fill_color,
            encoding,
            format,
        )
    }
}

/// A usage error for a rule clap cannot express, exiting non-zero with a
/// message.
fn usage(message: &str) -> crate::Error {
    IOError::new(ErrorKind::InvalidInput, message).into()
}

/// Parses a `--fill-color` value: a `#RRGGBB` or `#RRGGBBAA` hex, or a color
/// name, into straight RGBA bytes.
fn parse_fill_color(value: &str) -> std::result::Result<[u8; 4], String> {
    if let Some(hex) = value.strip_prefix('#') {
        return parse_hex(hex);
    }
    match value.to_ascii_lowercase().as_str() {
        "white" => Ok([255, 255, 255, 255]),
        "black" => Ok([0, 0, 0, 255]),
        "red" => Ok([255, 0, 0, 255]),
        "green" => Ok([0, 128, 0, 255]),
        "blue" => Ok([0, 0, 255, 255]),
        "yellow" => Ok([255, 255, 0, 255]),
        "cyan" => Ok([0, 255, 255, 255]),
        "magenta" => Ok([255, 0, 255, 255]),
        "gray" | "grey" => Ok([128, 128, 128, 255]),
        other => Err(format!(
            "unknown color \"{other}\"; use a #RRGGBB or #RRGGBBAA hex or a name like white"
        )),
    }
}

/// Parses the digits after `#` of a `RRGGBB` or `RRGGBBAA` hex color, defaulting
/// a missing alpha to opaque.
fn parse_hex(hex: &str) -> std::result::Result<[u8; 4], String> {
    let invalid = || format!("invalid color \"#{hex}\"; expected #RRGGBB or #RRGGBBAA");
    if (hex.len() != 6 && hex.len() != 8) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid());
    }
    let byte = |index: usize| {
        u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).map_err(|_| invalid())
    };
    let alpha = if hex.len() == 8 { byte(3)? } else { 255 };
    Ok([byte(0)?, byte(1)?, byte(2)?, alpha])
}

#[cfg(test)]
mod tests {
    use super::parse_fill_color;

    #[test]
    fn parses_a_six_digit_hex_as_opaque() {
        assert_eq!(parse_fill_color("#1A2B3C"), Ok([0x1A, 0x2B, 0x3C, 0xFF]));
    }

    #[test]
    fn parses_an_eight_digit_hex_with_alpha() {
        assert_eq!(parse_fill_color("#1A2B3C4D"), Ok([0x1A, 0x2B, 0x3C, 0x4D]));
    }

    #[test]
    fn parses_a_color_name_case_insensitively() {
        assert_eq!(parse_fill_color("white"), Ok([255, 255, 255, 255]));
        assert_eq!(parse_fill_color("Red"), Ok([255, 0, 0, 255]));
    }

    #[test]
    fn rejects_a_bad_length_or_non_hex() {
        assert!(parse_fill_color("#12345").is_err());
        assert!(parse_fill_color("#GG0000").is_err());
        assert!(parse_fill_color("chartreuse").is_err());
    }
}
