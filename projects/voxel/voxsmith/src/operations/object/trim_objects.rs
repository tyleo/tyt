use crate::{Error, Result};
use branded_id::U32Id;
use ty_math::{TyVector3I32, TyVector3U32};
use voxcore::{BVoxObject, Error as VoxError, VoxExt, VoxMain};

/// Shrinks each object in `object_ids` to its live extent and moves its
/// `origin` by the extent's min corner, so no voxel moves in the scene. An
/// object with no live voxels trims to zero bounds and keeps its origin.
/// Errors when an id is not one of the main's objects or the moved origin
/// leaves the `i32` range.
pub fn trim_objects<T: VoxExt>(
    main: &mut VoxMain<T>,
    object_ids: &[U32Id<BVoxObject>],
) -> Result<()> {
    for &object_id in object_ids {
        let Some(object) = main.object(object_id) else {
            return Err(VoxError::UnknownObject { object_id }.into());
        };

        let Some((min, size)) = object.live_extent() else {
            main.remap_object_voxels(object_id, TyVector3U32::ZERO, |_| {
                unreachable!("an object with no live voxels remaps none")
            })?;

            continue;
        };

        let origin = object.origin();

        let Some(new_origin) = offset_origin(origin, min) else {
            return Err(Error::invalid(format!(
                "object \"{}\" cannot trim: moving origin {origin} by {min} leaves the i32 range",
                object.name()
            )));
        };

        main.remap_object_voxels(object_id, size, |position| {
            TyVector3I32::try_from(position - min)
                .expect("a live voxel sits at or past the extent's min corner")
        })?;

        main.set_object_origin(object_id, new_origin)?;
    }

    Ok(())
}

fn offset_origin(origin: TyVector3I32, offset: TyVector3U32) -> Option<TyVector3I32> {
    let component =
        |origin: i32, offset: u32| i32::try_from(i64::from(origin) + i64::from(offset)).ok();

    Some(TyVector3I32::new(
        component(origin.x, offset.x)?,
        component(origin.y, offset.y)?,
        component(origin.z, offset.z)?,
    ))
}

#[cfg(test)]
mod tests {
    use crate::{operations::object::trim_objects, test_utilities::HookRecorder};
    use branded_id::U32Id;
    use ty_math::{TyVector3I32, TyVector3U32};
    use voxcore::{VoxMain, VoxObject};

    /// A `4 x 4 x 4` object at origin `(1, 2, 3)` with live voxels at
    /// `(1, 1, 1)` and `(2, 3, 1)`.
    fn scene() -> VoxMain<HookRecorder> {
        let mut main: VoxMain = VoxMain::default();

        let mut object = VoxObject::new("a".to_owned(), TyVector3U32::splat(4)).unwrap();
        object.set_origin(TyVector3I32::new(1, 2, 3));

        for position in [TyVector3U32::new(1, 1, 1), TyVector3U32::new(2, 3, 1)] {
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
    fn trims_to_the_live_extent_without_moving_a_voxel() {
        let mut main = scene();

        let before = scene_positions(&main);

        trim_objects(&mut main, &[U32Id::from_u32(0)]).unwrap();

        let (_, object) = main.iter_objects().next().unwrap();

        assert_eq!(object.bounds(), TyVector3U32::new(2, 3, 1));
        assert_eq!(object.origin(), TyVector3I32::new(2, 3, 4));
        assert_eq!(scene_positions(&main), before);
        assert_eq!(
            HookRecorder::events(&main),
            ["object 0 voxels remapped", "object 0 origin set"]
        );
        main.validate().unwrap();
    }

    #[test]
    fn an_empty_object_trims_to_zero_bounds() {
        let mut main: VoxMain = VoxMain::default();

        let object = VoxObject::new("a".to_owned(), TyVector3U32::splat(4)).unwrap();

        let object_id = main.retain_object(object).unwrap();

        trim_objects(&mut main, &[object_id]).unwrap();

        assert_eq!(main.object(object_id).unwrap().bounds(), TyVector3U32::ZERO);
        main.validate().unwrap();
    }

    #[test]
    fn an_origin_leaving_the_i32_range_is_an_error() {
        let mut main = scene();

        let object_id = U32Id::from_u32(0);

        main.set_object_origin(object_id, TyVector3I32::new(i32::MAX, 0, 0))
            .unwrap();

        assert!(trim_objects(&mut main, &[object_id]).is_err());
        assert_eq!(
            main.object(object_id).unwrap().bounds(),
            TyVector3U32::splat(4)
        );
    }
}
