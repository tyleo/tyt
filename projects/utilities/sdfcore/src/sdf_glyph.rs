use crate::SdfGlyphContour;

/// A character's glyph in a font, in font units.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SdfGlyph {
    /// How far the next character starts along the line.
    pub advance: f64,

    /// The glyph's closed contours.
    pub contours: Vec<SdfGlyphContour>,
}
