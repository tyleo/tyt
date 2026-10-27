use crate::Result;
use branded_id::U32Id;
use ty_math::{TyAxis3, TyVector3I32};
use voxcore::{BVoxObject, Error as VoxError, VoxExt, VoxMain};

/// Mirrors each object's voxels in place along `axis`, sending `p.a` to
/// `bounds.a - 1 - p.a`. `origin` and `bounds` stay. Errors when an id is not
/// one of the main's objects.
pub fn flip_object_voxels<T: VoxExt>(
    main: &mut VoxMain<T>,
    object_ids: &[U32Id<BVoxObject>],
    axis: TyAxis3,
) -> Result<()> {
    let index = axis.index();

    for &object_id in object_ids {
        let Some(object) = main.object(object_id) else {
            return Err(VoxError::UnknownObject { object_id }.into());
        };

        let bounds = object.bounds();

        main.remap_object_voxels(object_id, bounds, |position| {
            let mut flipped = position;
            flipped[index] = bounds[index] - 1 - position[index];

            TyVector3I32::try_from(flipped).expect("a grid position fits i32")
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{operations::object::flip_object_voxels, test_utilities::HookRecorder};
    use branded_id::U32Id;
    use ty_math::{TyAxis3, TyVector3U32};
    use voxcore::{VoxMain, VoxObject};

    /// A `2 x 3 x 4` object with live voxels at `(0, 0, 0)` and `(1, 2, 1)`.
    fn scene() -> VoxMain<HookRecorder> {
        let mut main: VoxMain = VoxMain::default();

        let mut object = VoxObject::new("a".to_owned(), TyVector3U32::new(2, 3, 4)).unwrap();

        for position in [TyVector3U32::ZERO, TyVector3U32::new(1, 2, 1)] {
            let voxel_id = object.voxel_id(position).unwrap();

            object.retain_voxel(voxel_id, &[]).unwrap();
        }

        main.retain_object(object).unwrap();

        main.put_ext(HookRecorder::default())
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

    #[test]
    fn mirrors_along_the_axis_in_place() {
        let mut main = scene();

        flip_object_voxels(&mut main, &[U32Id::from_u32(0)], TyAxis3::Z).unwrap();

        let (_, object) = main.iter_objects().next().unwrap();

        assert_eq!(object.bounds(), TyVector3U32::new(2, 3, 4));
        assert_eq!(
            positions(&main),
            [TyVector3U32::new(0, 0, 3), TyVector3U32::new(1, 2, 2)]
        );
        assert_eq!(HookRecorder::events(&main), ["object 0 voxels remapped"]);
        main.validate().unwrap();
    }

    #[test]
    fn two_flips_leave_the_object_unchanged() {
        for axis in [TyAxis3::X, TyAxis3::Y, TyAxis3::Z] {
            let mut main = scene();

            let before = positions(&main);

            for _ in 0..2 {
                flip_object_voxels(&mut main, &[U32Id::from_u32(0)], axis).unwrap();
            }

            assert_eq!(positions(&main), before);
        }
    }
}
