use ty_math::TyVector3F64;

/// The shadow a box of opaque cells casts from an eye behind the face it
/// shows the eye widest.
pub struct FaceShadow {
    /// The face's solid angle from the eye, near enough to rank shadows by.
    pub wide: f64,

    /// The face's plane, then the four planes through the eye and the
    /// face's edges, each as a unit normal and an offset in world meters. A
    /// point `p` lies in the shadow when `normal . p + offset` is no less
    /// than zero on all five.
    pub planes: [[f64; 4]; 5],
}

impl FaceShadow {
    /// The shadow from `eye` of the box with `corners`, all in world
    /// meters. Bit `a` of a corner's index picks the box's far side on axis
    /// `a`. `None` where the eye faces none of the box's faces from
    /// outside.
    pub fn new(corners: &[TyVector3F64; 8], eye: TyVector3F64) -> Option<Self> {
        // The face the eye sees widest: its corners in turn around it, the
        // unit normal that points away from the eye, and how wide it is.
        let mut widest: Option<([TyVector3F64; 4], TyVector3F64, f64)> = None;

        for axis in 0..3 {
            let (u, v) = (1 << ((axis + 1) % 3), 1 << ((axis + 2) % 3));

            for side in [0, 1 << axis] {
                let face = [side, side | u, side | u | v, side | v].map(|corner| corners[corner]);
                let to = (face[0] + face[2]) / 2.0 - eye;

                // The box lies past the face along this way.
                let inward = corners[side ^ (1 << axis)] - corners[side];
                let across = (face[1] - face[0]).cross(face[3] - face[0]);

                let Some(normal) = across.try_normalize() else {
                    continue;
                };

                let normal = normal * normal.dot(inward).signum();
                let wide = across.length() * normal.dot(to) / to.length().powi(3);

                if wide > widest.map_or(0.0, |(.., widest)| widest) {
                    widest = Some((face, normal, wide));
                }
            }
        }

        let (face, normal, wide) = widest?;
        let center = (face[0] + face[2]) / 2.0;
        let mut planes = [normal.extend(-normal.dot(center)).to_array(); 5];

        for edge in 0..4 {
            let side = (face[edge] - eye)
                .cross(face[(edge + 1) % 4] - eye)
                .try_normalize()?;
            let side = side * side.dot(center - eye).signum();

            planes[edge + 1] = side.extend(-side.dot(eye)).to_array();
        }

        Some(FaceShadow { wide, planes })
    }
}

#[cfg(test)]
mod tests {
    use crate::FaceShadow;
    use std::f64::consts::FRAC_PI_2;
    use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64};

    /// Whether `point` lies in `shadow`.
    fn shades(shadow: &FaceShadow, point: [f64; 3]) -> bool {
        shadow.planes.iter().all(|plane| {
            plane[0] * point[0] + plane[1] * point[1] + plane[2] * point[2] + plane[3] >= 0.0
        })
    }

    /// The corners of a wall 8 m wide and high and 2 m thick that stands
    /// across the world's +X from x = 10 to x = 12. Its first axis runs
    /// down the world's Z and its third up the world's X.
    fn wall() -> [TyVector3F64; 8] {
        let transform = TyTransformF64 {
            position: TyVector3F64::new(10.0, 0.0, 4.0),
            rotation: TyQuaternionF64::from_rotation_y(FRAC_PI_2),
            scale: TyVector3F64::splat(2.0),
        };

        [0, 1, 2, 3, 4, 5, 6, 7].map(|corner: u32| {
            let far = |axis: u32, cells: f64| f64::from(corner >> axis & 1) * cells;

            transform.transform_point(TyVector3F64::new(far(0, 4.0), far(1, 4.0), far(2, 1.0)))
        })
    }

    #[test]
    fn a_wall_shades_what_lies_behind_its_face() {
        let eye = TyVector3F64::new(0.0, 4.0, 0.0);
        let shadow = FaceShadow::new(&wall(), eye).expect("the eye faces the wall");

        // The wall's near face spans y and z from 0 to 8 and -4 to 4 at
        // x = 10.
        assert!(shades(&shadow, [20.0, 4.0, 0.0]));
        assert!(shades(&shadow, [10.5, 7.5, 3.5]));
        assert!(shades(&shadow, [30.0, 12.0, -11.0]));

        // Before the face, and past each of its edges.
        assert!(!shades(&shadow, [9.5, 4.0, 0.0]));
        assert!(!shades(&shadow, [20.0, 13.0, 0.0]));
        assert!(!shades(&shadow, [20.0, -5.0, 0.0]));
        assert!(!shades(&shadow, [20.0, 4.0, 9.0]));
        assert!(!shades(&shadow, [20.0, 4.0, -9.0]));

        // A face 64 m2 wide, 10 m off.
        assert!((shadow.wide - 0.64).abs() < 1e-9);
    }

    #[test]
    fn an_eye_inside_a_box_has_no_shadow() {
        let eye = TyVector3F64::new(11.0, 4.0, 0.0);

        assert!(FaceShadow::new(&wall(), eye).is_none());
    }

    #[test]
    fn an_eye_takes_the_face_it_sees_widest() {
        // An eye above and before the wall sees its top face narrow and
        // its near face wide.
        let eye = TyVector3F64::new(0.0, 9.0, 0.0);
        let shadow = FaceShadow::new(&wall(), eye).expect("the eye faces the wall");

        assert!(shades(&shadow, [20.0, 4.0, 0.0]));
        assert!(!shades(&shadow, [11.0, 20.0, 0.0]));
    }
}
