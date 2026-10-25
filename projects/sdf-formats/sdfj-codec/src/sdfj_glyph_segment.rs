/// A segment of a glyph contour, from where the previous segment ends.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SdfjGlyphSegment {
    /// A cubic curve through `control1` and `control2` to `to`.
    Cubic {
        control1: [f64; 2],

        control2: [f64; 2],

        to: [f64; 2],
    },

    /// A line to `to`.
    Line { to: [f64; 2] },

    /// A quadratic curve through `control` to `to`.
    Quadratic { control: [f64; 2], to: [f64; 2] },
}
