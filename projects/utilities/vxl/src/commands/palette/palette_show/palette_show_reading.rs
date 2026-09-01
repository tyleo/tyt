use crate::CliValue;
use voxsmith::PaletteShowReading;

impl CliValue for PaletteShowReading {
    const VARIANTS: &'static [Self] = &[
        PaletteShowReading::Auto,
        PaletteShowReading::LinearFloat,
        PaletteShowReading::Plain,
        PaletteShowReading::SrgbFloat,
        PaletteShowReading::SrgbHex,
    ];

    fn name(self) -> &'static str {
        match self {
            PaletteShowReading::Auto => "auto",
            PaletteShowReading::LinearFloat => "linear-float",
            PaletteShowReading::Plain => "plain",
            PaletteShowReading::SrgbFloat => "srgb-float",
            PaletteShowReading::SrgbHex => "srgb-hex",
        }
    }

    fn help(self) -> &'static str {
        match self {
            PaletteShowReading::Auto => {
                "By key: a glTF vocabulary color name reads `srgb-hex`, so a shape or component \
                 outside the vocabulary's standards errors and needs an explicit reading. \
                 Everything else assumes `plain`"
            }
            PaletteShowReading::LinearFloat => {
                "The stored linear values, `lin_srgb(...)` / `lin_srgba(...)` for a whole vector \
                 and the stored float for a component"
            }
            PaletteShowReading::Plain => "The stored value as it is, arrays and text included",
            PaletteShowReading::SrgbFloat => {
                "Transfer-encoded floats, `srgb(...)` / `srgba(...)` for a whole vector and the \
                 encoded float for a component. Alpha passes through. A component outside \
                 `[0, 1]` errors"
            }
            PaletteShowReading::SrgbHex => {
                "Transfer-encoded 8-bit hex, `#RRGGBB` / `#RRGGBBAA` for a whole vector and the \
                 two-digit hex pair for a component. Alpha quantizes raw. A component outside \
                 `[0, 1]` errors"
            }
        }
    }
}
