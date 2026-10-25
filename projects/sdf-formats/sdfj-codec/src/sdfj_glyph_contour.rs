use crate::SdfjGlyphSegment;

/// A closed contour of a glyph. The segments run from `start`, each from where
/// the one before it ends, and the contour closes back to `start`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SdfjGlyphContour {
    /// The point the contour starts at.
    pub start: [f64; 2],

    /// The contour's segments.
    pub segments: Vec<SdfjGlyphSegment>,
}
