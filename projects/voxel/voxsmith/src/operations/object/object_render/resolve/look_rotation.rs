use ty_math::{TyQuaternionExt, TyQuaternionF64, TyVector3F64, UNIT_ROTATION_TOLERANCE};

/// The rotation looking along `direction` with the frame's +Y up, or -Z up
/// where `direction` runs along +Y or -Y within the unit rotation
/// tolerance. Returns `None` for a zero direction.
pub fn look_rotation(direction: TyVector3F64) -> Option<TyQuaternionF64> {
    let direction = direction.try_normalize()?;

    let up = if 1.0 - direction.y.abs() <= UNIT_ROTATION_TOLERANCE {
        -TyVector3F64::Z
    } else {
        TyVector3F64::Y
    };

    TyQuaternionF64::from_look_direction(direction, up)
}

#[cfg(test)]
mod tests {
    use crate::operations::object::look_rotation;
    use ty_math::{TyVector3Ext, TyVector3F64};

    fn close(a: TyVector3F64, b: TyVector3F64) -> bool {
        a.is_normalized_approximately_equal(b, 1e-9)
    }

    #[test]
    fn a_look_down_or_up_takes_minus_z_as_up() {
        let down = look_rotation(-TyVector3F64::Y * 2.0).unwrap();
        assert!(close(down * -TyVector3F64::Z, -TyVector3F64::Y));
        assert!(close(down * TyVector3F64::Y, -TyVector3F64::Z));

        let up = look_rotation(TyVector3F64::Y).unwrap();
        assert!(close(up * -TyVector3F64::Z, TyVector3F64::Y));
        assert!(close(up * TyVector3F64::Y, -TyVector3F64::Z));

        let ahead = look_rotation(-TyVector3F64::Z).unwrap();
        assert!(close(ahead * TyVector3F64::Y, TyVector3F64::Y));

        assert_eq!(look_rotation(TyVector3F64::ZERO), None);
    }
}
