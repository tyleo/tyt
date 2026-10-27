/// A glyph of a test font: a character, its advance, and its contours of
/// points, each `(x, y, on_curve)`.
pub type TestGlyph = (char, u16, &'static [&'static [(i16, i16, bool)]]);
