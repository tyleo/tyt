// Public API

#[allow(clippy::module_inception)]
mod tree_grid_options;

pub use tree_grid_options::*;

// Optional API

#[cfg(any(
    feature = "render_text_columns",
    feature = "render_md_lists",
    feature = "render_text_rows",
    feature = "render_md_tables"
))]
mod level;

#[cfg(any(
    feature = "json",
    feature = "render_box_tables",
    feature = "render_text_columns",
    feature = "render_md_lists",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
mod no_box_hierarchy_options;

#[cfg(any(
    feature = "json",
    feature = "render_box_hierarchy",
    feature = "render_box_tables",
    feature = "render_text_columns",
    feature = "render_md_lists",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
mod no_header_level;

#[cfg(any(feature = "render_box_hierarchy", feature = "json"))]
mod no_label;

#[cfg(any(
    feature = "render_text_columns",
    feature = "render_box_hierarchy",
    feature = "json",
    feature = "render_md_lists",
    feature = "render_text_rows"
))]
mod no_table_shape;

#[cfg(any(
    feature = "json",
    feature = "render_box_hierarchy",
    feature = "render_box_tables",
    feature = "render_text_columns",
    feature = "render_md_lists",
    feature = "render_md_tables"
))]
mod no_width;

#[cfg(any(feature = "render_text_columns", feature = "render_text_rows"))]
mod text_label;
