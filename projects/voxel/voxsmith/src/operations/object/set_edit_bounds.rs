use crate::{Error, Result};
use branded_id::U32Id;
use ty_math::{TyVector3I32, TyVector3U32};
use voxcore::{BVoxObject, Error as VoxError, VoxExt, VoxMain};

/// Sets each object's build volume to the node-local box from `min` to `max`,
/// the box `node list --show-edit-bounds` prints. The object takes
/// `origin = min` and `bounds = max - min`, and its voxels shift in the grid so
/// no voxel moves in the scene. Errors if:
///
/// 1. `max` is below `min` on some axis
/// 2. an id is not one of the main's objects
/// 3. a live voxel would fall outside the box
pub fn set_edit_bounds<T: VoxExt>(
    main: &mut VoxMain<T>,
    object_ids: &[U32Id<BVoxObject>],
    min: TyVector3I32,
    max: TyVector3I32,
) -> Result<()> {
    let Some(bounds) = box_size(min, max) else {
        return Err(Error::invalid(format!(
            "edit bounds max {max} is below min {min}"
        )));
    };

    for &object_id in object_ids {
        let Some(object) = main.object(object_id) else {
            return Err(VoxError::UnknownObject { object_id }.into());
        };

        let origin = object.origin();

        for voxel_id in object.iter_live() {
            let position = object
                .voxel_position(voxel_id)
                .expect("a live voxel is within the grid");

            if !is_inside(origin, position, min, max) {
                return Err(Error::invalid(format!(
                    "object \"{}\" has a live voxel at grid {position} that falls outside \
                     edit bounds {min} to {max}",
                    object.name()
                )));
            }
        }

        main.remap_object_voxels(object_id, bounds, |position| {
            let component = |position: u32, origin: i32, min: i32| {
                i32::try_from(i64::from(position) + i64::from(origin) - i64::from(min))
                    .expect("a voxel inside the box lands inside the new grid")
            };

            TyVector3I32::new(
                component(position.x, origin.x, min.x),
                component(position.y, origin.y, min.y),
                component(position.z, origin.z, min.z),
            )
        })?;

        main.set_object_origin(object_id, min)?;
    }

    Ok(())
}

fn box_size(min: TyVector3I32, max: TyVector3I32) -> Option<TyVector3U32> {
    let component = |min: i32, max: i32| u32::try_from(i64::from(max) - i64::from(min)).ok();

    Some(TyVector3U32::new(
        component(min.x, max.x)?,
        component(min.y, max.y)?,
        component(min.z, max.z)?,
    ))
}

fn is_inside(
    origin: TyVector3I32,
    position: TyVector3U32,
    min: TyVector3I32,
    max: TyVector3I32,
) -> bool {
    let component = |origin: i32, position: u32, min: i32, max: i32| {
        let local = i64::from(origin) + i64::from(position);

        i64::from(min) <= local && local < i64::from(max)
    };

    component(origin.x, position.x, min.x, max.x)
        && component(origin.y, position.y, min.y, max.y)
        && component(origin.z, position.z, min.z, max.z)
}

#[cfg(test)]
mod tests {
    use crate::{operations::object::set_edit_bounds, test_utilities::HookRecorder};
    use branded_id::U32Id;
    use ty_math::{TyVector3I32, TyVector3U32};
    use voxcore::{VoxMain, VoxObject};

    /// A `4 x 8 x 4` object at the zero origin with live voxels at its two
    /// far corners, `(0, 0, 0)` and `(3, 7, 3)`.
    fn crate_scene() -> VoxMain<HookRecorder> {
        let mut main: VoxMain = VoxMain::default();

        let mut object = VoxObject::new("crate".to_owned(), TyVector3U32::new(4, 8, 4)).unwrap();

        for position in [TyVector3U32::ZERO, TyVector3U32::new(3, 7, 3)] {
            let voxel_id = object.voxel_id(position).unwrap();

            object.retain_voxel(voxel_id, &[]).unwrap();
        }

        main.retain_object(object).unwrap();

        main.put_ext(HookRecorder::default())
    }

    fn scene_positions(main: &VoxMain<HookRecorder>) -> Vec<TyVector3I32> {
        let (_, object) = main.iter_objects().next().unwrap();

        let mut positions: Vec<TyVector3I32> = object
            .iter_live()
            .map(|voxel_id| object.origin() + object.voxel_position(voxel_id).unwrap().as_ivec3())
            .collect();

        positions.sort_by_key(|position| (position.x, position.y, position.z));

        positions
    }

    #[test]
    fn grows_the_box_without_moving_a_voxel() {
        let mut main = crate_scene();

        let before = scene_positions(&main);

        // The README example: 2 voxels of margin on -x.
        set_edit_bounds(
            &mut main,
            &[U32Id::from_u32(0)],
            TyVector3I32::new(-2, 0, 0),
            TyVector3I32::new(4, 8, 4),
        )
        .unwrap();

        let (_, object) = main.iter_objects().next().unwrap();

        assert_eq!(object.bounds(), TyVector3U32::new(6, 8, 4));
        assert_eq!(object.origin(), TyVector3I32::new(-2, 0, 0));
        assert_eq!(scene_positions(&main), before);
        assert_eq!(
            HookRecorder::events(&main),
            ["object 0 voxels remapped", "object 0 origin set"]
        );
        main.validate().unwrap();
    }

    #[test]
    fn a_voxel_outside_the_box_is_an_error_that_changes_nothing() {
        let mut main = crate_scene();

        assert!(
            set_edit_bounds(
                &mut main,
                &[U32Id::from_u32(0)],
                TyVector3I32::new(0, 0, 0),
                TyVector3I32::new(3, 8, 4),
            )
            .is_err()
        );

        assert_eq!(
            main.object(U32Id::from_u32(0)).unwrap().bounds(),
            TyVector3U32::new(4, 8, 4)
        );
        assert!(HookRecorder::events(&main).is_empty());
    }

    #[test]
    fn max_below_min_is_an_error() {
        let mut main = crate_scene();

        assert!(
            set_edit_bounds(
                &mut main,
                &[U32Id::from_u32(0)],
                TyVector3I32::new(0, 0, 0),
                TyVector3I32::new(4, -1, 4),
            )
            .is_err()
        );
    }
}
