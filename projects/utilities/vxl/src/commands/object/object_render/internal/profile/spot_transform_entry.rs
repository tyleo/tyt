use crate::{PositiveF64, commands::RotationEntry};
use serde::Deserialize;
use ty_math::TyVector3F64;
use voxsmith::operations::object::SpotTransform;

/// A profile's spot light transform. Its `kind` takes a `--light-frame`
/// value or `orbit`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum SpotTransformEntry {
    World {
        position: [f64; 3],

        rotation: RotationEntry,
    },

    Subject {
        position: [f64; 3],

        rotation: RotationEntry,
    },

    Camera {
        position: [f64; 3],

        rotation: RotationEntry,
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

        rotation: RotationEntry,
    },
}

impl SpotTransformEntry {
    /// The transform in voxsmith's shape.
    pub(crate) fn into_transform(self) -> SpotTransform {
        match self {
            SpotTransformEntry::World { position, rotation } => SpotTransform::World {
                position: TyVector3F64::from_array(position),
                rotation: rotation.to_rotation(),
            },

            SpotTransformEntry::Subject { position, rotation } => SpotTransform::Subject {
                position: TyVector3F64::from_array(position),
                rotation: rotation.to_rotation(),
            },

            SpotTransformEntry::Camera { position, rotation } => SpotTransform::Camera {
                position: TyVector3F64::from_array(position),
                rotation: rotation.to_rotation(),
            },

            SpotTransformEntry::Orbit {
                azimuth,
                elevation,
                distance,
            } => SpotTransform::Orbit {
                azimuth,
                elevation,
                distance: distance.0,
            },

            SpotTransformEntry::Node {
                path,
                position,
                rotation,
            } => SpotTransform::Node {
                path,
                position: TyVector3F64::from_array(position),
                rotation: rotation.to_rotation(),
            },
        }
    }
}
