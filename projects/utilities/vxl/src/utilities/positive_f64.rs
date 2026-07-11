/// Parses a positive `f64` flag value, rejecting zero, negatives, and NaN so
/// `--voxel-size` and `--scale` fail at clap parse time.
pub(crate) fn parse_positive_f64(value: &str) -> Result<f64, String> {
    let number = value
        .parse::<f64>()
        .map_err(|_| format!("`{value}` is not a number"))?;

    if number <= 0.0 || number.is_nan() {
        return Err(format!("`{value}` must be greater than 0"));
    }

    Ok(number)
}

#[cfg(test)]
mod tests {
    use crate::parse_positive_f64;

    #[test]
    fn accepts_a_positive_number() {
        assert_eq!(parse_positive_f64("0.25"), Ok(0.25));
    }

    #[test]
    fn rejects_zero_negative_and_nan() {
        assert!(parse_positive_f64("0").is_err());
        assert!(parse_positive_f64("-1").is_err());
        assert!(parse_positive_f64("nan").is_err());
    }

    #[test]
    fn rejects_a_non_number() {
        assert!(parse_positive_f64("abc").is_err());
    }
}
