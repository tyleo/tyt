use crate::evaluator::oklab_from_rgb;
use std::f64::consts::TAU;

/// Converts linear RGB to Oklch, with hue a turn in `[0, 1)` and 0 where
/// the chroma reads as zero.
pub(crate) fn oklch_from_rgb(rgb: [f32; 3]) -> [f32; 3] {
    let [lightness, a, b] = oklab_from_rgb(rgb);
    let (a, b) = (f64::from(a), f64::from(b));
    let chroma = a.hypot(b);

    if chroma < CHROMA_FLOOR {
        return [lightness, 0.0, 0.0];
    }

    let turn = b.atan2(a) / TAU;
    let hue = if turn < 0.0 { turn + 1.0 } else { turn };

    [lightness, chroma as f32, hue as f32]
}

/// The chroma below which a color reads as gray, because the rounded
/// matrices leave that much noise on one.
const CHROMA_FLOOR: f64 = 1e-6;
