use crate::{Error, Result, operations::object::ResampleFactor};
use branded_id::U32Id;
use ty_math::{TyVector3I32, TyVector3U32};
use voxcore::{BVoxObject, Error as VoxError, VoxExt, VoxMain, VoxObject};

/// Splits each object's voxels into `factor` x `factor` x `factor` blocks of
/// the same samples, refining its grid by `factor`. `bounds` and `origin`
/// both scale by it. A node scale of `1 / factor` then keeps the object's
/// place in the scene. Errors when an id is not one of the main's objects, the
/// scaled grid leaves the u32 range or the cell cap, or the scaled origin
/// leaves the i32 range.
pub fn upsample_objects<T: VoxExt>(
    main: &mut VoxMain<T>,
    object_ids: &[U32Id<BVoxObject>],
    factor: ResampleFactor,
) -> Result<()> {
    let factor = factor.get();

    for &object_id in object_ids {
        let Some(object) = main.object(object_id) else {
            return Err(VoxError::UnknownObject { object_id }.into());
        };

        let bounds = object.bounds();

        let origin = object.origin();

        let Some(new_bounds) = scale_bounds(bounds, factor) else {
            return Err(Error::invalid(format!(
                "object \"{}\" cannot upsample: scaling bounds {bounds} by {factor} leaves the \
                 u32 range",
                object.name()
            )));
        };

        let cells = u64::from(new_bounds.x) * u64::from(new_bounds.y) * u64::from(new_bounds.z);

        if cells > VoxObject::MAX_GRID_CELLS {
            return Err(Error::invalid(format!(
                "object \"{}\" cannot upsample: a {cells}-cell grid at factor {factor} exceeds \
                 the {}-cell cap",
                object.name(),
                VoxObject::MAX_GRID_CELLS
            )));
        }

        let Some(new_origin) = scale_origin(origin, factor) else {
            return Err(Error::invalid(format!(
                "object \"{}\" cannot upsample: scaling origin {origin} by {factor} leaves the \
                 i32 range",
                object.name()
            )));
        };

        main.resample_object_voxels(object_id, new_bounds, |position| Some(position / factor))?;

        main.set_object_origin(object_id, new_origin)?;
    }

    Ok(())
}

fn scale_bounds(bounds: TyVector3U32, factor: u32) -> Option<TyVector3U32> {
    Some(TyVector3U32::new(
        bounds.x.checked_mul(factor)?,
        bounds.y.checked_mul(factor)?,
        bounds.z.checked_mul(factor)?,
    ))
}

fn scale_origin(origin: TyVector3I32, factor: u32) -> Option<TyVector3I32> {
    let factor = i32::try_from(factor).ok()?;

    Some(TyVector3I32::new(
        origin.x.checked_mul(factor)?,
        origin.y.checked_mul(factor)?,
        origin.z.checked_mul(factor)?,
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        operations::object::{ResampleFactor, upsample_objects},
        test_utilities::{HookRecorder, live_cells, two_material_scene},
    };
    use branded_id::U32Id;
    use ty_math::{TyVector3I32, TyVector3U32};
    use voxcore::VoxMain;

    fn factor(factor: u32) -> ResampleFactor {
        ResampleFactor::new(factor).unwrap()
    }

    #[test]
    fn splits_each_voxel_into_a_block_and_scales_the_origin() {
        // A `2 x 1 x 1` object at origin `(-1, 2, 0)` live at `(1, 0, 0)`.
        let mut main = two_material_scene(
            TyVector3U32::new(2, 1, 1),
            TyVector3I32::new(-1, 2, 0),
            &[(TyVector3U32::new(1, 0, 0), 1)],
        );

        upsample_objects(&mut main, &[U32Id::from_u32(0)], factor(2)).unwrap();

        let (_, object) = main.iter_objects().next().unwrap();

        assert_eq!(object.bounds(), TyVector3U32::new(4, 2, 2));
        assert_eq!(object.origin(), TyVector3I32::new(-2, 4, 0));

        let mut want = Vec::new();
        for x in 2..4 {
            for y in 0..2 {
                for z in 0..2 {
                    want.push((TyVector3U32::new(x, y, z), 1));
                }
            }
        }

        assert_eq!(live_cells(&main), want);
        assert_eq!(
            HookRecorder::events(&main),
            ["object 0 voxels resampled", "object 0 origin set"]
        );
        main.validate().unwrap();
    }

    #[test]
    fn a_scaled_origin_or_grid_leaving_its_range_is_an_error_that_changes_nothing() {
        let object_id = U32Id::from_u32(0);

        let mut main = two_material_scene(
            TyVector3U32::splat(2),
            TyVector3I32::new(i32::MAX / 2 + 1, 0, 0),
            &[(TyVector3U32::ZERO, 0)],
        );

        assert!(upsample_objects(&mut main, &[object_id], factor(2)).is_err());
        assert_eq!(
            main.object(object_id).unwrap().bounds(),
            TyVector3U32::splat(2)
        );
        assert!(HookRecorder::events(&main).is_empty());

        let mut main = two_material_scene(
            TyVector3U32::splat(1 << 9),
            TyVector3I32::ZERO,
            &[(TyVector3U32::ZERO, 0)],
        );

        // `1024^3` is past the cell cap, and `2^31 * 2` past u32.
        assert!(upsample_objects(&mut main, &[object_id], factor(2)).is_err());
        assert!(upsample_objects(&mut main, &[object_id], factor(1 << 23)).is_err());
        assert!(HookRecorder::events(&main).is_empty());
    }

    #[test]
    fn an_unknown_object_is_an_error() {
        let mut main: VoxMain = VoxMain::default();

        assert!(upsample_objects(&mut main, &[U32Id::from_u32(3)], factor(2)).is_err());
    }
}
