use crate::{Error, Result, operations::object::QuarterTurns};
use branded_id::U32Id;
use ty_math::{TyAxis3, TyVector3I32};
use voxcore::{BVoxObject, Error as VoxError, VoxExt, VoxMain};

/// Turns each object's voxels about the grid's center by `turns` quarter turns
/// about `axis`, following the right-hand rule. One turn about `y` sends
/// `(x, y, z)` to `(z, y, bounds.x - 1 - x)`. `origin` and `bounds` stay. Errors
/// when an id is not one of the main's objects or when one or three turns meet
/// unequal turned dimensions. Only a node rotation can turn such an object.
pub fn rotate_object_voxels<T: VoxExt>(
    main: &mut VoxMain<T>,
    object_ids: &[U32Id<BVoxObject>],
    axis: TyAxis3,
    turns: QuarterTurns,
) -> Result<()> {
    // A positive turn carries `u` toward `v`.
    let (u, v) = match axis {
        TyAxis3::X => (TyAxis3::Y, TyAxis3::Z),
        TyAxis3::Y => (TyAxis3::Z, TyAxis3::X),
        TyAxis3::Z => (TyAxis3::X, TyAxis3::Y),
    };

    for &object_id in object_ids {
        let Some(object) = main.object(object_id) else {
            return Err(VoxError::UnknownObject { object_id }.into());
        };

        let bounds = object.bounds();

        let (u_size, v_size) = (bounds[u.index()], bounds[v.index()]);

        if turns != QuarterTurns::Two && u_size != v_size {
            return Err(Error::invalid(format!(
                "object \"{}\" is {} x {} x {}, and a quarter turn about {axis} needs equal {u} \
                 and {v}; turn its node with node set rotation instead",
                object.name(),
                bounds.x,
                bounds.y,
                bounds.z,
            )));
        }

        main.remap_object_voxels(object_id, bounds, |position| {
            let (position_u, position_v) = (position[u.index()], position[v.index()]);

            let (turned_u, turned_v) = match turns {
                QuarterTurns::One => (v_size - 1 - position_v, position_u),
                QuarterTurns::Two => (u_size - 1 - position_u, v_size - 1 - position_v),
                QuarterTurns::Three => (position_v, u_size - 1 - position_u),
            };

            let mut turned = position;
            turned[u.index()] = turned_u;
            turned[v.index()] = turned_v;

            TyVector3I32::try_from(turned).expect("a grid position fits i32")
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        operations::object::{QuarterTurns, rotate_object_voxels},
        test_utilities::HookRecorder,
    };
    use branded_id::U32Id;
    use ty_math::{TyAxis3, TyVector3U32};
    use voxcore::{VoxMain, VoxObject};

    const AXES: [TyAxis3; 3] = [TyAxis3::X, TyAxis3::Y, TyAxis3::Z];

    /// An object of `bounds` with live voxels at `positions`.
    fn scene(bounds: TyVector3U32, positions: &[TyVector3U32]) -> VoxMain<HookRecorder> {
        let mut main: VoxMain = VoxMain::default();

        let mut object = VoxObject::new("crate".to_owned(), bounds).unwrap();

        for &position in positions {
            let voxel_id = object.voxel_id(position).unwrap();

            object.retain_voxel(voxel_id, &[]).unwrap();
        }

        main.retain_object(object).unwrap();

        main.put_ext(HookRecorder::default())
    }

    /// A `3 x 3 x 3` cube with three live voxels, no two related by a turn.
    fn cube() -> VoxMain<HookRecorder> {
        scene(
            TyVector3U32::splat(3),
            &[
                TyVector3U32::new(0, 0, 0),
                TyVector3U32::new(2, 1, 0),
                TyVector3U32::new(1, 2, 2),
            ],
        )
    }

    fn positions(main: &VoxMain<HookRecorder>) -> Vec<TyVector3U32> {
        let (_, object) = main.iter_objects().next().unwrap();

        let mut positions: Vec<TyVector3U32> = object
            .iter_live()
            .map(|voxel_id| object.voxel_position(voxel_id).unwrap())
            .collect();

        positions.sort_by_key(|position| (position.x, position.y, position.z));

        positions
    }

    fn rotate(main: &mut VoxMain<HookRecorder>, axis: TyAxis3, turns: QuarterTurns) {
        rotate_object_voxels(main, &[U32Id::from_u32(0)], axis, turns).unwrap();
    }

    #[test]
    fn one_turn_about_y_follows_the_right_hand_rule() {
        let mut main = scene(TyVector3U32::new(4, 8, 4), &[TyVector3U32::new(1, 5, 0)]);

        rotate(&mut main, TyAxis3::Y, QuarterTurns::One);

        // `(x, y, z)` goes to `(z, y, bounds.x - 1 - x)`.
        assert_eq!(positions(&main), [TyVector3U32::new(0, 5, 2)]);
        assert_eq!(HookRecorder::events(&main), ["object 0 voxels remapped"]);
        main.validate().unwrap();
    }

    #[test]
    fn four_turns_leave_the_object_unchanged() {
        for axis in AXES {
            let mut main = cube();

            let before = positions(&main);

            for _ in 0..4 {
                rotate(&mut main, axis, QuarterTurns::One);
            }

            assert_eq!(positions(&main), before);
        }
    }

    #[test]
    fn two_and_three_turns_match_repeated_single_turns() {
        for axis in AXES {
            for (turns, count) in [(QuarterTurns::Two, 2), (QuarterTurns::Three, 3)] {
                let mut once = cube();
                rotate(&mut once, axis, turns);

                let mut repeated = cube();
                for _ in 0..count {
                    rotate(&mut repeated, axis, QuarterTurns::One);
                }

                assert_eq!(positions(&once), positions(&repeated));
            }
        }
    }

    #[test]
    fn a_quarter_turn_needs_equal_turned_dimensions() {
        // The README door: 4 x 8 x 1 has unequal x and z.
        let door = TyVector3U32::new(4, 8, 1);

        let mut main = scene(door, &[TyVector3U32::new(3, 7, 0)]);

        assert!(
            rotate_object_voxels(
                &mut main,
                &[U32Id::from_u32(0)],
                TyAxis3::Y,
                QuarterTurns::One
            )
            .is_err()
        );
        assert!(HookRecorder::events(&main).is_empty());

        rotate(&mut main, TyAxis3::Y, QuarterTurns::Two);

        assert_eq!(positions(&main), [TyVector3U32::new(0, 7, 0)]);
    }
}
