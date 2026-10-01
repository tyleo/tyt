use crate::commands::{DistanceEntry, RotationEntry};
use serde::Deserialize;
use ty_math::TyVector3F64;
use voxsmith::operations::object::{FitOrFixed, PoseTransform};

/// A profile's view transform. Its `kind` takes a `--view-frame` value or
/// `orbit`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PoseTransformEntry {
    World {
        position: [f64; 3],

        rotation: RotationEntry,
    },

    Subject {
        position: [f64; 3],

        rotation: RotationEntry,
    },

    /// Angles in degrees. `distance` defaults to `fit`.
    Orbit {
        azimuth: f64,

        elevation: f64,

        distance: Option<DistanceEntry>,
    },

    /// `path` mirrors `--view-node`.
    Node {
        path: String,

        position: [f64; 3],

        rotation: RotationEntry,
    },
}

impl PoseTransformEntry {
    /// The transform in voxsmith's shape.
    pub(crate) fn into_transform(self) -> PoseTransform {
        match self {
            PoseTransformEntry::World { position, rotation } => PoseTransform::World {
                position: TyVector3F64::from_array(position),
                rotation: rotation.to_rotation(),
            },

            PoseTransformEntry::Subject { position, rotation } => PoseTransform::Subject {
                position: TyVector3F64::from_array(position),
                rotation: rotation.to_rotation(),
            },

            PoseTransformEntry::Orbit {
                azimuth,
                elevation,
                distance,
            } => PoseTransform::Orbit {
                azimuth,
                elevation,
                distance: distance.map_or(FitOrFixed::Fit, |distance| distance.0),
            },

            PoseTransformEntry::Node {
                path,
                position,
                rotation,
            } => PoseTransform::Node {
                path,
                position: TyVector3F64::from_array(position),
                rotation: rotation.to_rotation(),
            },
        }
    }
}
