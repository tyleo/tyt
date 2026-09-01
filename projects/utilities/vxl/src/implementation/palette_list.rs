use crate::{Format, Result, implementation};
use std::path::Path;
use voxsmith::{IndexRange, PaletteListFields, PaletteListLayout, render_palette_list};

/// Loads the voxel file at `input` and prints a per-palette overview: each
/// palette's index and, when enabled by `fields`, its property keys, material
/// count, and referencing objects. `filters` narrows the palettes and `layout`
/// chooses the Markdown table, the hierarchy tree, or a JSON form.
pub fn palette_list(
    input: &Path,
    from: Option<Format>,
    filters: &[IndexRange],
    fields: PaletteListFields,
    layout: PaletteListLayout,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;

    let output = render_palette_list(&state, filters, fields, layout)?;

    implementation::write_stdout(output.as_bytes())
}
