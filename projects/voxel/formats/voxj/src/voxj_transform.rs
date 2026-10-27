#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A node transform, composing as `Translation * Rotation * Scale`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VoxjTransform {
    /// `[x, y, z]`, may be fractional.
    #[cfg_attr(feature = "serde", serde(serialize_with = "components::serialize"))]
    pub position: [f64; 3],

    /// Unit quaternion `[x, y, z, w]`.
    #[cfg_attr(feature = "serde", serde(serialize_with = "components::serialize"))]
    pub rotation: [f64; 4],

    /// Per-axis `[x, y, z]`.
    #[cfg_attr(feature = "serde", serde(serialize_with = "components::serialize"))]
    pub scale: [f64; 3],
}

/// Serde for the transform's components.
#[cfg(feature = "serde")]
mod components {
    use serde::{
        Serializer,
        ser::{Error as SerError, SerializeTuple},
    };

    /// Serializes a transform's components. A NaN or an infinity errors
    /// because serde_json would write either as `null`.
    pub fn serialize<S, const N: usize>(
        components: &[f64; N],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(component) = components.iter().find(|component| !component.is_finite()) {
            return Err(SerError::custom(format!(
                "transform component must be finite, not {component}"
            )));
        }

        let mut tuple = serializer.serialize_tuple(N)?;

        for component in components {
            tuple.serialize_element(component)?;
        }

        tuple.end()
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use crate::VoxjTransform;

    /// The identity transform.
    fn identity() -> VoxjTransform {
        VoxjTransform {
            position: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0; 3],
        }
    }

    #[test]
    fn a_finite_transform_writes_its_components_as_arrays() {
        assert_eq!(
            serde_json::to_string(&identity()).unwrap(),
            r#"{"position":[0.0,0.0,0.0],"rotation":[0.0,0.0,0.0,1.0],"scale":[1.0,1.0,1.0]}"#
        );
    }

    #[test]
    fn a_non_finite_component_errors_on_write() {
        for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for transform in [
                VoxjTransform {
                    position: [0.0, number, 0.0],
                    ..identity()
                },
                VoxjTransform {
                    rotation: [0.0, 0.0, 0.0, number],
                    ..identity()
                },
                VoxjTransform {
                    scale: [number, 1.0, 1.0],
                    ..identity()
                },
            ] {
                assert!(serde_json::to_string(&transform).is_err(), "{transform:?}");
            }
        }
    }
}
