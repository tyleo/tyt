use crate::operations::sdf_doc::{fbm, lattice_hash, noise_seed, whole_count};
use branded_id::U32Id;
use sdfcore::{BSdfMaterial, SdfBorder, SdfPattern};
use ty_math::{TyAxis3, TyVector3F64, TyVector3I32};

/// The octaves a pattern's noise sums when the model leaves them out.
const DEFAULT_OCTAVES: u32 = 4;

/// A `grain`'s warp as a share of its period when the model leaves it out.
const DEFAULT_GRAIN_WARP: f64 = 0.75;

/// The range a hash spans, as a divisor that maps it into `[0, 1)`.
const HASH_RANGE: f64 = 4_294_967_296.0;

/// The material `pattern` picks for the cell whose frame position reads `q`
/// and whose center sits at `lattice_position` on the grid's lattice.
pub fn pattern_material(
    pattern: &SdfPattern,
    q: TyVector3F64,
    lattice_position: TyVector3F64,
) -> U32Id<BSdfMaterial> {
    let seed_of = |seed: Option<f64>| seed.map_or(0, noise_seed);
    let warped = |unwarped: f64, warp: f64, scale: f64, seed: u32| {
        unwarped + warp * fbm(q / scale, DEFAULT_OCTAVES, seed)
    };

    match pattern {
        SdfPattern::Bands {
            material_ids,
            axis,
            period,
            warp,
            seed,
        } => {
            let s = warped(
                q[axis.index()],
                warp.unwrap_or(0.0),
                4.0 * period,
                seed_of(*seed),
            );
            cycled(material_ids, (s / period).floor())
        }

        SdfPattern::Bricks {
            material_ids,
            length,
            height,
            axis,
            seed,
            border,
        } => bricks_material(
            material_ids,
            *border,
            [*length, *height],
            axis.unwrap_or(TyAxis3::Y),
            noise_seed(*seed),
            q,
        ),

        SdfPattern::Cells {
            material_ids,
            size,
            seed,
            border,
        } => cells_material(material_ids, *border, *size, noise_seed(*seed), q),

        SdfPattern::Checker { material_ids, size } => {
            cycled(material_ids, (q / *size).floor().element_sum())
        }

        SdfPattern::Gradient {
            material_ids,
            axis,
            from,
            to,
            warp,
            seed,
        } => {
            let s = warped(
                q[axis.index()],
                warp.unwrap_or(0.0),
                (to - from) / 4.0,
                seed_of(*seed),
            );
            spanned(material_ids, (s - from) / (to - from))
        }

        SdfPattern::Grain {
            material_ids,
            axis,
            period,
            warp,
            seed,
        } => {
            let seed = noise_seed(*seed);
            // The grain rings the axis line by reading the distance from it.
            let mut across = q;
            across[axis.index()] = 0.0;
            let s = warped(
                across.length(),
                warp.unwrap_or(DEFAULT_GRAIN_WARP * period),
                4.0 * period,
                seed,
            );
            picked(
                material_ids,
                lattice_hash(seed, &[lattice_coordinate((s / period).floor())]),
            )
        }

        SdfPattern::Noise {
            material_ids,
            scale,
            octaves,
            seed,
        } => {
            let octaves = octaves.map_or(DEFAULT_OCTAVES, whole_count);
            let t = (fbm(q / *scale, octaves, noise_seed(*seed)) + 1.0) / 2.0;
            spanned(material_ids, t)
        }

        SdfPattern::Speckle {
            base_id,
            accent_ids,
            size,
            density,
            seed,
        } => {
            let seed = noise_seed(*seed);
            let cube = (lattice_position / *size)
                .floor()
                .to_array()
                .map(lattice_coordinate);
            let hash = |channel: i32| lattice_hash(seed, &[cube[0], cube[1], cube[2], channel]);

            if unit(hash(0)) < *density {
                picked(accent_ids, hash(1))
            } else {
                *base_id
            }
        }
    }
}

