use std::iter;
use ty_math::TyVector2F64;

/// A segment of a closed contour. Each segment starts where the previous one
/// ends, and the first where the last ends.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutlineSegment {
    /// A line to `to`.
    Line { to: TyVector2F64 },

    /// A quadratic curve through `control` to `to`.
    Quadratic {
        control: TyVector2F64,

        to: TyVector2F64,
    },
}

impl OutlineSegment {
    /// The point the segment ends at.
    pub fn to(&self) -> TyVector2F64 {
        match *self {
            OutlineSegment::Line { to } | OutlineSegment::Quadratic { to, .. } => to,
        }
    }

    /// The segment's points, the control point first.
    pub fn points(&self) -> impl Iterator<Item = TyVector2F64> {
        let control = match *self {
            OutlineSegment::Line { .. } => None,
            OutlineSegment::Quadratic { control, .. } => Some(control),
        };

        control.into_iter().chain(iter::once(self.to()))
    }
}
