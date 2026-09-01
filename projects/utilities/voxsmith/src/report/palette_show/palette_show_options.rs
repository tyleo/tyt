use crate::{PaletteShowLabel, PaletteShowLayout, PaletteShowTableShape};
use std::num::NonZeroU8;

/// How [`render_palette_show`](crate::render_palette_show) lays out its
/// output. Each layout errors on an option it ignores. The default is the
/// `Rows` layout with concat labels, headings from `#`, nested tables, and
/// no wrapping.
#[derive(Clone, Copy, Debug, Default)]
pub struct PaletteShowOptions {
    /// How to arrange the value collections, and the serialization to emit.
    pub layout: PaletteShowLayout,

    /// How the text layouts label each value collection; `None` means full
    /// concat paths.
    pub label: Option<PaletteShowLabel>,

    /// The markdown level of the shallowest heading a heading-emitting render
    /// prints; `None` starts at `#`.
    pub header_level: Option<NonZeroU8>,

    /// How the `Tables` layout shapes its tables; `None` means one table per
    /// palette group under headings.
    pub table_shape: Option<PaletteShowTableShape>,

    /// The column budget the `Rows` layout wraps to, or `None` for no
    /// wrapping.
    pub width: Option<usize>,
}