/// The material of `material_ids` that `index` reaches when it cycles
/// through them in order.
fn cycled(material_ids: &[U32Id<BSdfMaterial>], index: f64) -> U32Id<BSdfMaterial> {
    material_ids[index.rem_euclid(material_ids.len() as f64) as usize]
}

/// The material of `material_ids` whose equal span of `[0, 1]` holds `t`, with
/// the first material before the spans and the last past them.
fn spanned(material_ids: &[U32Id<BSdfMaterial>], t: f64) -> U32Id<BSdfMaterial> {
    let last = material_ids.len() - 1;
    let index = (t * material_ids.len() as f64).floor();

    material_ids[index.clamp(0.0, last as f64) as usize]
}

/// The material of `material_ids` that `hash` picks.
fn picked(material_ids: &[U32Id<BSdfMaterial>], hash: u32) -> U32Id<BSdfMaterial> {
    material_ids[hash as usize % material_ids.len()]
}

/// `hash` mapped into `[0, 1)`.
fn unit(hash: u32) -> f64 {
    f64::from(hash) / HASH_RANGE
}

/// The whole number `value` as the 32-bit two's-complement coordinate the
/// hash reads.
fn lattice_coordinate(value: f64) -> i32 {
    value as i64 as i32
}

/// The material a `bricks` pattern picks at the frame position `q`. The head
/// joints sit a quarter brick off the multiples of `length`, so a face built at
/// a whole or half brick misses them.
fn bricks_material(
    material_ids: &[U32Id<BSdfMaterial>],
    border: Option<SdfBorder>,
    [length, height]: [f64; 2],
    axis: TyAxis3,
    seed: u32,
    q: TyVector3F64,
) -> U32Id<BSdfMaterial> {
    let course = (q[axis.index()] / height).floor();
    let shift = 0.25 + 0.5 * course.rem_euclid(2.0);

    let mut brick = [0; 3];
    let mut in_joint = false;

    for index in 0..3 {
        let (s, size) = if index == axis.index() {
            (q[index] / height, height)
        } else {
            (q[index] / length - shift, length)
        };
        let floor = s.floor();
        let within = s - floor;

        brick[index] = lattice_coordinate(floor);

        // Each brick opens with its joint.
        if let Some(SdfBorder { width, .. }) = border {
            in_joint |= size * within < width;
        }
    }

    match border {
        Some(SdfBorder { material_id, .. }) if in_joint => material_id,
        _ => picked(material_ids, lattice_hash(seed, &brick)),
    }
}

/// The material a `cells` pattern picks at the frame position `q`. Each
/// lattice cube `size` wide holds one feature point, and the cell takes the
/// nearest point's pick, or the border's material within half its width of
/// the seam between two points' cells.
fn cells_material(
    material_ids: &[U32Id<BSdfMaterial>],
    border: Option<SdfBorder>,
    size: f64,
    seed: u32,
    q: TyVector3F64,
) -> U32Id<BSdfMaterial> {
    let p = q / size;
    let floor = p.floor();
    let within = p - floor;
    let cube = [floor.x, floor.y, floor.z].map(lattice_coordinate);

    let hash = |offset: TyVector3I32, channel: i32| {
        lattice_hash(
            seed,
            &[
                cube[0].wrapping_add(offset.x),
                cube[1].wrapping_add(offset.y),
                cube[2].wrapping_add(offset.z),
                channel,
            ],
        )
    };

    // The offset from the cell to the feature point of the cube at `offset`.
    let point = |offset: TyVector3I32| {
        let jitter = TyVector3F64::new(
            unit(hash(offset, 0)),
            unit(hash(offset, 1)),
            unit(hash(offset, 2)),
        );
        offset.as_dvec3() + jitter - within
    };

    let mut nearest_offset = TyVector3I32::ZERO;
    let mut nearest = point(nearest_offset);
    let mut nearest_distance = f64::INFINITY;

    for offset in cube_offsets(1) {
        let r = point(offset);
        let distance = r.dot(r);

        if distance < nearest_distance {
            nearest_offset = offset;
            nearest = r;
            nearest_distance = distance;
        }
    }

    if let Some(SdfBorder { material_id, width }) = border {
        for offset in cube_offsets(2) {
            let offset = nearest_offset + offset;

            if offset == nearest_offset {
                continue;
            }

            let r = point(offset);
            let seam = ((nearest + r) / 2.0).dot((r - nearest).normalize());

            if size * seam <= width / 2.0 {
                return material_id;
            }
        }
    }

    picked(material_ids, hash(nearest_offset, 3))
}

