/// Extra floating-point operations this crate adds to `f32` and `f64`.
pub trait TyFloatExt {
    /// Maps `self` from the range `[low, high]` into a bucket index in
    /// `[0, buckets)`, clamping values outside the range to the end buckets. A
    /// non-positive range (`high <= low`) or a `buckets` of `0` maps to `0`.
    fn quantize(self, low: Self, high: Self, buckets: u32) -> u32;

    /// True when `self` and `other` are within `tolerance` of each other.
    // `self` by value matches the float convention (`f64::is_nan`); the trait
    // cannot express `Self: Copy` for the lint to see it.
    #[allow(clippy::wrong_self_convention)]
    fn is_approximately_equal(self, other: Self, tolerance: Self) -> bool;

    /// Wraps `self` into `[0, 1)` by subtracting its floor, so `1.25` and
    /// `-0.75` both map to `0.25`.
    fn wrap01(self) -> Self;

    /// Linearly interpolates from `self` towards `other` by `t`, with `t` of `0`
    /// returning `self` and `1` returning `other`.
    fn lerp(self, other: Self, t: Self) -> Self;
}

macro_rules! impl_ty_float_ext {
    ($t:ty) => {
        impl TyFloatExt for $t {
            fn quantize(self, low: $t, high: $t, buckets: u32) -> u32 {
                if high <= low || buckets == 0 {
                    return 0;
                }

                let t = ((self - low) / (high - low)).clamp(0.0, 1.0);

                ((t * buckets as $t) as u32).min(buckets - 1)
            }

            fn is_approximately_equal(self, other: $t, tolerance: $t) -> bool {
                (self - other).abs() < tolerance
            }

            fn wrap01(self) -> $t {
                self - self.floor()
            }

            fn lerp(self, other: $t, t: $t) -> $t {
                self + (other - self) * t
            }
        }
    };
}

impl_ty_float_ext!(f32);
impl_ty_float_ext!(f64);
