use crate::PositiveF64;
use serde::Deserialize;
use voxsmith::operations::object::FitOrFixed;

/// A profile's orbit distance, `fit` or meters.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(try_from = "DistanceRepr")]
pub struct DistanceEntry(pub(crate) FitOrFixed);

#[derive(Deserialize)]
#[serde(untagged)]
enum DistanceRepr {
    Keyword(String),

    Meters(f64),
}

impl TryFrom<DistanceRepr> for DistanceEntry {
    type Error = String;

    fn try_from(repr: DistanceRepr) -> Result<Self, String> {
        match repr {
            DistanceRepr::Keyword(keyword) if keyword == "fit" => {
                Ok(DistanceEntry(FitOrFixed::Fit))
            }

            DistanceRepr::Keyword(keyword) => {
                Err(format!("`{keyword}` is not a distance; use fit or meters"))
            }

            DistanceRepr::Meters(meters) => Ok(DistanceEntry(FitOrFixed::Fixed(
                PositiveF64::try_from(meters)?.0,
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::DistanceEntry;
    use voxsmith::operations::object::FitOrFixed;

    #[test]
    fn fit_or_a_positive_number() {
        assert_eq!(
            serde_json::from_str::<DistanceEntry>(r#""fit""#).unwrap(),
            DistanceEntry(FitOrFixed::Fit)
        );
        assert_eq!(
            serde_json::from_str::<DistanceEntry>("2.5").unwrap(),
            DistanceEntry(FitOrFixed::Fixed(2.5))
        );
        assert!(serde_json::from_str::<DistanceEntry>("0").is_err());
        assert!(serde_json::from_str::<DistanceEntry>(r#""near""#).is_err());
    }
}
