use clap::ValueEnum;

/// How a written voxj document encodes its color value pools: color space and
/// numeric encoding in one flag. This is a write-time serialization choice;
/// `--define-attribute` declares a pool's color kind when authoring. A
/// linear-kind pool serializes as float whichever value is chosen.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum VoxjColorFormat {
    /// Emit sRGB colors as `srgb-float` / `srgba-float` components in `[0, 1]`,
    /// so a color round-trips without 8-bit hex quantization.
    #[value(name = "float")]
    Float,

    /// Emit sRGB colors as `srgb-hex` / `srgba-hex` `#RRGGBB` / `#RRGGBBAA`
    /// strings, the human-editable form, quantizing each component to 8 bits.
    #[value(name = "hex")]
    Hex,

    /// Decode sRGB colors to linear light and emit the `linear-rgb-float` /
    /// `linear-rgba-float` kinds. No hex counterpart; always float.
    #[value(name = "linear-float")]
    LinearFloat,
}
