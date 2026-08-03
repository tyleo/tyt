use palette::Srgba;

/// An sRGB color with straight alpha, backed by [`palette::Srgba`]. The color
/// space is the type identity; `T` is the storage axis: `u8` is the `#RRGGBBAA`
/// byte form, `f32` / `f64` are normalized `[0, 1]`. Decode to linear light with
/// `into_linear` before lighting math.
pub type TySrgba<T = f32> = Srgba<T>;

#[cfg(test)]
mod tests {
    use crate::{TySrgbaF64, TySrgbaU8};

    #[test]
    fn array_round_trips() {
        // palette's array casts read and write component order.
        let color = TySrgbaU8::from([1, 2, 3, 4]);
        assert_eq!(color, TySrgbaU8::new(1, 2, 3, 4));
        assert_eq!(<[u8; 4]>::from(color), [1, 2, 3, 4]);

        let from: TySrgbaU8 = [10, 20, 30, 40].into();
        assert_eq!(from, TySrgbaU8::new(10, 20, 30, 40));
    }

    #[test]
    fn scalar_multiply_scales_each_component() {
        // Scalar `*` scales every channel, alpha included; the operands are
        // exact binary fractions so the equality is precise.
        assert_eq!(
            TySrgbaF64::new(0.25, 0.5, 0.125, 1.0) * 2.0,
            TySrgbaF64::new(0.5, 1.0, 0.25, 2.0)
        );
    }

    #[test]
    fn u8_to_f64_normalizes() {
        // `into_format` recasts the component number type straight, no transfer
        // function; the alpha recasts the same way.
        assert_eq!(
            TySrgbaU8::new(255, 128, 0, 64).into_format::<f64, f64>(),
            TySrgbaF64::new(1.0, 128.0 / 255.0, 0.0, 64.0 / 255.0)
        );
    }

    #[test]
    fn f64_to_u8_quantizes_and_clamps() {
        // Out-of-range components clamp to the byte endpoints.
        assert_eq!(
            TySrgbaF64::new(-0.5, 2.0, 0.5, 1.0).into_format::<u8, u8>(),
            TySrgbaU8::new(0, 255, 128, 255)
        );
    }

    #[test]
    fn u8_round_trips_through_f64() {
        // byte -> float -> byte is exact for byte-valued components.
        let bytes = TySrgbaU8::new(0, 128, 255, 64);
        assert_eq!(
            bytes.into_format::<f64, f64>().into_format::<u8, u8>(),
            bytes
        );
    }
}
