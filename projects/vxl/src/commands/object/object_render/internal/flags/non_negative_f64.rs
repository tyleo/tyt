use serde::Deserialize;
use std::str::FromStr;

/// A finite `f64` of zero or more.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(try_from = "f64")]
pub struct NonNegativeF64(pub(crate) f64);

impl TryFrom<f64> for NonNegativeF64 {
    type Error = String;

    /// Accepts a finite number of zero or more.
    fn try_from(number: f64) -> Result<Self, Self::Error> {
        if !(number.is_finite() && number >= 0.0) {
            return Err(format!("`{number}` must be 0 or more"));
        }

        Ok(NonNegativeF64(number))
    }
}

impl FromStr for NonNegativeF64 {
    type Err = String;

    /// Parses a finite number of zero or more.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let number = value
            .parse::<f64>()
            .map_err(|_| format!("`{value}` is not a number"))?;

        NonNegativeF64::try_from(number)
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::NonNegativeF64;

    #[test]
    fn zero_and_up_parse_and_the_rest_is_refused() {
        assert_eq!("0".parse::<NonNegativeF64>(), Ok(NonNegativeF64(0.0)));
        assert_eq!("2.5".parse::<NonNegativeF64>(), Ok(NonNegativeF64(2.5)));
        assert!("-1".parse::<NonNegativeF64>().is_err());
        assert!("nan".parse::<NonNegativeF64>().is_err());
        assert!("inf".parse::<NonNegativeF64>().is_err());
        assert!(serde_json::from_str::<NonNegativeF64>("-0.5").is_err());
    }
}
