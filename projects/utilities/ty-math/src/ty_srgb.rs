use crate::{TyFloatExt, TyVector3, ty_array_conversions};
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

impl<T: Copy> TySrgb<T> {
    /// The `r`, `g`, and `b` channels as a [`TyVector3`], a point in sRGB space
    /// for distance math.
    pub fn to_vector3(&self) -> TyVector3<T> {
        TyVector3::new(self.r, self.g, self.b)
    }
}

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

impl TySrgb<f64> {
    /// Quantizes each `[0, 1]` component to a byte without the sRGB transfer
    /// function, since the components are already sRGB-encoded. The inverse of
    /// `to_f64`. Encode from linear light on the linear type instead.
    pub fn to_u8(self) -> TySrgb<u8> {
        TySrgb::new(self.r.to_unorm8(), self.g.to_unorm8(), self.b.to_unorm8())
    }
}

#[cfg(test)]
mod tests {
    use crate::{TySrgbF64, TySrgbU8, TySrgbaU8, TyVector3F64};
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
    fn f64_to_u8_quantizes_and_clamps() {
        // Out-of-range components clamp to the byte endpoints; a half rounds
        // up, mirroring the four-component type.
        assert_eq!(
            TySrgbF64::new(-0.5, 2.0, 0.5).to_u8(),
            TySrgbU8::new(0, 255, 128)
        );
    }

    #[test]
    fn to_vector3_reads_channels() {
        // The channels map straight to a point for distance math.
        assert_eq!(
            TySrgbF64::new(0.25, 0.5, 0.75).to_vector3(),
            TyVector3F64::new(0.25, 0.5, 0.75)
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
