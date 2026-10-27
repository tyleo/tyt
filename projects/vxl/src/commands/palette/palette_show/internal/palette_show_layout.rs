use crate::CliValue;
use voxsmith::operations::palette::PaletteShowLayout;

impl CliValue for PaletteShowLayout {
    const VARIANTS: &'static [Self] = &[
        PaletteShowLayout::BoxHierarchy,
        PaletteShowLayout::BoxTables,
        PaletteShowLayout::JsonCompact,
        PaletteShowLayout::JsonPretty,
        PaletteShowLayout::MdTables,
        PaletteShowLayout::TextColumns,
        PaletteShowLayout::TextRows,
    ];

    fn name(self) -> &'static str {
        match self {
            PaletteShowLayout::BoxHierarchy => "box-hierarchy",
            PaletteShowLayout::BoxTables => "box-tables",
            PaletteShowLayout::JsonCompact => "json-compact",
            PaletteShowLayout::JsonPretty => "json-pretty",
            PaletteShowLayout::MdTables => "md-tables",
            PaletteShowLayout::TextColumns => "text-columns",
            PaletteShowLayout::TextRows => "text-rows",
        }
    }

    fn help(self) -> &'static str {
        match self {
            PaletteShowLayout::BoxHierarchy => {
                "The value collections as a box-glyph tree of palettes, properties, and \
                 components, each value collection's values inline on its node"
            }

            PaletteShowLayout::BoxTables => {
                "The value collections as box-glyph tables led by a `#` column of 0-based \
                 material indices, each section headed by a bare full-path line; `--table-shape` \
                 picks the shape and the label mode stays `concat`"
            }

            PaletteShowLayout::JsonCompact => {
                "The value collection tree as compact, single-line JSON records"
            }

            PaletteShowLayout::JsonPretty => {
                "The value collection tree as pretty-printed, multi-line JSON records"
            }

            PaletteShowLayout::MdTables => {
                "The value collections as aligned Markdown tables led by a `#` column of 0-based \
                 material indices; `--table-shape` picks per-palette tables under headings or one \
                 flat comparison table"
            }

            PaletteShowLayout::TextColumns => {
                "Each value collection as its own column beneath its label, padded to a common \
                 width"
            }

            PaletteShowLayout::TextRows => {
                "Each value collection on one row, separated by a blank line, under labels padded \
                 to the longest so each row's first value aligns. Swatch cells abut into a strip; \
                 other formats put one space between cells"
            }
        }
    }
}
