use crate::NamedCliValue;
use serde::Deserialize;
use ty_math::{TyAngleUnit, TyQuaternionF64, TyVector3F64};
use voxsmith::operations::object::Rotation;

/// A profile's rotation. Its `kind` picks the rotation flag it mirrors.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum RotationEntry {
    /// Mirrors `--view-quaternion` and `--light-quaternion`.
    Quaternion { value: [f64; 4] },

    /// Mirrors `--view-euler` and `--light-euler`. `unit` defaults to degrees.
    Euler {
        value: [f64; 3],

        unit: Option<NamedCliValue<TyAngleUnit>>,
    },

    /// Mirrors `--view-look-at` and `--light-look-at`. `target` defaults to
    /// the frame's origin.
    LookAt { target: Option<[f64; 3]> },

    /// Mirrors `--view-angles` and `--light-angles`, in degrees.
    Angles { azimuth: f64, elevation: f64 },
}

impl RotationEntry {
    /// The rotation in voxsmith's shape.
    pub(crate) fn to_rotation(self) -> Rotation {
        match self {
            RotationEntry::Quaternion {
                value: [x, y, z, w],
            } => Rotation::Quaternion {
                value: TyQuaternionF64::from_xyzw(x, y, z, w),
            },

            RotationEntry::Euler { value, unit } => Rotation::Euler {
                value: TyVector3F64::from_array(value),
                unit: unit.map_or(TyAngleUnit::Degrees, |unit| unit.0),
            },

            RotationEntry::LookAt { target } => Rotation::LookAt {
                target: target.map(TyVector3F64::from_array),
            },

            RotationEntry::Angles { azimuth, elevation } => Rotation::Angles { azimuth, elevation },
        }
    }
}
