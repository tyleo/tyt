use crate::{Error, Result};
use branded_id::U32Id;
use ty_math::{TyVector3I32, TyVector3U32};
use voxcore::{BVoxObject, Error as VoxError, VoxExt, VoxMain};

/// Moves each object's voxels by `offset` within its grid. `origin` and
/// `bounds` stay. Errors when an id is not one of the main's objects or a live
/// voxel would leave the grid.
pub fn translate_object_voxels<T: VoxExt>(
    main: &mut VoxMain<T>,
    object_ids: &[U32Id<BVoxObject>],
    offset: TyVector3I32,
) -> Result<()> {
    for &object_id in object_ids {
        let Some(object) = main.object(object_id) else {
            return Err(VoxError::UnknownObject { object_id }.into());
        };

        let bounds = object.bounds();

        for voxel_id in object.iter_live() {
            let position = object
                .voxel_position(voxel_id)
                .expect("a live voxel is within the grid");

            if !is_inside(position, offset, bounds) {
                return Err(Error::invalid(format!(
                    "object \"{}\" has a live voxel at grid {position} that offset {offset} \
                     moves outside bounds {bounds}",
                    object.name()
                )));
            }
        }

        main.remap_object_voxels(object_id, bounds, |position| {
            let component = |position: u32, offset: i32| {
                i32::try_from(i64::from(position) + i64::from(offset))
                    .expect("a voxel inside the grid has an i32 position")
            };

            TyVector3I32::new(
                component(position.x, offset.x),
                component(position.y, offset.y),
                component(position.z, offset.z),
            )
        })?;
    }

    Ok(())
}

fn is_inside(position: TyVector3U32, offset: TyVector3I32, bounds: TyVector3U32) -> bool {
    let component = |position: u32, offset: i32, bound: u32| {
        let moved = i64::from(position) + i64::from(offset);

        0 <= moved && moved < i64::from(bound)
    };

    component(position.x, offset.x, bounds.x)
        && component(position.y, offset.y, bounds.y)
        && component(position.z, offset.z, bounds.z)
}

#[cfg(test)]
mod tests {
    use crate::{operations::object::translate_object_voxels, test_utilities::HookRecorder};
    use branded_id::U32Id;
    use ty_math::{TyVector3I32, TyVector3U32};
    use voxcore::{VoxMain, VoxObject};

    /// A `4 x 4 x 4` object with a live voxel at `(1, 1, 1)`.
    fn scene() -> VoxMain<HookRecorder> {
        let mut main: VoxMain = VoxMain::default();

        let mut object = VoxObject::new("a".to_owned(), TyVector3U32::splat(4)).unwrap();

        let voxel_id = object.voxel_id(TyVector3U32::splat(1)).unwrap();
        object.retain_voxel(voxel_id, &[]).unwrap();

        main.retain_object(object).unwrap();

        main.put_ext(HookRecorder::default())
    }

    fn position(main: &VoxMain<HookRecorder>) -> TyVector3U32 {
        let (_, object) = main.iter_objects().next().unwrap();

        let voxel_id = object.iter_live().next().unwrap();

        object.voxel_position(voxel_id).unwrap()
    }

    #[test]
    fn moves_voxels_within_the_grid() {
        let mut main = scene();

        translate_object_voxels(
            &mut main,
            &[U32Id::from_u32(0)],
            TyVector3I32::new(2, -1, 0),
        )
        .unwrap();

        let (_, object) = main.iter_objects().next().unwrap();

        assert_eq!(object.bounds(), TyVector3U32::splat(4));
        assert_eq!(object.origin(), TyVector3I32::ZERO);
        assert_eq!(position(&main), TyVector3U32::new(3, 0, 1));
        assert_eq!(HookRecorder::events(&main), ["object 0 voxels remapped"]);
        main.validate().unwrap();
    }

    #[test]
    fn a_voxel_leaving_the_grid_is_an_error_that_changes_nothing() {
        for offset in [TyVector3I32::new(3, 0, 0), TyVector3I32::new(0, 0, -2)] {
            let mut main = scene();

            assert!(translate_object_voxels(&mut main, &[U32Id::from_u32(0)], offset).is_err());
            assert_eq!(position(&main), TyVector3U32::splat(1));
            assert!(HookRecorder::events(&main).is_empty());
        }
    }
}
