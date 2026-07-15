/// The color block a swatch-rendering cell format shows beside or
/// instead of a value's text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeGridSwatch {
    /// A truecolor swatch of the given `[r, g, b]` bytes.
    Color([u8; 3]),

    /// A grayscale swatch: the level repeated on all three channels.
    Gray(u8),
}
