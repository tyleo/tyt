#![deny(rustdoc::broken_intra_doc_links)]

//! Hierarchical-data rendering: populate a forest of labeled,
//! data-bearing nodes, then render it under a chosen layout and label
//! mode.

mod b_tree_grid_node;
#[cfg(feature = "json")]
mod json;
mod render;
mod tree_grid;
mod tree_grid_cell_format;
mod tree_grid_cells;
mod tree_grid_columns_options;
mod tree_grid_error;
mod tree_grid_header_options;
mod tree_grid_hierarchy_options;
mod tree_grid_label;
mod tree_grid_label_kind;
mod tree_grid_label_mode;
mod tree_grid_nested_table_options;
mod tree_grid_node;
mod tree_grid_options;
mod tree_grid_rows_options;
mod tree_grid_table_label_mode;
mod tree_grid_table_shape;
mod tree_grid_table_shape_kind;
mod tree_grid_visual;
mod value;

pub use b_tree_grid_node::*;
#[cfg(feature = "json")]
pub use json::*;
pub use tree_grid::*;
pub use tree_grid_cell_format::*;
pub use tree_grid_cells::*;
pub use tree_grid_columns_options::*;
pub use tree_grid_error::*;
pub use tree_grid_header_options::*;
pub use tree_grid_hierarchy_options::*;
pub use tree_grid_label::*;
pub use tree_grid_label_kind::*;
pub use tree_grid_label_mode::*;
pub use tree_grid_nested_table_options::*;
pub use tree_grid_node::*;
pub use tree_grid_options::*;
pub use tree_grid_rows_options::*;
pub use tree_grid_table_label_mode::*;
pub use tree_grid_table_shape::*;
pub use tree_grid_table_shape_kind::*;
pub use tree_grid_visual::*;
pub use value::*;
