use clap::ValueEnum;

/// The on-wire encoding a written voxj document uses for its sRGB color value
/// pools. voxcore stores every color as float components, so this only picks the
/// encoding of the sRGB color kinds, which the format offers in a hex and a
/// float form. Linear colors have no hex form and always serialize as float.
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
}
