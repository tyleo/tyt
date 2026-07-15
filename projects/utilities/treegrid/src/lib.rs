#![deny(rustdoc::broken_intra_doc_links)]

//! Hierarchical-data rendering: populate a forest of labeled,
//! data-bearing nodes, then render it under a chosen layout and label
//! mode.

mod b_tree_grid_node;
mod tree_grid;
mod tree_grid_cell_format;
mod tree_grid_error;
mod tree_grid_label;
mod tree_grid_label_mode;
mod tree_grid_layout;
mod tree_grid_node;
mod tree_grid_options;
mod tree_grid_swatch;
mod tree_grid_value;

pub use b_tree_grid_node::*;
pub use tree_grid::*;
pub use tree_grid_cell_format::*;
pub use tree_grid_error::*;
pub use tree_grid_label::*;
pub use tree_grid_label_mode::*;
pub use tree_grid_layout::*;
pub use tree_grid_node::*;
pub use tree_grid_options::*;
pub use tree_grid_swatch::*;
pub use tree_grid_value::*;
