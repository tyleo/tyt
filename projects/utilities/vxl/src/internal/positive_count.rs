use std::str::FromStr;

/// A count of at least one, parsed from a decimal string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositiveCount(pub usize);

impl FromStr for PositiveCount {
    type Err = String;

    /// Parses a decimal count of at least one.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.parse::<usize>() {
            Ok(count) if count >= 1 => Ok(PositiveCount(count)),
            _ => Err(format!("`{value}` is not a count of at least 1")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::PositiveCount;

    #[test]
    fn parses_a_positive_count() {
        assert_eq!("256".parse::<PositiveCount>().unwrap(), PositiveCount(256));
    }

    #[test]
    fn rejects_zero_and_non_numbers() {
        assert!("0".parse::<PositiveCount>().is_err());
        assert!("lots".parse::<PositiveCount>().is_err());
    }
}
