use crate::{Format, Result, implementation};
use std::{fs, path::Path};
use voxsmith::{InfoDocument, InfoLayout, render_info, voxj_version_from_bytes};

/// Loads the voxel file at `input` and reports what it contains in `layout`.
/// The document is read into voxcore first; only the format and, for Voxel Json,
/// the document version come from outside that model.
pub fn info(input: &Path, from: Option<Format>, layout: InfoLayout) -> Result<()> {
    // The raw-voxj loader keeps any source's ext as a block, so the report
    // can say whether the document carries one.
    let state = implementation::load_state_voxj(input, from)?;

    // load_state_voxj resolved the format, so this inference cannot fail.
    let format = from
        .or_else(|| Format::from_path(input))
        .expect("load_state_voxj resolved the input format");

    let voxj_version = match format {
        Format::Voxj => Some(read_voxj_version(input)?),
        _ => None,
    };

    let name = input
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| input.display().to_string());

    let document = InfoDocument {
        name: &name,
        format: implementation::voxel_format(format),
        voxj_version,
        has_ext: state.ext().is_some(),
    };

    let output = render_info(&state, &document, layout);

    implementation::write_stdout(output.as_bytes())
}

/// The Voxel Json document version of `input`; voxcore does not carry it.
fn read_voxj_version(input: &Path) -> Result<u32> {
    Ok(voxj_version_from_bytes(&fs::read(input)?)?)
}
