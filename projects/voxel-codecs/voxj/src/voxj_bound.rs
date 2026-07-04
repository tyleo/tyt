#[cfg(feature = "serde")]
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error as DeError, Unexpected, Visitor},
};
#[cfg(feature = "serde")]
use std::fmt::{Formatter, Result as FmtResult};

/// One side of a value pool's `min`/`max` bound: a finite number, or the
/// literal string `"none"` for unbounded on that side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VoxjBound {
    /// A finite numeric bound. Serialized as a JSON integer when integral, so a
    /// bound like `1` does not round-trip as `1.0`.
    Number(f64),

    /// Unbounded on this side; the literal string `"none"`.
    None,
}

#[cfg(feature = "serde")]
impl Serialize for VoxjBound {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            // Serialize an integral bound as a JSON integer.
            VoxjBound::Number(n)
                if n.fract() == 0.0 && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 =>
            {
                serializer.serialize_i64(*n as i64)
            }

            VoxjBound::Number(n) => serializer.serialize_f64(*n),

            VoxjBound::None => serializer.serialize_str("none"),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for VoxjBound {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(VoxjBoundVisitor)
    }
}

/// Rejects JSON `null` and non-finite numbers, so a bound is always a finite
/// number or the string `"none"`.
#[cfg(feature = "serde")]
struct VoxjBoundVisitor;

#[cfg(feature = "serde")]
impl Visitor<'_> for VoxjBoundVisitor {
    type Value = VoxjBound;

    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        formatter.write_str("a finite number or the string \"none\"")
    }

    fn visit_i64<E>(self, value: i64) -> Result<VoxjBound, E>
    where
        E: DeError,
    {
        Ok(VoxjBound::Number(value as f64))
    }

    fn visit_u64<E>(self, value: u64) -> Result<VoxjBound, E>
    where
        E: DeError,
    {
        Ok(VoxjBound::Number(value as f64))
    }

    fn visit_f64<E>(self, value: f64) -> Result<VoxjBound, E>
    where
        E: DeError,
    {
        // A bound must be finite; reject NaN and +/- infinity.
        if value.is_finite() {
            Ok(VoxjBound::Number(value))
        } else {
            Err(E::custom("value pool bound must be a finite number"))
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<VoxjBound, E>
    where
        E: DeError,
    {
        if value == "none" {
            Ok(VoxjBound::None)
        } else {
            Err(E::invalid_value(Unexpected::Str(value), &self))
        }
    }
}
