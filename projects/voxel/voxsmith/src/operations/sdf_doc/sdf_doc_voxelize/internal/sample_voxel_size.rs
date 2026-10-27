use crate::{
    Error, Result,
    operations::sdf_doc::{Bounds3d, SdfPlace, SdfShapes, part_bounds},
    utilities::GridResolution,
};
use sdfcore::SdfState;
use std::collections::HashSet;
use ty_math::TyVector3F64;

/// The voxel size `resolution` sets for the model in `state`, whose places are
/// `places`.
pub fn sample_voxel_size(
    state: &SdfState,
    places: &[SdfPlace],
    shapes: &SdfShapes,
    resolution: GridResolution,
) -> Result<f64> {
    let (reference, count) = match resolution {
        GridResolution::VoxelSize(voxel_size) => {
            return if voxel_size > 0.0 && voxel_size.is_finite() {
                Ok(voxel_size)
            } else {
                Err(Error::invalid(format!(
                    "voxel size must be above zero, not {voxel_size}"
                )))
            };
        }

        GridResolution::ReferenceCount { reference, count } => (reference, count),
    };

    if count == 0 {
        return Err(Error::invalid("resolution count must be above zero, not 0"));
    }

    let world = places
        .iter()
        .flat_map(|place| {
            state.nodes[place.node_id.to_usize_id()]
                .child_object_ids
                .iter()
                .filter_map(|object_id| part_bounds(state, shapes, *object_id))
                .map(|bounds| Bounds3d {
                    min: bounds.min + place.offset,
                    max: bounds.max + place.offset,
                })
        })
        .reduce(|world, part| world.union(&part))
        .map_or(TyVector3F64::ZERO, |world| world.max - world.min);

    let mut object_ids = HashSet::new();

    let objects: Vec<TyVector3F64> = places
        .iter()
        .flat_map(|place| &state.nodes[place.node_id.to_usize_id()].child_object_ids)
        .filter(|object_id| object_ids.insert(**object_id))
        .filter_map(|object_id| part_bounds(state, shapes, *object_id))
        .map(|bounds| bounds.max - bounds.min)
        .collect();

    let side = reference.side(world, &objects);

    if side > 0.0 {
        Ok(side / f64::from(count))
    } else {
        Err(Error::invalid(format!(
            "resolution reference {reference:?} must measure above zero, not {side}"
        )))
    }
}
