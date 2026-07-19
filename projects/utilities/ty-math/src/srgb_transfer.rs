/// The sRGB transfer function on one linear component, odd-extended by sign so
/// out-of-gamut inputs encode as `sign(x) * f(|x|)`, per CSS Color 4.
pub(crate) fn linear_to_srgb(linear: f64) -> f64 {
    let magnitude = linear.abs();

    let encoded = if magnitude <= 0.003_130_8 {
        12.92 * magnitude
    } else {
        1.055 * magnitude.powf(1.0 / 2.4) - 0.055
    };

    encoded.copysign(linear)
}

/// Inverts the sRGB transfer function to linear light, odd-extended by sign so
/// out-of-gamut inputs decode as `sign(x) * g(|x|)`, per CSS Color 4.
pub(crate) fn srgb_to_linear(c: f64) -> f64 {
    let magnitude = c.abs();

    let linear = if magnitude <= 0.040_45 {
        magnitude / 12.92
    } else {
        ((magnitude + 0.055) / 1.055).powf(2.4)
    };

    linear.copysign(c)
}
