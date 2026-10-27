use ty_math::TyLinSrgbF32;

/// The Khronos PBR Neutral tonemap over a linear color. A color whose
/// brightest channel stays below `0.8` loses only a small flare offset.
/// Brighter highlights compress toward `1` and desaturate toward white.
pub fn tonemap(color: TyLinSrgbF32) -> TyLinSrgbF32 {
    const START_COMPRESSION: f32 = 0.8 - 0.04;
    const DESATURATION: f32 = 0.15;

    let darkest = color.red.min(color.green).min(color.blue);

    let offset = if darkest < 0.08 {
        darkest - 6.25 * darkest * darkest
    } else {
        0.04
    };

    let color = color - TyLinSrgbF32::new(offset, offset, offset);

    let peak = color.red.max(color.green).max(color.blue);

    if peak < START_COMPRESSION {
        return color;
    }

    let d = 1.0 - START_COMPRESSION;
    let new_peak = 1.0 - d * d / (peak + d - START_COMPRESSION);
    let color = color * (new_peak / peak);

    let toward_white = 1.0 - 1.0 / (DESATURATION * (peak - new_peak) + 1.0);
    let white = TyLinSrgbF32::new(new_peak, new_peak, new_peak);

    color * (1.0 - toward_white) + white * toward_white
}

#[cfg(test)]
mod tests {
    use crate::tonemap;
    use ty_math::TyLinSrgbF32;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn colors_below_the_knee_keep_their_hue_and_highlights_compress() {
        let black = tonemap(TyLinSrgbF32::new(0.0, 0.0, 0.0));
        assert_eq!(black, TyLinSrgbF32::new(0.0, 0.0, 0.0));

        // A mid gray loses only the flare offset.
        let gray = tonemap(TyLinSrgbF32::new(0.5, 0.5, 0.5));
        assert!(close(gray.red, 0.46) && close(gray.green, 0.46) && close(gray.blue, 0.46));

        // White compresses below one and stays neutral.
        let white = tonemap(TyLinSrgbF32::new(1.0, 1.0, 1.0));
        assert!(white.red < 1.0 && white.red > 0.8);
        assert!(close(white.red, white.green) && close(white.red, white.blue));

        // A bright red compresses and desaturates toward white.
        let red = tonemap(TyLinSrgbF32::new(2.0, 0.0, 0.0));
        assert!(red.red < 1.0 && red.red > 0.9);
        assert!(red.green > 0.0 && close(red.green, red.blue) && red.green < red.red);

        let brighter = tonemap(TyLinSrgbF32::new(4.0, 0.0, 0.0));
        assert!(brighter.red > red.red && brighter.red < 1.0);
    }
}
