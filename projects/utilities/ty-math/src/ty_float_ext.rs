/// Extra floating-point operations this crate adds to `f32` and `f64`.
pub trait TyFloatExt {
    /// Maps `self` from the range `[low, high]` into a bucket index in
    /// `[0, buckets)`, clamping values outside the range to the end buckets. A
    /// non-positive range (`high <= low`) or a `buckets` of `0` maps to `0`.
    fn quantize(self, low: Self, high: Self, buckets: u32) -> u32;
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
        }
    };
}

impl_ty_float_ext!(f32);
impl_ty_float_ext!(f64);
