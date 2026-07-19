/// How a node's values render to cells in the text layouts.
///
/// An unset [`TreeGridNode::format`](crate::TreeGridNode::format)
/// defers to the grid's cell policy per value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeGridCellFormat {
    /// The bare visual, text for a value without one. The strip
    /// format: a node whose every cell is a bare visual joins them
    /// with no separator.
    Visual,

    /// The visual beside the text, text alone for a value without
    /// one.
    VisualText,

    /// Text alone.
    Text,
}
