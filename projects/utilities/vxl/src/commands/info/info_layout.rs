use crate::CliValue;
use voxsmith::InfoLayout;

impl CliValue for InfoLayout {
    const VARIANTS: &'static [Self] = &[
        InfoLayout::Tables,
        InfoLayout::JsonPretty,
        InfoLayout::JsonCompact,
    ];

    fn name(self) -> &'static str {
        match self {
            InfoLayout::Tables => "tables",
            InfoLayout::JsonPretty => "json-pretty",
            InfoLayout::JsonCompact => "json-compact",
        }
    }

    fn help(self) -> &'static str {
        match self {
            InfoLayout::Tables => {
                "A file-name title over `Document`, `Palettes`, and `Objects` record tables"
            }
            InfoLayout::JsonPretty => "Pretty-printed, multi-line JSON",
            InfoLayout::JsonCompact => "Compact, single-line JSON",
        }
    }
}
