/// Parses a finite number for a clap argument, rejecting NaN and infinities.
pub fn parse_finite_f64(text: &str) -> Result<f64, String> {
    let number = text
        .parse::<f64>()
        .map_err(|_| format!("`{text}` is not a number"))?;

    if !number.is_finite() {
        return Err(format!("`{text}` is not a finite number"));
    }

    Ok(number)
}

#[cfg(test)]
mod tests {
    use crate::parse_finite_f64;

    #[test]
    fn parses_a_finite_number() {
        assert_eq!(parse_finite_f64("-2.5"), Ok(-2.5));
    }

    #[test]
    fn rejects_non_finite_and_non_numbers() {
        assert!(parse_finite_f64("nan").is_err());
        assert!(parse_finite_f64("inf").is_err());
        assert!(parse_finite_f64("-infinity").is_err());
        assert!(parse_finite_f64("1e999").is_err());
        assert!(parse_finite_f64("abc").is_err());
    }
}
