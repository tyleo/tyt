use crate::operations::sdf_doc::OutlineSegment;
use sdfcore::{SdfFontData, SdfGlyphSegment, SdfTextAlign};
use std::result::Result as StdResult;
use ty_math::TyVector2F64;

/// The contours of `string` set in `font` along +u with its baseline on v = 0.
/// A capital letter stands `size` tall, and `align` picks the point of the line
/// on u = 0. Errors report the font as `font_name`.
pub fn text_outline(
    font: &SdfFontData,
    font_name: &str,
    string: &str,
    size: f64,
    align: SdfTextAlign,
) -> StdResult<Vec<Vec<OutlineSegment>>, String> {
    let cap_height = font.cap_height.ok_or_else(|| {
        format!("the font `{font_name}` gives no cap height and has no `H` to measure")
    })?;

    let tolerance = font.units_per_em / 1000.0;
    let mut contours = Vec::new();
    let mut pen = 0.0;
    let mut previous = None;
    for character in string.chars() {
        let glyph = font
            .glyphs
            .get(&character)
            .ok_or_else(|| format!("the font `{font_name}` has no `{character}`"))?;
        if let Some(previous) = previous.replace(character) {
            pen += font
                .kerning
                .get(&(previous, character))
                .copied()
                .unwrap_or(0.0);
        }
        let offset = TyVector2F64::new(pen, 0.0);

        for contour in &glyph.contours {
            let mut builder = ContourBuilder::new(contour.start + offset, tolerance);
            for segment in &contour.segments {
                match *segment {
                    SdfGlyphSegment::Cubic {
                        control1,
                        control2,
                        to,
                    } => builder.cubic(control1 + offset, control2 + offset, to + offset),

                    SdfGlyphSegment::Line { to } => builder.line(to + offset),

                    SdfGlyphSegment::Quadratic { control, to } => {
                        builder.quadratic(control + offset, to + offset)
                    }
                }
            }

            contours.extend(builder.close());
        }

        pen += glyph.advance;
    }

    if contours.is_empty() {
        return Err(format!(
            "`{string}` draws nothing in the font `{font_name}`"
        ));
    }

    let scale = size / cap_height;
    let shift = TyVector2F64::new(
        match align {
            SdfTextAlign::Center => -pen / 2.0,
            SdfTextAlign::End => -pen,
            SdfTextAlign::Start => 0.0,
        },
        0.0,
    );
    let place = |point: TyVector2F64| (point + shift) * scale;

    Ok(contours
        .into_iter()
        .map(|contour| {
            contour
                .into_iter()
                .map(|segment| match segment {
                    OutlineSegment::Line { to } => OutlineSegment::Line { to: place(to) },

                    OutlineSegment::Quadratic { control, to } => OutlineSegment::Quadratic {
                        control: place(control),
                        to: place(to),
                    },
                })
                .collect()
        })
        .collect())
}

/// Collects one contour in font units and splits each cubic curve into
/// quadratic curves that stray from it by at most `tolerance`.
struct ContourBuilder {
    tolerance: f64,

    start: TyVector2F64,

    current: TyVector2F64,

    segments: Vec<OutlineSegment>,
}

impl ContourBuilder {
    fn new(start: TyVector2F64, tolerance: f64) -> Self {
        ContourBuilder {
            tolerance,
            start,
            current: start,
            segments: Vec::new(),
        }
    }

    fn line(&mut self, to: TyVector2F64) {
        self.current = to;
        self.segments.push(OutlineSegment::Line { to });
    }

    fn quadratic(&mut self, control: TyVector2F64, to: TyVector2F64) {
        self.current = to;
        self.segments
            .push(OutlineSegment::Quadratic { control, to });
    }

