use crate::SdfGlyphSegment;
use ty_math::TyVector2F64;

/// A closed contour of a glyph. The segments run from `start`, each from where
/// the one before it ends, and the contour closes back to `start`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SdfGlyphContour {
    /// The point the contour starts at.
    pub start: TyVector2F64,

    /// The contour's segments.
    pub segments: Vec<SdfGlyphSegment>,
}
