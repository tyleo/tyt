use ty_math::{TyMatrix4x4F64, TyQuaternionF64, TyVector3F64};

/// The near plane in world units. Reversed depth keeps f32's relative
/// precision at every distance, so the plane sits far nearer than any cell.
const NEAR: f64 = 1.0 / 1024.0 / 1024.0;

/// One eye of a [`BoxView`](crate::BoxView)'s frame.
#[derive(Clone, Copy, Debug)]
pub struct ViewEye {
    /// The matrix that carries a world point to the eye's clip space. Depth
    /// runs reversed.
    pub world_to_clip: TyMatrix4x4F64,

    /// The eye in the world.
    pub position: TyVector3F64,
}

impl ViewEye {
    /// The eye at `position`, looking down `rotation`'s -Z with +Y up,
    /// through a frustum whose sides sit at the tangents `left`, `right`,
    /// `up`, and `down` of their angles from the axis. A side below or left
    /// of the axis has a negative tangent, as OpenXR's fields of view do.
    pub fn new(
        position: TyVector3F64,
        rotation: TyQuaternionF64,
        left: f64,
        right: f64,
        up: f64,
        down: f64,
    ) -> Self {
        let across = right - left;
        let along = up - down;

        // Each row gives one clip component of a point in the eye's space.
        let rows = [
            [2.0 / across, 0.0, (right + left) / across, 0.0],
            [0.0, 2.0 / along, (up + down) / along, 0.0],
            [0.0, 0.0, 0.0, NEAR],
            [0.0, 0.0, -1.0, 0.0],
        ];

        let world_to_eye = TyMatrix4x4F64::from_rotation_translation(rotation, position).inverse();

        ViewEye {
            world_to_clip: TyMatrix4x4F64::from_cols_array_2d(&rows).transpose() * world_to_eye,
            position,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ViewEye;
    use ty_math::{TyQuaternionF64, TyVector3F64};

    #[test]
    fn a_frustums_sides_land_on_the_clip_box() {
        let eye = ViewEye::new(
            TyVector3F64::new(1.0, 2.0, 3.0),
            TyQuaternionF64::from_rotation_y(90f64.to_radians()),
            -0.5,
            1.5,
            1.0,
            -2.0,
        );

        // The eye looks down the world's -X. Its right is the world's -Z.
        let at = |right: f64, up: f64, depth: f64| {
            let point = TyVector3F64::new(1.0 - depth, 2.0 + up * depth, 3.0 - right * depth);
            let clip = eye.world_to_clip * point.extend(1.0);

            (clip.truncate() / clip.w).to_array()
        };

        let close = |a: [f64; 3], b: [f64; 3]| a.iter().zip(b).all(|(a, b)| (a - b).abs() < 1e-9);

        assert!(close(
            at(-0.5, -2.0, 4.0),
            [-1.0, -1.0, 0.25 / 1024.0 / 1024.0]
        ));
        assert!(close(at(1.5, 1.0, 0.5), [1.0, 1.0, 2.0 / 1024.0 / 1024.0]));
        assert!(close(at(0.5, -0.5, 1.0), [0.0, 0.0, 1.0 / 1024.0 / 1024.0]));
    }
}
