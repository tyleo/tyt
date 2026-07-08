use crate::ty_array_conversions;
use std::hash::{Hash, Hasher};

/// An sRGB color without alpha, the three-component companion to
/// [`TySrgba`](crate::TySrgba). The color space is the type identity; `T` is the
/// storage axis: `u8` bytes or `f32` / `f64` normalized `[0, 1]`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TySrgb<T = f32> {
    /// The red component, gamma-encoded.
    pub r: T,

    /// The green component, gamma-encoded.
    pub g: T,

    /// The blue component, gamma-encoded.
    pub b: T,
}

impl<T> TySrgb<T> {
    /// Creates a color from its components.
    pub fn new(r: T, g: T, b: T) -> Self {
        Self { r, g, b }
    }
}

ty_array_conversions!(TySrgb, 3, r, g, b);

// `Eq` / `Hash` on the 8-bit storage only, mirroring `TySrgba`, so the byte
// color can key a dedup map.
impl Eq for TySrgb<u8> {}

impl Hash for TySrgb<u8> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.r.hash(state);
        self.g.hash(state);
        self.b.hash(state);
    }
}

impl TySrgb<u8> {
    /// Normalizes each 8-bit component straight to `[0, 1]` without the sRGB
    /// transfer function; the gamma-encoded color as floats. The three-component
    /// companion to [`TySrgba::to_f64`](crate::TySrgba::to_f64).
    pub fn to_f64(self) -> TySrgb<f64> {
        TySrgb::new(
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{TySrgbF64, TySrgbU8, TySrgbaU8};
    use std::collections::HashSet;

    #[test]
    fn array_round_trips() {
        let color = TySrgbU8::from_array([1, 2, 3]);
        assert_eq!(color, TySrgbU8::new(1, 2, 3));
        assert_eq!(color.to_array(), [1, 2, 3]);
    }

    #[test]
    fn to_srgb_drops_alpha() {
        assert_eq!(TySrgbaU8::new(1, 2, 3, 4).to_srgb(), TySrgbU8::new(1, 2, 3));
    }

    #[test]
    fn u8_to_f64_normalizes() {
        // Straight normalize, no transfer function.
        assert_eq!(
            TySrgbU8::new(255, 128, 0).to_f64(),
            TySrgbF64::new(1.0, 128.0 / 255.0, 0.0)
        );
    }

    #[test]
    fn u8_color_keys_a_hash_set() {
        let mut set = HashSet::new();
        set.insert(TySrgbU8::new(1, 2, 3));
        set.insert(TySrgbU8::new(1, 2, 3));
        set.insert(TySrgbU8::new(9, 9, 9));
        assert_eq!(set.len(), 2);
    }
}
