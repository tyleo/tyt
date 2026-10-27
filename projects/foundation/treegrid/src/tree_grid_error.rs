use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An invalid option combination, reported by the
/// [`TreeGridOptions`](crate::TreeGridOptions) `resolve_*` methods.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeGridError {
    /// Bare roots were requested on a layout other than `box-hierarchy`.
    BareRootsWithoutBoxHierarchy,

    /// Box table sections head with bare full-path lines; the `header`
    /// label mode is invalid there.
    HeaderLabelWithBoxTables,

    /// The flat table shape heads its columns with full paths; the
    /// `header` label mode is invalid there.
    HeaderLabelWithFlatTables,

    /// `header_level` was set on a render that emits no headings.
    HeaderLevelWithoutHeaders,

    /// A list has no inline label slot to carry a path; the `concat`
    /// label mode is invalid with `md-lists`.
    LabelConcatWithMdLists,

    /// A label mode was set, but the layout carries its labels
    /// structurally and takes no mode.
    LabelModeWithoutLabels,

    /// A table layout cannot head its columns with nothing; label mode
    /// `none` is invalid there.
    LabelNoneWithTables,

    /// A table shape was set on a layout other than `box-tables` or
    /// `md-tables`.
    TableShapeWithoutTables,

    /// Value children were requested on a layout other than
    /// `box-hierarchy`.
    ValueChildrenWithoutBoxHierarchy,

    /// A width was set on a layout other than `text-rows`.
    WidthWithoutTextRows,
}

impl Display for TreeGridError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TreeGridError::BareRootsWithoutBoxHierarchy => {
                write!(
                    f,
                    "bare roots were requested, but the layout is not box hierarchy"
                )
            }

            TreeGridError::HeaderLabelWithBoxTables => {
                write!(f, "the box tables layout takes the concat label mode")
            }

            TreeGridError::HeaderLabelWithFlatTables => {
                write!(f, "the flat table shape requires the concat label mode")
            }

            TreeGridError::HeaderLevelWithoutHeaders => {
                write!(
                    f,
                    "a header level was set, but the render emits no headings"
                )
            }

            TreeGridError::LabelConcatWithMdLists => {
                write!(
                    f,
                    "the markdown lists layout takes the header or none label mode"
                )
            }

            TreeGridError::LabelModeWithoutLabels => {
                write!(
                    f,
                    "a label mode was set, but the layout carries its labels structurally"
                )
            }

            TreeGridError::LabelNoneWithTables => {
                write!(
                    f,
                    "the table layouts require labels, but the label mode is none"
                )
            }

            TreeGridError::TableShapeWithoutTables => {
                write!(
                    f,
                    "a table shape was set, but the layout is not a table layout"
                )
            }

            TreeGridError::ValueChildrenWithoutBoxHierarchy => {
                write!(
                    f,
                    "value children were requested, but the layout is not box hierarchy"
                )
            }

            TreeGridError::WidthWithoutTextRows => {
                write!(f, "a width was set, but the layout is not text rows")
            }
        }
    }
}

impl StdError for TreeGridError {}
