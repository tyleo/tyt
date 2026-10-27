use crate::operations::sdf_doc::{Bounds2d, OutlineSegment};

/// A closed contour of an outline.
#[derive(Clone, Debug, PartialEq)]
pub struct OutlineContour {
    /// The contour's segments.
    pub segments: Vec<OutlineSegment>,

    /// The box around the segments' points, which holds the contour.
    pub bounds: Bounds2d,
}

impl OutlineContour {
    /// The contour of `segments`.
    ///
    /// # Panics
    ///
    /// When `segments` is empty.
    pub fn new(segments: Vec<OutlineSegment>) -> Self {
        let bounds = Bounds2d::from_points(segments.iter().flat_map(OutlineSegment::points));

        Self { segments, bounds }
    }
}
