use std::str::FromStr;

/// The `--max-palette-cells` value: a cap on the number of cells a palette may
/// hold, or `none` for no cap. Shared by `voxelize` and the palette-reduction
/// commands, which reduce a palette to at most this many cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaxPaletteCells {
    /// `none`: no cap; the palette keeps every cell.
    Unlimited,

    /// At most this many cells; at least one.
    Cells(usize),
}

impl MaxPaletteCells {
    /// The cap as an `Option`, `None` for `none`.
    pub fn limit(self) -> Option<usize> {
        match self {
            MaxPaletteCells::Unlimited => None,
            MaxPaletteCells::Cells(cells) => Some(cells),
        }
    }
}

impl FromStr for MaxPaletteCells {
    type Err = String;

    /// Parses `none` or a positive cell count.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("none") {
            return Ok(MaxPaletteCells::Unlimited);
        }

        match value.parse::<usize>() {
            Ok(cells) if cells >= 1 => Ok(MaxPaletteCells::Cells(cells)),
            _ => Err(format!(
                "`{value}` is not a cell cap; use none or a count of at least 1"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::MaxPaletteCells;

    #[test]
    fn parses_none_and_a_count() {
        assert_eq!(
            "none".parse::<MaxPaletteCells>().unwrap(),
            MaxPaletteCells::Unlimited
        );
        assert_eq!(
            "256".parse::<MaxPaletteCells>().unwrap(),
            MaxPaletteCells::Cells(256)
        );
    }

    #[test]
    fn rejects_zero_and_non_numbers() {
        assert!("0".parse::<MaxPaletteCells>().is_err());
        assert!("lots".parse::<MaxPaletteCells>().is_err());
    }

    #[test]
    fn limit_maps_unlimited_to_none() {
        assert_eq!(MaxPaletteCells::Unlimited.limit(), None);
        assert_eq!(MaxPaletteCells::Cells(8).limit(), Some(8));
    }
}
