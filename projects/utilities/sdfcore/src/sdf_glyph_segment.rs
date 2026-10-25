use ty_math::TyVector2F64;

/// A segment of a glyph contour, from where the previous segment ends.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SdfGlyphSegment {
    /// A cubic curve through `control1` and `control2` to `to`.
    Cubic {
        control1: TyVector2F64,

        control2: TyVector2F64,

        to: TyVector2F64,
    },

    /// A line to `to`.
    Line { to: TyVector2F64 },

    /// A quadratic curve through `control` to `to`.
    Quadratic {
        control: TyVector2F64,

        to: TyVector2F64,
    },
}
