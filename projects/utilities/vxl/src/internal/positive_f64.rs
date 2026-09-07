use std::str::FromStr;

/// A positive `f64`: greater than zero and not NaN.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositiveF64(pub f64);

impl FromStr for PositiveF64 {
    type Err = String;

    /// Parses a number greater than zero, rejecting zero, negatives, and NaN.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let number = value
            .parse::<f64>()
            .map_err(|_| format!("`{value}` is not a number"))?;

        if number <= 0.0 || number.is_nan() {
            return Err(format!("`{value}` must be greater than 0"));
        }

        Ok(PositiveF64(number))
    }
}

#[cfg(test)]
mod tests {
    use crate::PositiveF64;

    #[test]
    fn accepts_a_positive_number() {
        assert_eq!("0.25".parse::<PositiveF64>(), Ok(PositiveF64(0.25)));
    }

    #[test]
    fn rejects_zero_negative_and_nan() {
        assert!("0".parse::<PositiveF64>().is_err());
        assert!("-1".parse::<PositiveF64>().is_err());
        assert!("nan".parse::<PositiveF64>().is_err());
    }

    #[test]
    fn rejects_a_non_number() {
        assert!("abc".parse::<PositiveF64>().is_err());
    }
}
