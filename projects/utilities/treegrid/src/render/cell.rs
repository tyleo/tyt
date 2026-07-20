use crate::render;

/// One rendered cell: the bytes to emit and the display columns they occupy, so
/// alignment lays out by a visual's declared width instead of re-measuring its
/// bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Cell {
    /// The bytes to emit.
    pub(crate) rendered: String,

    /// The display columns the bytes occupy.
    pub(crate) width: usize,

    /// The strip rule's test: whether the cell is a bare visual.
    pub(crate) bare_visual: bool,
}

impl Cell {
    /// A text cell, its width measured past ANSI escapes.
    pub(crate) fn text(text: impl Into<String>) -> Self {
        let rendered = text.into();
        let width = render::visible_width(&rendered);
        Self {
            rendered,
            width,
            bare_visual: false,
        }
    }
}
