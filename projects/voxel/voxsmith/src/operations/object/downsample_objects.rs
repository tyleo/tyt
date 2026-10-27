use crate::{
    Result,
    operations::object::{KeepRule, ResampleFactor},
};
use branded_id::U32Id;
use std::{cmp::Reverse, collections::HashMap};
use ty_math::{TyVector3I32, TyVector3U32};
use voxcore::{BVoxMaterial, BVoxObject, BVoxVoxel, Error as VoxError, VoxExt, VoxMain, VoxObject};

/// Merges each object's voxels into `factor` x `factor` x `factor` blocks of
/// the node's lattice, coarsening its grid by `factor`. The new grid covers
/// the old box: `origin` rounds down and the far corner rounds up to the
/// coarse lattice. A node scale of `factor` then keeps the object's place in
/// the scene. `keep` decides whether a block is live by how many of its cells
/// are, with a cell outside the old grid counting as empty. A live block takes
/// the samples most of its live cells share. A tie goes to the earliest cell
/// in raster order. Errors when an id is not one of the main's objects.
pub fn downsample_objects<T: VoxExt>(
    main: &mut VoxMain<T>,
    object_ids: &[U32Id<BVoxObject>],
    factor: ResampleFactor,
    keep: KeepRule,
) -> Result<()> {
    let factor = factor.get();

    for &object_id in object_ids {
        let Some(object) = main.object(object_id) else {
            return Err(VoxError::UnknownObject { object_id }.into());
        };

        let (new_origin, new_bounds) = coarse_box(object.origin(), object.bounds(), factor);

        let sources = block_sources(object, new_origin, new_bounds, factor, keep);

        let plane = new_bounds.y * new_bounds.z;

        main.resample_object_voxels(object_id, new_bounds, |position| {
            sources[(position.x * plane + position.y * new_bounds.z + position.z) as usize]
        })?;

        main.set_object_origin(object_id, new_origin)?;
    }

    Ok(())
}

/// The coarse box covering the node-local box at `origin` of `bounds`, as
/// its origin and bounds. An axis of zero bounds stays empty.
fn coarse_box(
    origin: TyVector3I32,
    bounds: TyVector3U32,
    factor: u32,
) -> (TyVector3I32, TyVector3U32) {
    let factor = i64::from(factor);

    let axis = |origin: i32, bounds: u32| {
        let (origin, bounds) = (i64::from(origin), i64::from(bounds));

        let min = origin.div_euclid(factor);

        let max = (origin + bounds).div_euclid(factor)
            + i64::from((origin + bounds).rem_euclid(factor) != 0);

        let size = if bounds == 0 { 0 } else { max - min };

        (
            i32::try_from(min).expect("a divided origin stays within i32"),
            u32::try_from(size).expect("a divided size stays within u32"),
        )
    };

    let (x, y, z) = (
        axis(origin.x, bounds.x),
        axis(origin.y, bounds.y),
        axis(origin.z, bounds.z),
    );

    (
        TyVector3I32::new(x.0, y.0, z.0),
        TyVector3U32::new(x.1, y.1, z.1),
    )
}

/// Per cell of the coarse grid in raster order, the old cell it draws from,
/// or `None` when `keep` drops the block.
fn block_sources(
    object: &VoxObject,
    new_origin: TyVector3I32,
    new_bounds: TyVector3U32,
    factor: u32,
    keep: KeepRule,
) -> Vec<Option<TyVector3U32>> {
    let layer_ids: Vec<_> = object.iter_layers().map(|(layer_id, _)| layer_id).collect();

    let cells = usize::try_from(u64::from(factor).pow(3)).expect("a block's cell count fits usize");

    let mut sources = Vec::new();

    for x in 0..new_bounds.x {
        for y in 0..new_bounds.y {
            for z in 0..new_bounds.z {
                let block = TyVector3U32::new(x, y, z);

                let mut tally: HashMap<Vec<U32Id<BVoxMaterial>>, (usize, U32Id<BVoxVoxel>)> =
                    HashMap::new();

                let mut live = 0;

                for voxel_id in block_cells(object, new_origin, block, factor) {
                    live += 1;

                    let samples: Vec<_> = layer_ids
                        .iter()
                        .map(|&layer_id| {
                            object
                                .voxel_material(voxel_id, layer_id)
                                .expect("a live voxel samples every layer")
                        })
                        .collect();

                    tally.entry(samples).or_insert((0, voxel_id)).0 += 1;
                }

                let kept = match keep {
                    KeepRule::All => live == cells,
                    KeepRule::Any => live > 0,
                    KeepRule::Majority => live * 2 >= cells,
                };

                let winner = tally
                    .into_values()
                    .max_by_key(|&(count, first_id)| (count, Reverse(first_id)))
                    .filter(|_| kept)
                    .map(|(_, first_id)| {
                        object
                            .voxel_position(first_id)
                            .expect("a live voxel is within the grid")
                    });

                sources.push(winner);
            }
        }
    }

    sources
}

