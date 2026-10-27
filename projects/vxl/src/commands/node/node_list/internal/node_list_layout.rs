use crate::CliValue;
use voxsmith::operations::node::NodeListLayout;

impl CliValue for NodeListLayout {
    const VARIANTS: &'static [Self] = &[
        NodeListLayout::BoxHierarchy,
        NodeListLayout::JsonCompact,
        NodeListLayout::JsonPretty,
    ];

    fn name(self) -> &'static str {
        match self {
            NodeListLayout::BoxHierarchy => "box-hierarchy",
            NodeListLayout::JsonCompact => "json-compact",
            NodeListLayout::JsonPretty => "json-pretty",
        }
    }

    fn help(self) -> &'static str {
        match self {
            NodeListLayout::BoxHierarchy => {
                "The scene graph as a box-glyph tree, each entity's tag and view rows inline on \
                 its nodes"
            }

            NodeListLayout::JsonCompact => "The scene graph as compact, single-line JSON records",

            NodeListLayout::JsonPretty => {
                "The scene graph as pretty-printed, multi-line JSON records"
            }
        }
    }
}