    fn cubic(&mut self, c1: TyVector2F64, c2: TyVector2F64, p3: TyVector2F64) {
        let p0 = self.current;
        let at = |t: f64| {
            let s = 1.0 - t;
            p0 * (s * s * s) + c1 * (3.0 * s * s * t) + c2 * (3.0 * s * t * t) + p3 * (t * t * t)
        };
        let tangent = |t: f64| {
            let s = 1.0 - t;
            (c1 - p0) * (3.0 * s * s) + (c2 - c1) * (6.0 * s * t) + (p3 - c2) * (3.0 * t * t)
        };

        // The quadratic through `(3 * (c1 + c2) - p0 - p3) / 4` strays from its
        // cubic by at most sqrt(3) / 36 of the cubic's third difference, which
        // shrinks with the cube of the share of the curve a piece covers.
        let error = 3f64.sqrt() / 36.0 * (p3 - c2 * 3.0 + c1 * 3.0 - p0).length();
        let pieces = ((error / self.tolerance).cbrt().ceil() as u32).max(1);
        let step = 1.0 / f64::from(pieces);

        for piece in 1..=pieces {
            let (t0, t1) = (f64::from(piece - 1) * step, f64::from(piece) * step);
            let from = self.current;
            let to = if piece == pieces { p3 } else { at(t1) };
            let control1 = from + tangent(t0) * (step / 3.0);
            let control2 = to - tangent(t1) * (step / 3.0);
            self.quadratic(((control1 + control2) * 3.0 - from - to) / 4.0, to);
        }
    }

    /// The contour closed back to its start, or `None` when it encloses
    /// nothing.
    fn close(mut self) -> Option<Vec<OutlineSegment>> {
        if self.current != self.start {
            self.line(self.start);
        }

        // A contour of one segment encloses nothing.
        (self.segments.len() >= 2).then_some(self.segments)
    }
}

#[cfg(test)]
mod tests {
    use super::ContourBuilder;
    use crate::operations::sdf_doc::{OutlineSegment, text_outline};
    use sdfcore::{SdfFontData, SdfGlyph, SdfGlyphContour, SdfGlyphSegment, SdfTextAlign};
    use std::collections::BTreeMap;
    use ty_math::TyVector2F64;

    fn point(u: f64, v: f64) -> TyVector2F64 {
        TyVector2F64::new(u, v)
    }

    fn line(u: f64, v: f64) -> SdfGlyphSegment {
        SdfGlyphSegment::Line { to: point(u, v) }
    }

    /// A capital 500 wide and 700 tall, the face's cap height.
    fn h() -> (char, SdfGlyph) {
        (
            'H',
            SdfGlyph {
                advance: 600.0,
                contours: vec![SdfGlyphContour {
                    start: point(0.0, 0.0),
                    segments: vec![line(0.0, 700.0), line(500.0, 700.0), line(500.0, 0.0)],
                }],
            },
        )
    }

    /// A bowl rising to 500 between 0 and 500.
    fn u() -> (char, SdfGlyph) {
        (
            'U',
            SdfGlyph {
                advance: 550.0,
                contours: vec![SdfGlyphContour {
                    start: point(0.0, 0.0),
                    segments: vec![SdfGlyphSegment::Quadratic {
                        control: point(250.0, 500.0),
                        to: point(500.0, 0.0),
                    }],
                }],
            },
        )
    }

    fn space() -> (char, SdfGlyph) {
        (
            ' ',
            SdfGlyph {
                advance: 250.0,
                contours: Vec::new(),
            },
        )
    }

    fn outline(
        cap_height: Option<f64>,
        glyphs: Vec<(char, SdfGlyph)>,
        string: &str,
        align: SdfTextAlign,
    ) -> Result<Vec<Vec<OutlineSegment>>, String> {
        let font = SdfFontData {
            units_per_em: 1000.0,
            cap_height,
            glyphs: glyphs.into_iter().collect(),
            kerning: BTreeMap::new(),
        };
        text_outline(&font, "sign", string, 0.07, align)
    }

    #[test]
    fn each_character_follows_the_advances_scaled_to_the_cap_height() {
        let contours = outline(
            Some(700.0),
            vec![h(), u(), space()],
            "H U",
            SdfTextAlign::Start,
        )
        .unwrap();

        // 700 units scale to 0.07, so each unit spans 0.0001.
        assert_eq!(contours.len(), 2);
        let [h, u] = [&contours[0], &contours[1]].map(|contour| contour.as_slice());
        let close = |a: TyVector2F64, b: TyVector2F64| a.distance(b) < 1e-12;
        assert!(close(h[1].to(), point(0.05, 0.07)), "{h:?}");
        let OutlineSegment::Quadratic { control, to } = u[0] else {
            panic!("{u:?}");
        };
        assert!(close(control, point(0.11, 0.05)), "{control}");
        assert!(close(to, point(0.135, 0.0)), "{to}");
    }

