use crate::commands::RotationEntry;
use serde::Deserialize;
use voxsmith::operations::object::RotationTransform;

/// A profile's directional light transform. Its `kind` takes a
/// `--light-frame` value.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum RotationTransformEntry {
    World {
        rotation: RotationEntry,
    },

    Camera {
        rotation: RotationEntry,
    },

    /// `path` mirrors `--light-node`.
    Node {
        path: String,
        rotation: RotationEntry,
    },
}

impl RotationTransformEntry {
    /// The transform in voxsmith's shape.
    pub(crate) fn into_transform(self) -> RotationTransform {
        match self {
            RotationTransformEntry::World { rotation } => RotationTransform::World {
                rotation: rotation.to_rotation(),
            },

            RotationTransformEntry::Camera { rotation } => RotationTransform::Camera {
                rotation: rotation.to_rotation(),
            },

            RotationTransformEntry::Node { path, rotation } => RotationTransform::Node {
                path,
                rotation: rotation.to_rotation(),
            },
        }
    }
}
