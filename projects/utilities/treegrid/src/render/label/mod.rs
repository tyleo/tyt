mod data_paths;
mod group;
mod groups;
mod heading;
// Tables pad through markdown_table, so pad_right keeps the narrower
// union.
#[cfg(any(feature = "render_columns", feature = "render_rows"))]
mod pad_right;

pub(crate) use group::*;
pub(crate) use heading::*;
#[cfg(any(feature = "render_columns", feature = "render_rows"))]
pub(crate) use pad_right::*;
