use crate::operations::sdf_doc::lattice_hash;
use std::f64::consts::FRAC_1_SQRT_2;
use ty_math::TyVector3F64;

/// The twelve unit gradients the hash picks among, in hash order.
const GRADIENTS: [TyVector3F64; 12] = [
    TyVector3F64::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0),
    TyVector3F64::new(-FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0),
    TyVector3F64::new(FRAC_1_SQRT_2, -FRAC_1_SQRT_2, 0.0),
    TyVector3F64::new(-FRAC_1_SQRT_2, -FRAC_1_SQRT_2, 0.0),
    TyVector3F64::new(FRAC_1_SQRT_2, 0.0, FRAC_1_SQRT_2),
    TyVector3F64::new(-FRAC_1_SQRT_2, 0.0, FRAC_1_SQRT_2),
    TyVector3F64::new(FRAC_1_SQRT_2, 0.0, -FRAC_1_SQRT_2),
    TyVector3F64::new(-FRAC_1_SQRT_2, 0.0, -FRAC_1_SQRT_2),
    TyVector3F64::new(0.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    TyVector3F64::new(0.0, -FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    TyVector3F64::new(0.0, FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
    TyVector3F64::new(0.0, -FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
];

/// Gradient noise at `point` under `seed`, blending the eight corners of the
/// lattice cube around the point with the quintic fade.
pub fn gradient_noise(point: TyVector3F64, seed: u32) -> f64 {
    let floor = point.floor();
    let f = point - floor;
    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);

    // The lattice wraps as 32-bit two's-complement integers.
    let cell = [floor.x, floor.y, floor.z].map(|coordinate| coordinate as i64 as i32);

    let corner = |dx: i32, dy: i32, dz: i32| {
        let coordinates = [
            cell[0].wrapping_add(dx),
            cell[1].wrapping_add(dy),
            cell[2].wrapping_add(dz),
        ];
        let gradient = GRADIENTS[(lattice_hash(seed, &coordinates) % 12) as usize];
        gradient.dot(f - TyVector3F64::new(f64::from(dx), f64::from(dy), f64::from(dz)))
    };

    let va = corner(0, 0, 0);
    let vb = corner(1, 0, 0);
    let vc = corner(0, 1, 0);
    let vd = corner(1, 1, 0);
    let ve = corner(0, 0, 1);
    let vf = corner(1, 0, 1);
    let vg = corner(0, 1, 1);
    let vh = corner(1, 1, 1);

    va + u.x * (vb - va)
        + u.y * (vc - va)
        + u.z * (ve - va)
        + u.x * u.y * (va - vb - vc + vd)
        + u.y * u.z * (va - vc - ve + vg)
        + u.z * u.x * (va - vb - ve + vf)
        + (-va + vb + vc - vd + ve - vf - vg + vh) * u.x * u.y * u.z
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::gradient_noise;
    use ty_math::TyVector3F64;

    #[test]
    fn the_noise_reads_zero_on_the_lattice() {
        for point in [
            TyVector3F64::ZERO,
            TyVector3F64::new(3.0, -2.0, 7.0),
            TyVector3F64::new(-1.0, 1.0, -1.0),
        ] {
            assert_eq!(gradient_noise(point, 5), 0.0);
        }
    }

    #[test]
    fn the_noise_runs_continuously_across_a_cube_face() {
        let below = gradient_noise(TyVector3F64::new(0.3, 0.999_999_9, 0.6), 2);
        let above = gradient_noise(TyVector3F64::new(0.3, 1.000_000_1, 0.6), 2);

        assert!((below - above).abs() < 1e-6);
    }
}
