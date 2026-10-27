use crate::operations::object::{Rotation, look_rotation};
use ty_math::{TyAngleUnit, TyQuaternionExt, TyQuaternionF64, TyVector3Ext, TyVector3F64};

/// Resolves `rotation` in its frame for an entity at `eye` in that frame.
/// Returns `None` when a look-at aims at `eye` itself.
pub fn resolve_rotation(rotation: &Rotation, eye: TyVector3F64) -> Option<TyQuaternionF64> {
    match *rotation {
        Rotation::Quaternion { value } => Some(value),

        Rotation::Euler { value, unit } => Some(TyQuaternionF64::from_euler_radians(
            value.map(|angle| unit.to_radians(angle)),
        )),

        Rotation::LookAt { target } => look_rotation(target.unwrap_or(TyVector3F64::ZERO) - eye),

        Rotation::Angles { azimuth, elevation } => {
            let direction = TyVector3F64::from_azimuth_elevation(
                TyAngleUnit::Degrees.to_radians(azimuth),
                TyAngleUnit::Degrees.to_radians(elevation),
            );

            look_rotation(-direction)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::object::{Rotation, resolve_rotation};
    use ty_math::{TyAngleUnit, TyQuaternionExt, TyQuaternionF64, TyVector3Ext, TyVector3F64};

    fn close(a: TyVector3F64, b: TyVector3F64) -> bool {
        a.is_normalized_approximately_equal(b, 1e-9)
    }

    #[test]
    fn each_form_resolves_in_its_frame() {
        let eye = TyVector3F64::new(0.0, 0.0, 5.0);

        let quaternion = TyQuaternionF64::from_axis_angle(TyVector3F64::Y, 1.0);
        assert_eq!(
            resolve_rotation(&Rotation::Quaternion { value: quaternion }, eye),
            Some(quaternion)
        );

        let euler = resolve_rotation(
            &Rotation::Euler {
                value: TyVector3F64::new(0.0, 90.0, 0.0),
                unit: TyAngleUnit::Degrees,
            },
            eye,
        )
        .unwrap();
        assert!(euler.is_approximately_equal(
            TyQuaternionF64::from_euler_radians(TyVector3F64::new(
                0.0,
                1.0f64.to_radians() * 90.0,
                0.0
            )),
            1e-9
        ));

        // The default target is the frame's origin, straight ahead of the eye.
        let look = resolve_rotation(&Rotation::LookAt { target: None }, eye).unwrap();
        assert!(close(look * -TyVector3F64::Z, -TyVector3F64::Z));
        assert_eq!(
            resolve_rotation(&Rotation::LookAt { target: Some(eye) }, eye),
            None
        );

        // Something 90 degrees of azimuth out sits on +X and faces -X.
        let angles = resolve_rotation(
            &Rotation::Angles {
                azimuth: 90.0,
                elevation: 0.0,
            },
            eye,
        )
        .unwrap();
        assert!(close(angles * -TyVector3F64::Z, -TyVector3F64::X));
    }
}
