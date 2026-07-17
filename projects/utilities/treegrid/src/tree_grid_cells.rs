use crate::{TreeGridCellFormat, TreeGridVisual};
use std::borrow::Cow;

/// The cell policy of a [`TreeGrid`](crate::TreeGrid): how the grid's
/// value type renders to cells in the text layouts.
///
/// A grid stores values of the associated [`Value`](Self::Value) type
/// and the renders ask the policy for each value's pieces, so any
/// value type renders, including one the policy's author does not
/// own. The default policy,
/// [`TreeGridValueCells`](crate::TreeGridValueCells), reads
/// pre-rendered [`TreeGridValue`](crate::TreeGridValue)s; a text-only
/// policy implements [`text`](Self::text) alone.
pub trait TreeGridCells {
    /// The value type the grid stores.
    type Value;

    /// The value's cell text.
    fn text<'a>(&self, value: &'a Self::Value) -> Cow<'a, str>;

    /// The value's non-text cell content, shown by the
    /// visual-rendering cell formats. Defaults to none.
    fn visual(&self, _value: &Self::Value) -> Option<TreeGridVisual> {
        None
    }

    /// The format a node with no explicit format uses for this value.
    /// Defaults to [`TreeGridCellFormat::Text`].
    fn format(&self, _value: &Self::Value) -> TreeGridCellFormat {
        TreeGridCellFormat::Text
    }
}
