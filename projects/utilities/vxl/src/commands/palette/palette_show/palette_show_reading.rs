use clap::ValueEnum;

/// How a `palette show` collection's numbers spell: whether the sRGB
/// transfer applies, and hex versus numbers. The three sRGB-named readings
/// are the color assertion and apply only to `vec-3-float` and `vec-4-float`
/// value pools; alpha never transfer-encodes.
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum PaletteShowReading {
    /// By key: a glTF vocabulary color name reads `srgb-hex`, or
    /// `linear-float` when a stored component leaves `[0, 1]`; everything
    /// else reads `plain`.
    #[value(name = "auto")]
    Auto,

    /// The stored linear values, `lin_srgb(...)` / `lin_srgba(...)` for a
    /// whole vector and the stored float for a component.
    #[value(name = "linear-float")]
    LinearFloat,

    /// The stored value as it is, arrays and text included.
    #[value(name = "plain")]
    Plain,

    /// Transfer-encoded floats, `srgb(...)` / `srgba(...)` for a whole
    /// vector and the encoded float for a component; alpha passes through.
    /// A component outside `[0, 1]` errors.
    #[value(name = "srgb-float")]
    SrgbFloat,

    /// Transfer-encoded 8-bit hex, `#RRGGBB` / `#RRGGBBAA` for a whole
    /// vector and the two-digit hex pair for a component; alpha quantizes
    /// raw. A component outside `[0, 1]` errors.
    #[value(name = "srgb-hex")]
    SrgbHex,
}
