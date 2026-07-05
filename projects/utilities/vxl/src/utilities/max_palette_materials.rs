use std::str::FromStr;

/// The `--max-palette-materials` value: a cap on the number of materials a
/// palette may hold, or `none` for no cap. Shared by `voxelize` and the
/// palette-reduction commands, which reduce a palette to at most this many
/// materials.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaxPaletteMaterials {
    /// `none`: no cap; the palette keeps every material.
    Unlimited,

    /// At most this many materials; at least one.
    Materials(usize),
}

impl MaxPaletteMaterials {
    /// The cap as an `Option`, `None` for `none`.
    pub fn limit(self) -> Option<usize> {
        match self {
            MaxPaletteMaterials::Unlimited => None,
            MaxPaletteMaterials::Materials(materials) => Some(materials),
        }
    }
}

impl FromStr for MaxPaletteMaterials {
    type Err = String;

    /// Parses `none` or a positive material count.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("none") {
            return Ok(MaxPaletteMaterials::Unlimited);
        }

        match value.parse::<usize>() {
            Ok(materials) if materials >= 1 => Ok(MaxPaletteMaterials::Materials(materials)),
            _ => Err(format!(
                "`{value}` is not a material cap; use none or a count of at least 1"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::MaxPaletteMaterials;

    #[test]
    fn parses_none_and_a_count() {
        assert_eq!(
            "none".parse::<MaxPaletteMaterials>().unwrap(),
            MaxPaletteMaterials::Unlimited
        );
        assert_eq!(
            "256".parse::<MaxPaletteMaterials>().unwrap(),
            MaxPaletteMaterials::Materials(256)
        );
    }

    #[test]
    fn rejects_zero_and_non_numbers() {
        assert!("0".parse::<MaxPaletteMaterials>().is_err());
        assert!("lots".parse::<MaxPaletteMaterials>().is_err());
    }

    #[test]
    fn limit_maps_unlimited_to_none() {
        assert_eq!(MaxPaletteMaterials::Unlimited.limit(), None);
        assert_eq!(MaxPaletteMaterials::Materials(8).limit(), Some(8));
    }
}
