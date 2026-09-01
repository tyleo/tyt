use crate::CliValue;
use voxsmith::PaletteShowTableShape;

impl CliValue for PaletteShowTableShape {
    const VARIANTS: &'static [Self] = &[
        PaletteShowTableShape::Nested,
        PaletteShowTableShape::Flat,
        PaletteShowTableShape::Records,
    ];

    fn name(self) -> &'static str {
        match self {
            PaletteShowTableShape::Nested => "nested",
            PaletteShowTableShape::Flat => "flat",
            PaletteShowTableShape::Records => "records",
        }
    }

    fn help(self) -> &'static str {
        match self {
            PaletteShowTableShape::Nested => "One table per palette group, under nested headings",
            PaletteShowTableShape::Flat => {
                "One table over every value collection, the cross-palette comparison view"
            }
            PaletteShowTableShape::Records => {
                "One row per property under each palette's heading, component values in \
                 relative-path columns"
            }
        }
    }
}
