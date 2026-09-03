use crate::{Format, Result, implementation};
use std::path::Path;
use voxsmith::{
    HierarchyShowLayout, HierarchyShowOptions, HierarchyViews, PatternView, render_hierarchy_show,
};

/// Loads the voxel file at `input` and prints its scene graph under `layout`.
pub fn hierarchy_show(
    input: &Path,
    from: Option<Format>,
    pattern: Option<PatternView>,
    layout: HierarchyShowLayout,
    collapse_instances: bool,
    views: HierarchyViews,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;

    let options = HierarchyShowOptions {
        pattern,
        layout,
        collapse_instances,
        views,
    };

    let output = render_hierarchy_show(&state, &options)?;

    implementation::write_stdout(output.as_bytes())
}
