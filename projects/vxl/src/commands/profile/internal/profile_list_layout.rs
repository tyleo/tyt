use crate::CliValue;
use voxsmith::operations::profile::ProfileListLayout;

impl CliValue for ProfileListLayout {
    const VARIANTS: &'static [Self] = &[
        ProfileListLayout::BoxHierarchy,
        ProfileListLayout::BoxTables,
        ProfileListLayout::JsonCompact,
        ProfileListLayout::JsonPretty,
        ProfileListLayout::MdLists,
        ProfileListLayout::MdTables,
        ProfileListLayout::TextRows,
    ];

    fn name(self) -> &'static str {
        match self {
            ProfileListLayout::BoxHierarchy => "box-hierarchy",
            ProfileListLayout::BoxTables => "box-tables",
            ProfileListLayout::JsonCompact => "json-compact",
            ProfileListLayout::JsonPretty => "json-pretty",
            ProfileListLayout::MdLists => "md-lists",
            ProfileListLayout::MdTables => "md-tables",
            ProfileListLayout::TextRows => "text-rows",
        }
    }

    fn help(self) -> &'static str {
        match self {
            ProfileListLayout::BoxHierarchy => {
                "A box-glyph tree, one branch per origin over its profiles"
            }

            ProfileListLayout::BoxTables => {
                "One box-glyph table, a column per origin over its profiles"
            }

            ProfileListLayout::JsonCompact => "Compact, single-line JSON",

            ProfileListLayout::JsonPretty => "Pretty-printed, multi-line JSON",

            ProfileListLayout::MdLists => {
                "A `# profiles` heading over one section per origin, each a numbered Markdown \
                 list of its profiles"
            }

            ProfileListLayout::MdTables => {
                "One Markdown table, a column per origin over its profiles"
            }

            ProfileListLayout::TextRows => "One row per origin, its profiles beside it",
        }
    }
}
