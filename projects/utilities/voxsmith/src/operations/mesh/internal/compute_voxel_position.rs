use vox_value_language::{Components, Dimension, Domain, Value};
use voxcore::VoxObject;

/// Each live voxel's grid position from the minimum corner of the live
/// extent, a voxel `u32` vec3 array in raster order.
pub(crate) fn compute_voxel_position(object: &VoxObject) -> Value {
    let origin = object
        .live_extent()
        .map(|(minimum, _)| minimum.to_array())
        .unwrap_or([0; 3]);

    let mut components = Vec::with_capacity(object.live_count() * 3);

    for voxel_id in object.iter_live() {
        let position = object
            .voxel_position(voxel_id)
            .expect("a live voxel is within the grid")
            .to_array();

        components.extend((0..3).map(|axis| position[axis] - origin[axis]));
    }

    Value::new(Domain::Voxel, Dimension::Vec3, Components::U32(components))
        .expect("three components per voxel fill a vec3 array")
}

#[cfg(test)]
mod tests {
    use crate::operations::mesh::compute_voxel_position;
    use ty_math::TyVector3U32;
    use vox_value_language::{Components, Domain};
    use voxcore::VoxObject;

    #[test]
    fn positions_start_at_the_live_extent_in_raster_order() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(4, 4, 4)).unwrap();
        for position in [[3, 2, 1], [1, 2, 1], [1, 3, 2]] {
            let voxel_id = object
                .voxel_id(TyVector3U32::new(position[0], position[1], position[2]))
                .unwrap();
            object.retain_voxel(voxel_id, &[]).unwrap();
        }

        let value = compute_voxel_position(&object);

        assert_eq!(value.domain(), Domain::Voxel);
        assert_eq!(
            value.components(),
            &Components::U32(vec![0, 0, 0, 0, 1, 1, 2, 0, 0])
        );
    }
}
