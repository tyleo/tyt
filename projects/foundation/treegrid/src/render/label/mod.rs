// Optional API

// Box tables head with bare lines, so heading keeps the markdown union.
#[cfg(any(
    feature = "render_text_columns",
    feature = "render_md_lists",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
mod heading;

#[cfg(any(
    feature = "render_text_columns",
    feature = "render_md_lists",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
pub(crate) use heading::*;

// The label-mode walks keep the narrower union because lists walk the tree
// directly.
#[cfg(any(
    feature = "render_box_tables",
    feature = "render_text_columns",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
mod mode;

#[cfg(any(
    feature = "render_box_tables",
    feature = "render_text_columns",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
pub(crate) use mode::*;

// Tables pad through their frames, so pad_right keeps the narrower union.
#[cfg(any(feature = "render_text_columns", feature = "render_text_rows"))]
mod pad_right;

#[cfg(any(feature = "render_text_columns", feature = "render_text_rows"))]
pub(crate) use pad_right::*;

// Internal API

mod tree_grid_label_ext;
