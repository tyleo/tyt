/// An unsigned component type, widened to `u64` for the checked arithmetic.
pub(crate) trait Unsigned: Copy + Ord {
    /// The value as a `u64`.
    fn to_u64(self) -> u64;

    /// The value from a `u64`, none where it does not fit.
    fn from_u64(value: u64) -> Option<Self>;

    /// The value as an `f64`.
    fn to_f64(self) -> f64;

    /// The largest value.
    fn max_value() -> Self;
}

impl Unsigned for u8 {
    fn to_u64(self) -> u64 {
        u64::from(self)
    }

    fn from_u64(value: u64) -> Option<Self> {
        u8::try_from(value).ok()
    }

    fn to_f64(self) -> f64 {
        f64::from(self)
    }

    fn max_value() -> Self {
        u8::MAX
    }
}

impl Unsigned for u16 {
    fn to_u64(self) -> u64 {
        u64::from(self)
    }

    fn from_u64(value: u64) -> Option<Self> {
        u16::try_from(value).ok()
    }

    fn to_f64(self) -> f64 {
        f64::from(self)
    }

    fn max_value() -> Self {
        u16::MAX
    }
}

impl Unsigned for u32 {
    fn to_u64(self) -> u64 {
        u64::from(self)
    }

    fn from_u64(value: u64) -> Option<Self> {
        u32::try_from(value).ok()
    }

    fn to_f64(self) -> f64 {
        f64::from(self)
    }

    fn max_value() -> Self {
        u32::MAX
    }
}
