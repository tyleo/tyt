use std::fmt::{Display, Formatter, Result as FmtResult};

/// The range of one vocabulary property: an inclusive interval, optionally
/// unioned with exactly zero as `ior` uses for "does not refract".
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialRange {
    /// The interval's inclusive lower bound.
    pub min: f64,

    /// The interval's inclusive upper bound, or `None` for an unbounded top.
    pub max: Option<f64>,

    /// Whether exactly zero is admitted outside the interval.
    pub admits_zero: bool,
}

impl MaterialRange {
    /// Whether `value` lies in the range. An unbounded top means arbitrarily
    /// large and finite, so no infinity lies in a range, and neither does
    /// NaN.
    pub fn contains(self, value: f64) -> bool {
        (self.admits_zero && value == 0.0)
            || (value.is_finite() && value >= self.min && self.max.is_none_or(|max| value <= max))
    }

    /// `value` clamped onto the interval. The zero the union admits is a
    /// marker, not a clamp target, so an out-of-range value lands on the
    /// interval's nearest end. NaN has no clamp and stays NaN.
    pub fn clamp(self, value: f64) -> f64 {
        value.clamp(self.min, self.max.unwrap_or(f64::INFINITY))
    }
}

impl Display for MaterialRange {
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
    use crate::material::MaterialRange;

    /// `ior`'s range: exactly zero or one and up.
    const IOR: MaterialRange = MaterialRange {
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
    fn an_unbounded_top_excludes_the_infinities() {
        let unbounded = MaterialRange {
            min: 0.0,
            max: None,
            admits_zero: false,
        };
        assert!(unbounded.contains(f64::MAX));
        assert!(!unbounded.contains(f64::INFINITY));
        assert!(!unbounded.contains(f64::NEG_INFINITY));
        assert!(!IOR.contains(f64::INFINITY));
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
        let closed = MaterialRange {
            min: 0.0,
            max: Some(1.0),
            admits_zero: false,
        };
        let open = MaterialRange {
            min: 0.0,
            max: None,
            admits_zero: false,
        };
        assert_eq!(closed.to_string(), "0 to 1");
        assert_eq!(open.to_string(), "0 or greater");
        assert_eq!(IOR.to_string(), "0, or 1 or greater");
    }
}
