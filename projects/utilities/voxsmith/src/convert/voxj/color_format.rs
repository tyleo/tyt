/// How a Voxel Json writer encodes an sRGB color value pool on the wire.
///
/// voxcore stores every color as float components, so this only picks the
/// on-wire encoding for the sRGB color kinds, which the format offers in both a
/// hex and a float form. Linear colors have no hex form and always serialize as
/// float regardless of this choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorFormat {
    /// Emit sRGB colors as `srgb-float` / `srgba-float` float components, the
    /// default, so a color round-trips without 8-bit hex quantization.
    Float,

    /// Emit sRGB colors as `srgb-hex` / `srgba-hex` `#RRGGBB` / `#RRGGBBAA`
    /// strings, the human-editable form. Each component is quantized to 8 bits.
    Hex,
}