/// The live old voxels within coarse cell `block`, in raster order.
fn block_cells(
    object: &VoxObject,
    new_origin: TyVector3I32,
    block: TyVector3U32,
    factor: u32,
) -> impl Iterator<Item = U32Id<BVoxVoxel>> + '_ {
    let bounds = object.bounds().to_array();

    // The block's min corner on the old grid, which may sit outside it.
    let corner = |new_origin: i32, block: u32, origin: i32| {
        (i64::from(new_origin) + i64::from(block)) * i64::from(factor) - i64::from(origin)
    };

    let origin = object.origin();

    let corner = [
        corner(new_origin.x, block.x, origin.x),
        corner(new_origin.y, block.y, origin.y),
        corner(new_origin.z, block.z, origin.z),
    ];

    let factor = i64::from(factor);

    (0..factor).flat_map(move |dx| {
        (0..factor).flat_map(move |dy| {
            (0..factor).filter_map(move |dz| {
                let cell = [corner[0] + dx, corner[1] + dy, corner[2] + dz];

                let inside = cell
                    .iter()
                    .zip(bounds)
                    .all(|(&cell, bound)| 0 <= cell && cell < i64::from(bound));

                let position = inside.then(|| {
                    let component =
                        |cell: i64| u32::try_from(cell).expect("a cell inside the box fits u32");

                    TyVector3U32::new(component(cell[0]), component(cell[1]), component(cell[2]))
                })?;

                let voxel_id = object
                    .voxel_id(position)
                    .expect("a cell inside the box is within the grid");

                object.is_live(voxel_id).then_some(voxel_id)
            })
        })
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        operations::object::{KeepRule, ResampleFactor, downsample_objects, upsample_objects},
        test_utilities::{HookRecorder, live_cells, two_material_scene},
    };
    use branded_id::U32Id;
    use ty_math::{TyVector3I32, TyVector3U32};
    use voxcore::VoxMain;

    const RULES: [KeepRule; 3] = [KeepRule::All, KeepRule::Any, KeepRule::Majority];

    fn factor(factor: u32) -> ResampleFactor {
        ResampleFactor::new(factor).unwrap()
    }

    fn downsample(main: &mut VoxMain<HookRecorder>, factor_value: u32, keep: KeepRule) {
        downsample_objects(main, &[U32Id::from_u32(0)], factor(factor_value), keep).unwrap();
    }

    /// Every cell of a `bounds` grid, live with material `material`.
    fn solid(bounds: TyVector3U32, material: u32) -> Vec<(TyVector3U32, u32)> {
        let mut cells = Vec::new();
        for x in 0..bounds.x {
            for y in 0..bounds.y {
                for z in 0..bounds.z {
                    cells.push((TyVector3U32::new(x, y, z), material));
                }
            }
        }
        cells
    }

    #[test]
    fn merges_blocks_onto_the_coarse_lattice() {
        for keep in RULES {
            // A solid `4 x 4 x 2` object at origin `(0, -4, 2)`, which every
            // factor-2 block covers whole.
            let mut main = two_material_scene(
                TyVector3U32::new(4, 4, 2),
                TyVector3I32::new(0, -4, 2),
                &solid(TyVector3U32::new(4, 4, 2), 0),
            );

            downsample(&mut main, 2, keep);

            let (_, object) = main.iter_objects().next().unwrap();

            assert_eq!(object.bounds(), TyVector3U32::new(2, 2, 1));
            assert_eq!(object.origin(), TyVector3I32::new(0, -2, 1));
            assert_eq!(live_cells(&main), solid(TyVector3U32::new(2, 2, 1), 0));
            assert_eq!(
                HookRecorder::events(&main),
                ["object 0 voxels resampled", "object 0 origin set"]
            );
            main.validate().unwrap();
        }
    }

    #[test]
    fn the_grid_grows_to_cover_an_origin_the_factor_does_not_divide() {
        // A solid `2 x 1 x 1` object at origin `(-1, 0, 0)` straddles two
        // factor-2 blocks, each with one live cell of its eight.
        let scene = || {
            two_material_scene(
                TyVector3U32::new(2, 1, 1),
                TyVector3I32::new(-1, 0, 0),
                &solid(TyVector3U32::new(2, 1, 1), 1),
            )
        };

        for (keep, want) in [
            (KeepRule::All, vec![]),
            (KeepRule::Any, solid(TyVector3U32::new(2, 1, 1), 1)),
            (KeepRule::Majority, vec![]),
        ] {
            let mut main = scene();

            downsample(&mut main, 2, keep);

            let (_, object) = main.iter_objects().next().unwrap();

            assert_eq!(object.bounds(), TyVector3U32::new(2, 1, 1));
            assert_eq!(object.origin(), TyVector3I32::new(-1, 0, 0));
            assert_eq!(live_cells(&main), want);
        }
    }

    #[test]
    fn a_block_takes_the_samples_most_of_its_live_cells_share() {
        // Of a `2 x 2 x 2` block, five cells sample material 1 and three
        // sample 0.
        let mut cells = solid(TyVector3U32::splat(2), 1);
        for (_, material) in &mut cells[..3] {
            *material = 0;
        }

        let mut main = two_material_scene(TyVector3U32::splat(2), TyVector3I32::ZERO, &cells);

        downsample(&mut main, 2, KeepRule::All);

        assert_eq!(live_cells(&main), [(TyVector3U32::ZERO, 1)]);
    }

    #[test]
    fn a_tie_goes_to_the_earliest_cell_in_raster_order() {
        // Four cells sample material 1, then four sample 0.
        let mut cells = solid(TyVector3U32::splat(2), 0);
        for (_, material) in &mut cells[..4] {
            *material = 1;
        }

        let mut main = two_material_scene(TyVector3U32::splat(2), TyVector3I32::ZERO, &cells);

        downsample(&mut main, 2, KeepRule::Majority);

        assert_eq!(live_cells(&main), [(TyVector3U32::ZERO, 1)]);
    }

    #[test]
    fn majority_keeps_a_block_at_least_half_live() {
        // A one-voxel-thick wall fills four of each block's eight cells.
        let mut main = two_material_scene(
            TyVector3U32::new(4, 4, 1),
            TyVector3I32::ZERO,
            &solid(TyVector3U32::new(4, 4, 1), 0),
        );

        downsample(&mut main, 2, KeepRule::Majority);

        let (_, object) = main.iter_objects().next().unwrap();

        assert_eq!(object.bounds(), TyVector3U32::new(2, 2, 1));
        assert_eq!(live_cells(&main), solid(TyVector3U32::new(2, 2, 1), 0));

        // A lone voxel fills one of eight.
        let mut main = two_material_scene(
            TyVector3U32::splat(2),
            TyVector3I32::ZERO,
            &[(TyVector3U32::ZERO, 0)],
        );

        downsample(&mut main, 2, KeepRule::Majority);

        assert!(live_cells(&main).is_empty());
    }

    #[test]
    fn an_upsample_downsamples_back_to_the_original() {
        let cells = [
            (TyVector3U32::new(0, 2, 1), 0),
            (TyVector3U32::new(2, 0, 0), 1),
            (TyVector3U32::new(2, 2, 2), 1),
        ];

        for keep in RULES {
            let mut main =
                two_material_scene(TyVector3U32::splat(3), TyVector3I32::new(-2, 5, 7), &cells);

            upsample_objects(&mut main, &[U32Id::from_u32(0)], factor(3)).unwrap();

            downsample(&mut main, 3, keep);

            let (_, object) = main.iter_objects().next().unwrap();

            assert_eq!(object.bounds(), TyVector3U32::splat(3));
            assert_eq!(object.origin(), TyVector3I32::new(-2, 5, 7));
            assert_eq!(live_cells(&main), cells);
            main.validate().unwrap();
        }
    }

    #[test]
    fn an_empty_axis_stays_empty() {
        let mut main =
            two_material_scene(TyVector3U32::new(4, 0, 4), TyVector3I32::new(3, 3, 3), &[]);

        downsample(&mut main, 2, KeepRule::Any);

        let (_, object) = main.iter_objects().next().unwrap();

        assert_eq!(object.bounds(), TyVector3U32::new(3, 0, 3));
        assert_eq!(object.origin(), TyVector3I32::new(1, 1, 1));
        main.validate().unwrap();
    }

    #[test]
    fn an_unknown_object_is_an_error() {
        let mut main: VoxMain = VoxMain::default();

        assert!(
            downsample_objects(&mut main, &[U32Id::from_u32(3)], factor(2), KeepRule::Any).is_err()
        );
    }
}
