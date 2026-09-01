use crate::CliValue;
use voxsmith::PaletteShowLayout;

impl CliValue for PaletteShowLayout {
    const VARIANTS: &'static [Self] = &[
        PaletteShowLayout::Hierarchy,
        PaletteShowLayout::Rows,
        PaletteShowLayout::Columns,
        PaletteShowLayout::Tables,
        PaletteShowLayout::JsonPretty,
        PaletteShowLayout::JsonCompact,
    ];

    fn name(self) -> &'static str {
        match self {
            PaletteShowLayout::Hierarchy => "hierarchy",
            PaletteShowLayout::Rows => "rows",
            PaletteShowLayout::Columns => "columns",
            PaletteShowLayout::Tables => "tables",
            PaletteShowLayout::JsonPretty => "json-pretty",
            PaletteShowLayout::JsonCompact => "json-compact",
        }
    }

    fn help(self) -> &'static str {
        match self {
            PaletteShowLayout::Hierarchy => {
                "The value collections as a box-glyph tree of palettes, properties, and \
                 components, each value collection's values inline on its node"
            }
            PaletteShowLayout::Rows => {
                "Each value collection on one row, separated by a blank line, under labels padded \
                 to the longest so each row's first value aligns. Swatch cells abut into a strip; \
                 other formats put one space between cells"
            }
            PaletteShowLayout::Columns => {
                "Each value collection as its own column beneath its label, padded to a common \
                 width"
            }
            PaletteShowLayout::Tables => {
                "The value collections as aligned markdown tables led by a `#` column of 0-based \
                 material indices; `--table-shape` picks per-palette tables under headings or one \
                 flat comparison table"
            }
            PaletteShowLayout::JsonPretty => "The value collection tree as indented JSON records",
            PaletteShowLayout::JsonCompact => {
                "The value collection tree as single-line JSON records"
            }
        }
    }
}