    #[test]
    fn a_kerned_pair_moves_the_second_character_and_the_rest_of_the_line() {
        let font = SdfFontData {
            units_per_em: 1000.0,
            cap_height: Some(700.0),
            glyphs: [h(), u()].into(),
            kerning: [(('H', 'U'), -100.0), (('U', 'H'), 300.0)].into(),
        };
        let contours = text_outline(&font, "sign", "HUH", 0.07, SdfTextAlign::Start).unwrap();

        // `U` starts at 600 - 100 and the second `H` at 500 + 550 + 300.
        let close = |a: TyVector2F64, b: TyVector2F64| a.distance(b) < 1e-12;
        assert!(close(contours[1][0].to(), point(0.1, 0.0)), "{contours:?}");
        assert!(
            close(contours[2][0].to(), point(0.135, 0.07)),
            "{contours:?}"
        );
    }

    #[test]
    fn the_alignment_puts_the_start_middle_or_end_on_zero() {
        for (align, left) in [
            (SdfTextAlign::Start, 0.0),
            (SdfTextAlign::Center, -0.03),
            (SdfTextAlign::End, -0.06),
        ] {
            let contours = outline(Some(700.0), vec![h()], "H", align).unwrap();
            let left_most = contours[0]
                .iter()
                .flat_map(OutlineSegment::points)
                .map(|point| point.x)
                .fold(f64::INFINITY, f64::min);
            assert!((left_most - left).abs() < 1e-12, "{align:?}: {left_most}");
        }
    }

    #[test]
    fn a_text_the_font_cannot_draw_errors() {
        for (cap_height, glyphs, string, message) in [
            (Some(700.0), vec![h()], "HÉ", "the font `sign` has no `É`"),
            (
                None,
                vec![u()],
                "U",
                "the font `sign` gives no cap height and has no `H` to measure",
            ),
            (
                Some(700.0),
                vec![h(), space()],
                "  ",
                "`  ` draws nothing in the font `sign`",
            ),
        ] {
            assert_eq!(
                outline(cap_height, glyphs, string, SdfTextAlign::Start).unwrap_err(),
                message
            );
        }
    }

    #[test]
    fn a_cubic_splits_into_quadratics_within_the_tolerance() {
        let (p0, c1, c2, p3) = (
            point(0.0, 0.0),
            point(100.0, 900.0),
            point(900.0, -400.0),
            point(1000.0, 300.0),
        );
        let mut builder = ContourBuilder::new(p0, 1.0);
        builder.cubic(c1, c2, p3);
        let contour = builder.close().unwrap();

        let quadratics = &contour[..contour.len() - 1];
        assert!(quadratics.len() > 1);
        assert_eq!(contour[contour.len() - 1], OutlineSegment::Line { to: p0 });

        let step = 1.0 / quadratics.len() as f64;
        let mut from = p0;
        for (index, segment) in quadratics.iter().enumerate() {
            let OutlineSegment::Quadratic { control, to } = *segment else {
                panic!("{segment:?}");
            };

            for sample in 0..=100 {
                let s = f64::from(sample) / 100.0;
                let t = (index as f64 + s) * step;
                let cubic = p0 * (1.0 - t).powi(3)
                    + c1 * (3.0 * (1.0 - t).powi(2) * t)
                    + c2 * (3.0 * (1.0 - t) * t * t)
                    + p3 * t.powi(3);
                let quadratic =
                    from * ((1.0 - s) * (1.0 - s)) + control * (2.0 * s * (1.0 - s)) + to * (s * s);
                assert!(cubic.distance(quadratic) <= 1.0, "{index} {s}");
            }
            from = to;
        }
        assert_eq!(from, p3);
    }
}
