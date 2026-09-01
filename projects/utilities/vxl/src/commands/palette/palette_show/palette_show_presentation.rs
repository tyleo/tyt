use crate::CliValue;
use voxsmith::PaletteShowPresentation;

impl CliValue for PaletteShowPresentation {
    const VARIANTS: &'static [Self] = &[
        PaletteShowPresentation::Auto,
        PaletteShowPresentation::Swatch,
        PaletteShowPresentation::SwatchValue,
        PaletteShowPresentation::Value,
    ];

    fn name(self) -> &'static str {
        match self {
            PaletteShowPresentation::Auto => "auto",
            PaletteShowPresentation::Swatch => "swatch",
            PaletteShowPresentation::SwatchValue => "swatch-value",
            PaletteShowPresentation::Value => "value",
        }
    }

    fn help(self) -> &'static str {
        match self {
            PaletteShowPresentation::Auto => {
                "A swatch beside the text for a whole color, else bare text"
            }
            PaletteShowPresentation::Swatch => "Swatches alone, with no value text",
            PaletteShowPresentation::SwatchValue => "Each swatch followed by its value text",
            PaletteShowPresentation::Value => "Value text alone, one per line",
        }
    }
}