/// The offsets to the cubes within `reach` cubes of a cube on each axis, in
/// raster order with x outermost.
fn cube_offsets(reach: i32) -> impl Iterator<Item = TyVector3I32> {
    (-reach..=reach).flat_map(move |x| {
        (-reach..=reach)
            .flat_map(move |y| (-reach..=reach).map(move |z| TyVector3I32::new(x, y, z)))
    })
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::pattern_material;
    use branded_id::U32Id;
    use sdfcore::{BSdfMaterial, SdfBorder, SdfPattern};
    use std::f64::consts::TAU;
    use ty_math::{TyAxis3, TyVector3F64};

    /// The materials `0` to `count - 1`.
    fn materials(count: u32) -> Vec<U32Id<BSdfMaterial>> {
        (0..count).map(U32Id::from_u32).collect()
    }

    /// The index of the material `pattern` picks at `q` for the cell centered
    /// at `lattice_position`.
    fn pick(pattern: &SdfPattern, q: TyVector3F64, lattice_position: TyVector3F64) -> u32 {
        pattern_material(pattern, q, lattice_position).to_u32()
    }

    /// The index of the material `pattern` picks at `q`, in whichever cell.
    fn pick_at(pattern: &SdfPattern, x: f64, y: f64, z: f64) -> u32 {
        pick(pattern, TyVector3F64::new(x, y, z), TyVector3F64::ZERO)
    }

    #[test]
    fn bands_cycle_through_the_materials_one_period_thick() {
        let bands = SdfPattern::Bands {
            material_ids: materials(3),
            axis: TyAxis3::Y,
            period: 0.1,
            warp: None,
            seed: None,
        };

        let picks: Vec<u32> = (-2..5)
            .map(|cell| pick_at(&bands, 0.3, (f64::from(cell) + 0.5) * 0.1, 0.7))
            .collect();
        assert_eq!(picks, [1, 2, 0, 1, 2, 0, 1]);
    }

    #[test]
    fn a_warp_bends_the_bands() {
        let bands = |warp: Option<f64>| SdfPattern::Bands {
            material_ids: materials(2),
            axis: TyAxis3::X,
            period: 0.5,
            warp,
            seed: Some(4.0),
        };

        let straight: Vec<u32> = (0..50)
            .map(|step| pick_at(&bands(None), 0.25, f64::from(step) * 0.1, 0.0))
            .collect();
        let bent: Vec<u32> = (0..50)
            .map(|step| pick_at(&bands(Some(1.0)), 0.25, f64::from(step) * 0.1, 0.0))
            .collect();

        assert!(straight.iter().all(|pick| *pick == 0));
        assert!(bent.contains(&1));
    }

    #[test]
    fn a_grain_ring_around_the_axis_takes_one_hashed_pick() {
        let grain = SdfPattern::Grain {
            material_ids: materials(5),
            axis: TyAxis3::Z,
            period: 1.0,
            warp: Some(0.0),
            seed: 2.0,
        };

        let ring: Vec<u32> = (0..12)
            .map(|step| {
                let angle = f64::from(step) * TAU / 12.0;
                let z = f64::from(step) * 0.7 - 4.0;
                pick_at(&grain, 2.5 * angle.cos(), 2.5 * angle.sin(), z)
            })
            .collect();
        assert!(ring.iter().all(|pick| *pick == ring[0]), "{ring:?}");

        let across: Vec<u32> = (0..20)
            .map(|step| pick_at(&grain, f64::from(step) + 0.5, 0.0, 0.0))
            .collect();
        assert!(across.iter().any(|pick| *pick != across[0]));
    }

    #[test]
    fn a_gradient_holds_its_ends_past_the_range() {
        let gradient = SdfPattern::Gradient {
            material_ids: materials(4),
            axis: TyAxis3::X,
            from: 0.0,
            to: 2.0,
            warp: None,
            seed: None,
        };

        let picks: Vec<u32> = [-1.0, 0.1, 0.6, 1.1, 1.9, 3.0]
            .iter()
            .map(|x| pick_at(&gradient, *x, 0.0, 0.0))
            .collect();
        assert_eq!(picks, [0, 0, 1, 2, 3, 3]);
    }

    #[test]
    fn a_gradient_from_high_to_low_runs_its_materials_down() {
        let gradient = SdfPattern::Gradient {
            material_ids: materials(4),
            axis: TyAxis3::X,
            from: 2.0,
            to: 0.0,
            warp: None,
            seed: None,
        };

        let picks: Vec<u32> = [-1.0, 0.1, 0.6, 1.1, 1.9, 3.0]
            .iter()
            .map(|x| pick_at(&gradient, *x, 0.0, 0.0))
            .collect();
        assert_eq!(picks, [3, 3, 2, 1, 0, 0]);
    }

    #[test]
    fn noise_spreads_the_cells_across_the_materials() {
        let noise = SdfPattern::Noise {
            material_ids: materials(3),
            scale: 0.5,
            octaves: None,
            seed: 11.0,
        };

        let mut counts = [0; 3];
        for index in 0..3000 {
            let t = f64::from(index);
            counts[pick_at(&noise, t * 0.0731, t * 0.0377, t * 0.0519) as usize] += 1;
        }

        assert!(counts.iter().all(|count| *count > 600), "{counts:?}");
    }

    #[test]
    fn checker_alternates_over_cubes_size_wide() {
        let checker = SdfPattern::Checker {
            material_ids: materials(2),
            size: 0.1,
        };

        assert_eq!(pick_at(&checker, 0.05, 0.05, 0.05), 0);
        assert_eq!(pick_at(&checker, 0.15, 0.05, 0.05), 1);
        assert_eq!(pick_at(&checker, 0.15, 0.15, 0.05), 0);
        assert_eq!(pick_at(&checker, -0.05, 0.05, 0.05), 1);
    }

    #[test]
    fn speckle_picks_per_lattice_cube_at_its_density() {
        let speckle = |density: f64| SdfPattern::Speckle {
            base_id: U32Id::from_u32(9),
            accent_ids: materials(2),
            size: 0.2,
            density,
            seed: 5.0,
        };

        let accents = |density: f64| {
            (0..1000)
                .filter(|index| {
                    let cube = f64::from(*index) * 0.2 + 0.1;
                    let lattice_position = TyVector3F64::new(cube, -cube, 0.7);
                    pick(&speckle(density), TyVector3F64::ZERO, lattice_position) != 9
                })
                .count()
        };

        assert_eq!(accents(0.0), 0);
        assert_eq!(accents(1.0), 1000);
        assert!((200..300).contains(&accents(0.25)));

        let picks: Vec<u32> = (0..2000)
            .map(|index| {
                let lattice_position = TyVector3F64::new(f64::from(index) * 0.1 + 0.05, 0.1, 0.1);
                pick(&speckle(0.5), TyVector3F64::ZERO, lattice_position)
            })
            .collect();
        assert!(picks.chunks(2).all(|pair| pair[0] == pair[1]));
        assert!(picks.iter().any(|pick| *pick != picks[0]));

        let lattice_position = TyVector3F64::new(0.45, 0.55, 0.65);
        assert_eq!(
            pick(
                &speckle(0.5),
                TyVector3F64::new(1.0, 2.0, 3.0),
                lattice_position
            ),
            pick(
                &speckle(0.5),
                TyVector3F64::new(-7.0, 0.0, 9.0),
                lattice_position
            )
        );
    }

    #[test]
    fn bricks_stagger_each_course_by_half_a_brick() {
        let bricks = SdfPattern::Bricks {
            material_ids: materials(64),
            length: 0.2,
            height: 0.05,
            axis: None,
            seed: 2.0,
            border: None,
        };

        // Even courses split at 0.05 and odd courses at 0.15.
        let even = |x| pick_at(&bricks, x, 0.025, 0.1);
        let odd = |x| pick_at(&bricks, x, 0.075, 0.1);
        assert_eq!(even(0.06), even(0.24));
        assert_ne!(even(0.04), even(0.06));
        assert_eq!(odd(0.16), odd(0.34));
        assert_ne!(odd(0.14), odd(0.16));
        assert_ne!(even(0.1), pick_at(&bricks, 0.1, 0.125, 0.1));
    }

    #[test]
    fn a_bricks_border_fills_the_joints_on_every_axis() {
        let bricks = |axis| SdfPattern::Bricks {
            material_ids: materials(4),
            length: 0.2,
            height: 0.05,
            axis,
            seed: 2.0,
            border: Some(SdfBorder {
                material_id: U32Id::from_u32(99),
                width: 0.01,
            }),
        };

        let upright = bricks(None);
        assert_eq!(pick_at(&upright, 0.1, 0.051, 0.1), 99);
        assert_eq!(pick_at(&upright, 0.052, 0.025, 0.1), 99);
        assert_eq!(pick_at(&upright, 0.1, 0.025, 0.052), 99);
        assert_ne!(pick_at(&upright, 0.1, 0.025, 0.1), 99);

        // A face built at a whole or half brick stays brick.
        assert_ne!(pick_at(&upright, 0.0, 0.025, 0.0), 99);
        assert_ne!(pick_at(&upright, 0.1, 0.075, 0.2), 99);

        let paving = bricks(Some(TyAxis3::Z));
        assert_eq!(pick_at(&paving, 0.1, 0.1, 0.051), 99);
        assert_ne!(pick_at(&paving, 0.1, 0.1, 0.025), 99);
    }

    #[test]
    fn a_joint_whole_cells_wide_fills_that_many_cells_in_every_course() {
        for joint_cells in [1, 2] {
            let bricks = SdfPattern::Bricks {
                material_ids: materials(4),
                length: 0.2,
                height: 0.06,
                axis: None,
                seed: 2.0,
                border: Some(SdfBorder {
                    material_id: U32Id::from_u32(99),
                    width: f64::from(joint_cells) * 0.01,
                }),
            };

            // Cells 0.01 wide stack six to a course.
            for cell in 0..300 {
                let y = (f64::from(cell) + 0.5) * 0.01;
                assert_eq!(
                    pick_at(&bricks, 0.1, y, 0.1) == 99,
                    cell % 6 < joint_cells,
                    "{joint_cells} {cell}"
                );
            }
        }
    }

    #[test]
    fn cells_pick_per_feature_point_and_border_the_seams() {
        let cells = |border: bool| SdfPattern::Cells {
            material_ids: materials(8),
            size: 1.0,
            seed: 3.0,
            border: border.then(|| SdfBorder {
                material_id: U32Id::from_u32(99),
                width: 0.1,
            }),
        };

        let line: Vec<u32> = (0..400)
            .map(|step| pick_at(&cells(false), f64::from(step) * 0.01, 0.37, 0.61))
            .collect();
        let changes = line.windows(2).filter(|pair| pair[0] != pair[1]).count();
        assert!(changes >= 2, "{line:?}");

        let bordered: Vec<u32> = (0..400)
            .map(|step| pick_at(&cells(true), f64::from(step) * 0.01, 0.37, 0.61))
            .collect();
        assert!(bordered.contains(&99));
        for (plain, border) in line.iter().zip(&bordered) {
            assert!(border == plain || *border == 99);
        }
    }
}
