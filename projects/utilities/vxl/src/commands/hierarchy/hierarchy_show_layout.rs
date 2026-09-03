use crate::CliValue;
use voxsmith::HierarchyShowLayout;

impl CliValue for HierarchyShowLayout {
    const VARIANTS: &'static [Self] = &[
        HierarchyShowLayout::Hierarchy,
        HierarchyShowLayout::JsonPretty,
        HierarchyShowLayout::JsonCompact,
    ];

    fn name(self) -> &'static str {
        match self {
            HierarchyShowLayout::Hierarchy => "hierarchy",
            HierarchyShowLayout::JsonPretty => "json-pretty",
            HierarchyShowLayout::JsonCompact => "json-compact",
        }
    }

    fn help(self) -> &'static str {
        match self {
            HierarchyShowLayout::Hierarchy => {
                "The scene graph as a box-glyph tree, each entity's tag and view rows inline on \
                 its nodes"
            }
            HierarchyShowLayout::JsonPretty => "The scene graph as indented JSON records",
            HierarchyShowLayout::JsonCompact => "The scene graph as single-line JSON records",
        }
    }
}
