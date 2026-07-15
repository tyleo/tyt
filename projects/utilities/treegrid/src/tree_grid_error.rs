use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An invalid flag combination, reported by
/// [`TreeGridOptions::resolve`](crate::TreeGridOptions::resolve).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeGridError {
    /// The `tables` layout cannot head its columns with nothing; label
    /// mode `none` is invalid there.
    LabelNoneWithTables,

    /// A label mode was set, but the layout carries its labels
    /// structurally and takes no mode.
    LabelModeWithoutLabels,

    /// `header_level` was set on a render that emits no headings.
    HeaderLevelWithoutHeaders,

    /// The flat table shape heads its columns with full paths; the
    /// `header` label mode is invalid there.
    HeaderLabelWithFlatTables,

    /// A table shape was set on a layout other than `tables`.
    TableShapeWithoutTables,

    /// Bare roots were requested on a layout other than `hierarchy`.
    BareRootsWithoutHierarchy,

    /// Value children were requested on a layout other than
    /// `hierarchy`.
    ValueChildrenWithoutHierarchy,

    /// A width was set on a layout other than `rows`.
    WidthWithoutRows,
}

impl Display for TreeGridError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TreeGridError::LabelNoneWithTables => {
                write!(
                    f,
                    "the tables layout requires labels, but the label mode is none"
                )
            }
            TreeGridError::LabelModeWithoutLabels => {
                write!(
                    f,
                    "a label mode was set, but the layout carries its labels structurally"
                )
            }
            TreeGridError::HeaderLevelWithoutHeaders => {
                write!(
                    f,
                    "a header level was set, but the render emits no headings"
                )
            }
            TreeGridError::HeaderLabelWithFlatTables => {
                write!(f, "the flat table shape requires the concat label mode")
            }
            TreeGridError::TableShapeWithoutTables => {
                write!(f, "a table shape was set, but the layout is not tables")
            }
            TreeGridError::BareRootsWithoutHierarchy => {
                write!(
                    f,
                    "bare roots were requested, but the layout is not hierarchy"
                )
            }
            TreeGridError::ValueChildrenWithoutHierarchy => {
                write!(
                    f,
                    "value children were requested, but the layout is not hierarchy"
                )
            }
            TreeGridError::WidthWithoutRows => {
                write!(f, "a width was set, but the layout is not rows")
            }
        }
    }
}

impl StdError for TreeGridError {}
