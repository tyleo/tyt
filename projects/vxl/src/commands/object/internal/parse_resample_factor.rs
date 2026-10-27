use voxsmith::operations::object::ResampleFactor;

/// Parses a `--factor` value for a clap argument: a whole number of at least
/// 2.
pub fn parse_resample_factor(text: &str) -> Result<ResampleFactor, String> {
    let factor = text
        .parse::<u32>()
        .map_err(|_| format!("`{text}` is not a whole number"))?;

    ResampleFactor::new(factor).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use crate::commands::parse_resample_factor;

    #[test]
    fn parses_a_factor_of_at_least_two() {
        assert_eq!(parse_resample_factor("10").unwrap().get(), 10);
        assert!(parse_resample_factor("1").is_err());
        assert!(parse_resample_factor("-2").is_err());
        assert!(parse_resample_factor("2.5").is_err());
    }
}
