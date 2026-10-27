use crate::operations::sdf_doc::exact_sin_cos;
use ty_math::TyVector2F64;

/// The span of an `arc`, a `sector`, or a cut `torus` from one angle to
/// another, counterclockwise from +u.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArcSpan {
    turn_sin: f64,

    turn_cos: f64,

    half_sin: f64,

    half_cos: f64,
}

impl ArcSpan {
    /// The span from `from_degrees` to `to_degrees`.
    pub fn new(from_degrees: f64, to_degrees: f64) -> Self {
        let (turn_sin, turn_cos) = exact_sin_cos(90.0 - (from_degrees + to_degrees) / 2.0);
        let (half_sin, half_cos) = exact_sin_cos((to_degrees - from_degrees) / 2.0);

        Self {
            turn_sin,
            turn_cos,
            half_sin,
            half_cos,
        }
    }

    /// `point` turned so the span's middle lies on +v, with u folded to
    /// `abs(u)`.
    pub fn fold(&self, point: TyVector2F64) -> TyVector2F64 {
        TyVector2F64::new(
            (point.x * self.turn_cos - point.y * self.turn_sin).abs(),
            point.x * self.turn_sin + point.y * self.turn_cos,
        )
    }

    /// Whether the folded point `folded` lies past the span's end.
    pub fn is_past_end(&self, folded: TyVector2F64) -> bool {
        self.half_cos * folded.x > self.half_sin * folded.y
    }

    /// The span's end at `radius` from the center in the folded frame.
    pub fn end(&self, radius: f64) -> TyVector2F64 {
        TyVector2F64::new(radius * self.half_sin, radius * self.half_cos)
    }

    /// The folded point `folded` turned further so the span's end lies on +v.
    /// A point inside the span then reads a negative u.
    pub fn to_end_frame(self, folded: TyVector2F64) -> TyVector2F64 {
        TyVector2F64::new(
            self.half_cos * folded.x - self.half_sin * folded.y,
            self.half_sin * folded.x + self.half_cos * folded.y,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::ArcSpan;
    use ty_math::TyVector2F64;

    #[test]
    fn the_middle_folds_onto_v_and_the_ends_mark_the_span() {
        let span = ArcSpan::new(0.0, 90.0);

        let middle = TyVector2F64::new(1.0, 1.0);
        let folded = span.fold(middle);
        assert!(folded.x.abs() < 1e-15);
        assert!(!span.is_past_end(folded));

        assert!(!span.is_past_end(span.fold(TyVector2F64::new(1.0, 0.1))));
        assert!(span.is_past_end(span.fold(TyVector2F64::new(1.0, -0.1))));
        assert!(span.is_past_end(span.fold(TyVector2F64::new(-0.1, 1.0))));

        let end = span.end(1.0);
        assert!((span.fold(TyVector2F64::new(1.0, 0.0)) - end).length() < 1e-15);
        assert!(span.to_end_frame(end).x.abs() < 1e-15);
        assert!(span.to_end_frame(folded).x < 0.0);
    }

    #[test]
    fn a_full_turn_holds_every_direction() {
        let span = ArcSpan::new(30.0, 390.0);

        for point in [
            TyVector2F64::new(1.0, 0.0),
            TyVector2F64::new(-1.0, 0.0),
            TyVector2F64::new(0.0, -1.0),
        ] {
            assert!(!span.is_past_end(span.fold(point)));
        }
    }
}
