/// How a Voxel Json writer encodes a color value pool: color space and numeric
/// encoding together. Only the two sRGB kinds vary; a linear-kind value pool
/// serializes as float under every choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorFormat {
    /// Emit sRGB colors as `srgb-float` / `srgba-float` float components, the
    /// default, so a color round-trips without 8-bit hex quantization.
    Float,

    /// Emit sRGB colors as `srgb-hex` / `srgba-hex` `#RRGGBB` / `#RRGGBBAA`
    /// strings, the human-editable form. Each component is quantized to 8 bits.
    Hex,

    /// Decode sRGB colors to linear light and emit the `linear-rgb-float` /
    /// `linear-rgba-float` kinds.
    LinearFloat,
}
