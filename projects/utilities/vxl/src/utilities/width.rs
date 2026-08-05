use std::str::FromStr;

/// A line-width budget for wrapped output.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Width {
    /// Wrap to the terminal width, or not at all when stdout is not a terminal.
    Terminal,

    /// Never wrap; one line per value collection.
    Unlimited,

    /// Wrap to a fixed number of columns.
    Columns(usize),
}

impl FromStr for Width {
    type Err = String;

    /// Parses `terminal`, `unlimited`, or a column count.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "terminal" => Ok(Width::Terminal),
            "unlimited" => Ok(Width::Unlimited),
            _ => value.parse::<usize>().map(Width::Columns).map_err(|_| {
                format!("`{value}` is not a width; use terminal, unlimited, or a column count")
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Width;

    #[test]
    fn parses_the_keywords_and_a_count() {
        assert_eq!("terminal".parse::<Width>().unwrap(), Width::Terminal);
        assert_eq!("unlimited".parse::<Width>().unwrap(), Width::Unlimited);
        assert_eq!("40".parse::<Width>().unwrap(), Width::Columns(40));
    }

    #[test]
    fn rejects_other_text() {
        assert!("wide".parse::<Width>().is_err());
        assert!("-1".parse::<Width>().is_err());
    }
}
