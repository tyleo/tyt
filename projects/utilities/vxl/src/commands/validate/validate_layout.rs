use crate::CliValue;
use voxsmith::ValidateLayout;

impl CliValue for ValidateLayout {
    const VARIANTS: &'static [Self] = &[
        ValidateLayout::Tables,
        ValidateLayout::JsonPretty,
        ValidateLayout::JsonCompact,
    ];

    fn name(self) -> &'static str {
        match self {
            ValidateLayout::Tables => "tables",
            ValidateLayout::JsonPretty => "json-pretty",
            ValidateLayout::JsonCompact => "json-compact",
        }
    }

    fn help(self) -> &'static str {
        match self {
            ValidateLayout::Tables => {
                "A file-name heading over one line per check and a closing pass/fail summary"
            }
            ValidateLayout::JsonPretty => "Pretty-printed, multi-line JSON",
            ValidateLayout::JsonCompact => "Compact, single-line JSON",
        }
    }
}
