/// One value's non-text cell content: pre-rendered bytes and the
/// display columns they occupy.
///
/// The renderer treats the content as opaque, emitting `rendered` and
/// laying out by `width`, so any terminal medium works: the stock
/// color blocks from
/// [`TreeGridSwatch::render`](crate::TreeGridSwatch::render), or
/// content a cell policy builds itself. Spacing a visual wants around
/// itself belongs in its own bytes and width.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeGridVisual {
    /// The bytes to emit.
    pub rendered: String,

    /// The display columns the bytes occupy.
    pub width: usize,
}

impl TreeGridVisual {
    /// Creates a visual from pre-rendered bytes and their display
    /// width.
    pub fn new(rendered: impl Into<String>, width: usize) -> Self {
        Self {
            rendered: rendered.into(),
            width,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::TreeGridVisual;

    #[test]
    fn new_stores_the_bytes_and_width() {
        let visual = TreeGridVisual::new("**", 2);

        assert_eq!(visual.rendered, "**");
        assert_eq!(visual.width, 2);
    }
}
