use crate::FindNonFinite;
use serde::{Serialize, Serializer, ser::Error as SerError};

/// Serializes a field whose numbers are all finite. A NaN or an infinity
/// errors because serde_json would write either as `null`.
pub fn finite<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: FindNonFinite + Serialize,
    S: Serializer,
{
    if let Some(number) = value.find_non_finite() {
        return Err(SerError::custom(format!(
            "number must be finite, not {number}"
        )));
    }

    value.serialize(serializer)
}

#[cfg(test)]
mod tests {
    use crate::{SdfjIntValue, SdfjPropertyValue, SdfjShades, SdfjShape2d, SdfjShape3d};

    #[test]
    fn finite_numbers_write() {
        assert_eq!(
            serde_json::to_string(&SdfjShape3d::Sphere {
                center: [0.0, 1.0, 0.0],
                radius: 0.5,
            })
            .unwrap(),
            r#"{"kind":"sphere","center":[0.0,1.0,0.0],"radius":0.5}"#,
        );
    }

    #[test]
    fn non_finite_numbers_error_on_write() {
        for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                serde_json::to_string(&SdfjShape3d::Sphere {
                    center: [0.0, number, 0.0],
                    radius: 0.5,
                })
                .is_err()
            );

            assert!(
                serde_json::to_string(&SdfjShape3d::Box {
                    min: [0.0; 3],
                    max: [1.0; 3],
                    round: Some(number),
                })
                .is_err()
            );

            assert!(
                serde_json::to_string(&SdfjShape2d::Polygon {
                    points: vec![[0.0, 0.0], [1.0, number]],
                })
                .is_err()
            );

            assert!(serde_json::to_string(&SdfjIntValue::NumberArray(vec![number])).is_err());

            assert!(serde_json::to_string(&SdfjPropertyValue::Number(number)).is_err());

            assert!(
                serde_json::to_string(&SdfjShades {
                    base: 0,
                    count: number,
                    spread: None,
                })
                .is_err()
            );
        }
    }
}
