use crate::{
    EvalFailure,
    evaluator::{EvalResult, rgb_from_oklab},
};
use std::f64::consts::TAU;

/// Converts Oklch to linear RGB, erroring on a hue outside `[0, 1]` or a
/// negative chroma.
pub(crate) fn rgb_from_oklch([lightness, chroma, hue]: [f32; 3]) -> EvalResult<[f32; 3]> {
    if !(0.0..=1.0).contains(&hue) {
        return Err(EvalFailure::HueRange { hue });
    }

    if chroma < 0.0 {
        return Err(EvalFailure::Chroma { chroma });
    }

    let angle = f64::from(hue) * TAU;
    let chroma = f64::from(chroma);

    Ok(rgb_from_oklab([
        lightness,
        (chroma * angle.cos()) as f32,
        (chroma * angle.sin()) as f32,
    ]))
}
