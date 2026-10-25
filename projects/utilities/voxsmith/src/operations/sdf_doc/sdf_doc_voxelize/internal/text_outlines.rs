use crate::{
    Error, Result,
    operations::sdf_doc::{EntryPaths, OutlineSegment, text_outline},
};
use branded_id::{IteratorExt, U32Id};
use sdfcore::{BSdfShape2d, SdfMain, SdfShape2d, SdfTextAlign};
use std::collections::HashMap;

/// The contours of each `text` of `main`, laid out in its font. An error
/// starts with where `paths` points for the failing text.
pub fn text_outlines(
    main: &SdfMain,
    paths: &EntryPaths,
) -> Result<HashMap<U32Id<BSdfShape2d>, Vec<Vec<OutlineSegment>>>> {
    let state = main.state();
    let mut outlines = HashMap::new();

    for (shape2d_id, shape) in state.shapes2d.iter().enumerate_ids() {
        let SdfShape2d::Text {
            font_id,
            string,
            size,
            align,
        } = shape
        else {
            continue;
        };

        let at =
            |message: String| Error::invalid(format!("{}: {message}", paths.shape2d(shape2d_id)));
        let font_name = state.fonts[font_id.to_usize_id()].name();

        let font_data = main
            .font_data()
            .ok_or_else(|| at(format!("the font `{font_name}` is not loaded")))?;

        let outline = text_outline(
            &font_data[font_id.to_usize_id()],
            font_name,
            string,
            *size,
            align.unwrap_or(SdfTextAlign::Start),
        )
        .map_err(at)?;
        outlines.insert(shape2d_id, outline);
    }

    Ok(outlines)
}
