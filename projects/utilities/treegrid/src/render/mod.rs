//! Machinery shared by two or more layout renders, grouped by feature
//! gate so each `cfg` sits once, on the module that rides it.

mod cell;
#[cfg(any(
    feature = "render_columns",
    feature = "render_hierarchy",
    feature = "render_rows",
    feature = "render_tables"
))]
mod cell_render;
#[cfg(any(feature = "render_hierarchy", feature = "render_rows"))]
mod cell_separator;
#[cfg(any(
    feature = "render_columns",
    feature = "render_rows",
    feature = "render_tables"
))]
mod label;
mod visible_width;

pub(crate) use cell::*;
#[cfg(any(
    feature = "render_columns",
    feature = "render_rows",
    feature = "render_tables"
))]
pub(crate) use label::*;
pub(crate) use visible_width::*;
