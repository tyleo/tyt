use crate::PositiveF64;
use serde::Deserialize;
use ty_math::TyVector3F64;
use voxsmith::operations::object::PositionTransform;

/// A profile's point light transform. Its `kind` takes a `--light-frame`
/// value or `orbit`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PositionTransformEntry {
    World {
        position: [f64; 3],
    },

    Subject {
        position: [f64; 3],
    },

    Camera {
        position: [f64; 3],
    },

    /// Angles in degrees and a distance in meters.
    Orbit {
        azimuth: f64,

        elevation: f64,

        distance: PositiveF64,
    },

    /// `path` mirrors `--light-node`.
    Node {
        path: String,
        position: [f64; 3],
    },
}

impl PositionTransformEntry {
    /// The transform in voxsmith's shape.
    pub(crate) fn into_transform(self) -> PositionTransform {
        match self {
            PositionTransformEntry::World { position } => PositionTransform::World {
                position: TyVector3F64::from_array(position),
            },

            PositionTransformEntry::Subject { position } => PositionTransform::Subject {
                position: TyVector3F64::from_array(position),
            },

            PositionTransformEntry::Camera { position } => PositionTransform::Camera {
                position: TyVector3F64::from_array(position),
            },

            PositionTransformEntry::Orbit {
                azimuth,
                elevation,
                distance,
            } => PositionTransform::Orbit {
                azimuth,
                elevation,
                distance: distance.0,
            },

            PositionTransformEntry::Node { path, position } => PositionTransform::Node {
                path,
                position: TyVector3F64::from_array(position),
            },
        }
    }
}
