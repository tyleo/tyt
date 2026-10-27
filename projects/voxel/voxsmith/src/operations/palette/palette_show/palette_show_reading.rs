/// How a value collection's numbers spell: whether the sRGB transfer
/// applies, and hex versus numbers. The three color readings are the color
/// assertion and apply only to `vec-3-float` and `vec-4-float` value pools.
/// Alpha never transfer-encodes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PaletteShowReading {
    /// By key: a material vocabulary color name reads `SrgbHex`, so a shape
    /// or component outside the vocabulary's standards errors and needs an
    /// explicit reading. Everything else reads `Plain`.
    #[default]
    Auto,

    /// The stored linear values, `lin_srgb(...)` / `lin_srgba(...)` for a
    /// whole vector and the stored float for a component.
    LinearFloat,

    /// The stored value as it is, arrays and text included.
    Plain,

    /// Transfer-encoded floats, `srgb(...)` / `srgba(...)` for a whole vector
    /// and the encoded float for a component. Alpha passes through. A
    /// component outside `[0, 1]` errors.
    SrgbFloat,

    /// Transfer-encoded 8-bit hex, `#RRGGBB` / `#RRGGBBAA` for a whole vector
    /// and the two-digit hex pair for a component. Alpha quantizes raw. A
    /// component outside `[0, 1]` errors.
    SrgbHex,
}
