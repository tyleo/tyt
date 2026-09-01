use crate::CliValue;
use voxsmith::PaletteListLayout;

impl CliValue for PaletteListLayout {
    const VARIANTS: &'static [Self] = &[
        PaletteListLayout::Hierarchy,
        PaletteListLayout::Tables,
        PaletteListLayout::JsonPretty,
        PaletteListLayout::JsonCompact,
    ];

    fn name(self) -> &'static str {
        match self {
            PaletteListLayout::Hierarchy => "hierarchy",
            PaletteListLayout::Tables => "tables",
            PaletteListLayout::JsonPretty => "json-pretty",
            PaletteListLayout::JsonCompact => "json-compact",
        }
    }

    fn help(self) -> &'static str {
        match self {
            PaletteListLayout::Hierarchy => {
                "Indented tree, one palette per branch, like `hierarchy show`"
            }
            PaletteListLayout::Tables => {
                "A `# palettes` heading over one aligned record table, one row per palette"
            }
            PaletteListLayout::JsonPretty => "Pretty-printed, multi-line JSON",
            PaletteListLayout::JsonCompact => "Compact, single-line JSON",
        }
    }
}
