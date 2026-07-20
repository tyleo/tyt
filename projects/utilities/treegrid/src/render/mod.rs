//! Machinery shared by two or more layout renders, grouped by feature
//! gate so each `cfg` sits once, on the module that rides it.

mod cell;
#[cfg(any(
    feature = "columns",
    feature = "hierarchy",
    feature = "rows",
    feature = "tables"
))]
mod cell_render;
#[cfg(any(feature = "hierarchy", feature = "rows"))]
mod cell_separator;
#[cfg(any(feature = "columns", feature = "rows", feature = "tables"))]
mod label;
mod visible_width;

pub(crate) use cell::*;
#[cfg(any(feature = "columns", feature = "rows", feature = "tables"))]
pub(crate) use label::*;
pub(crate) use visible_width::*;
