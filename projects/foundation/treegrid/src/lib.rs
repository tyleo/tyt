#![deny(rustdoc::broken_intra_doc_links)]

//! Hierarchical-data rendering: populate a forest of labeled,
//! data-bearing nodes, then render it under a chosen layout and label
//! mode.

// Public API

mod b_tree_grid_node;
mod tree_grid;
mod tree_grid_cell_format;
mod tree_grid_cells;
mod tree_grid_error;
mod tree_grid_header_options;
mod tree_grid_label;
mod tree_grid_label_kind;
mod tree_grid_label_mode;
mod tree_grid_node;
mod tree_grid_options;
mod tree_grid_table_shape_kind;
mod tree_grid_visual;
mod value;

pub use b_tree_grid_node::*;
pub use tree_grid::*;
pub use tree_grid_cell_format::*;
pub use tree_grid_cells::*;
pub use tree_grid_error::*;
pub use tree_grid_header_options::*;
pub use tree_grid_label::*;
pub use tree_grid_label_kind::*;
pub use tree_grid_label_mode::*;
pub use tree_grid_node::*;
pub use tree_grid_options::*;
pub use tree_grid_table_shape_kind::*;
pub use tree_grid_visual::*;
pub use value::*;

// Optional API

#[cfg(feature = "ty-math")]
mod color;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "json")]
pub use json::*;

#[cfg(any(
    feature = "render_box_hierarchy",
    feature = "render_box_tables",
    feature = "render_text_columns",
    feature = "render_md_lists",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
mod render;

#[cfg(feature = "render_box_hierarchy")]
mod render_box_hierarchy;

#[cfg(feature = "render_box_hierarchy")]
pub use render_box_hierarchy::*;

#[cfg(feature = "render_box_tables")]
mod render_box_tables;

#[cfg(feature = "render_box_tables")]
pub use render_box_tables::*;

#[cfg(feature = "render_md_lists")]
mod render_md_lists;

#[cfg(feature = "render_md_lists")]
pub use render_md_lists::*;

#[cfg(feature = "render_md_tables")]
mod render_md_tables;

#[cfg(feature = "render_md_tables")]
pub use render_md_tables::*;

#[cfg(feature = "render_text_columns")]
mod render_text_columns;

#[cfg(feature = "render_text_columns")]
pub use render_text_columns::*;

#[cfg(feature = "render_text_rows")]
mod render_text_rows;

#[cfg(feature = "render_text_rows")]
pub use render_text_rows::*;

#[cfg(any(feature = "render_box_tables", feature = "render_md_tables"))]
mod tree_grid_table_label_mode;

#[cfg(any(feature = "render_box_tables", feature = "render_md_tables"))]
pub use tree_grid_table_label_mode::*;
