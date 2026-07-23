use palette::Srgb;

/// An sRGB color without alpha, backed by [`palette::Srgb`]. The three-component
/// companion to [`TySrgba`](crate::TySrgba); the color space is the type
/// identity and `T` is the storage axis (`u8` bytes or `f32` / `f64` normalized
/// `[0, 1]`).
pub type TySrgb<T = f32> = Srgb<T>;

#[cfg(test)]
mod tests {
    use crate::{TySrgbU8, TySrgbaU8};

    #[test]
    fn color_field_drops_alpha() {
        // `.color` is palette's drop-alpha, replacing tyt's `to_srgb`.
        assert_eq!(TySrgbaU8::new(1, 2, 3, 4).color, TySrgbU8::new(1, 2, 3));
    }
}
