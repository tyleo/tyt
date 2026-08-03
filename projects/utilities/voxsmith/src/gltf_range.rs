use crate::{Error, Result};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// The glTF schema range of one vocabulary attribute: an inclusive interval
/// from `min` to `max`, optionally unioned with exactly zero, the shape
/// `KHR_materials_ior` gives `ior` for "does not refract".
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GltfRange {
    /// The interval's inclusive lower bound.
    pub min: f64,

    /// The interval's inclusive upper bound, or `None` for an unbounded top.
    pub max: Option<f64>,

    /// Whether exactly zero is admitted outside the interval.
    pub admits_zero: bool,
}

impl GltfRange {
    /// Whether `value` lies in the range. NaN lies in no range.
    pub fn contains(self, value: f64) -> bool {
        (self.admits_zero && value == 0.0)
            || (value >= self.min && self.max.is_none_or(|max| value <= max))
    }

    /// `value` clamped onto the interval. The zero the union admits is a
    /// marker, not a clamp target, so an out-of-range value lands on the
    /// interval's nearest end. NaN has no clamp and stays NaN.
    pub fn clamp(self, value: f64) -> f64 {
        value.clamp(self.min, self.max.unwrap_or(f64::INFINITY))
    }

    /// Errors unless `value` lies in the range, naming the vocabulary
    /// property `name` that binds it.
    pub fn check(self, name: &str, value: f64) -> Result<()> {
        if self.contains(value) {
            Ok(())
        } else {
            Err(Error::invalid(format!(
                "`{name}` is {value}, outside the glTF range {self}"
            )))
        }
    }
}

impl Display for GltfRange {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        if self.admits_zero {
            write!(formatter, "0, or ")?;
        }
        match self.max {
            Some(max) => write!(formatter, "{} to {}", self.min, max),
            None => write!(formatter, "{} or greater", self.min),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GltfRange;

    /// `ior`'s range: exactly zero or one and up.
    const IOR: GltfRange = GltfRange {
        min: 1.0,
        max: None,
        admits_zero: true,
    };

    #[test]
    fn the_union_admits_zero_and_the_interval_only() {
        assert!(IOR.contains(0.0));
        assert!(IOR.contains(1.0));
        assert!(IOR.contains(2.5));
        assert!(!IOR.contains(0.5));
        assert!(!IOR.contains(-1.0));
        assert!(!IOR.contains(f64::NAN));
    }

    #[test]
    fn the_clamp_targets_the_interval_not_the_zero() {
        assert_eq!(IOR.clamp(0.5), 1.0);
        assert_eq!(IOR.clamp(-1.0), 1.0);
        assert_eq!(IOR.clamp(2.5), 2.5);
        assert!(IOR.clamp(f64::NAN).is_nan());
    }

    #[test]
    fn displays_each_shape() {
        let closed = GltfRange {
            min: 0.0,
            max: Some(1.0),
            admits_zero: false,
        };
        let open = GltfRange {
            min: 0.0,
            max: None,
            admits_zero: false,
        };
        assert_eq!(closed.to_string(), "0 to 1");
        assert_eq!(open.to_string(), "0 or greater");
        assert_eq!(IOR.to_string(), "0, or 1 or greater");
    }
}
