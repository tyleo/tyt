use crate::Result;
use branded_id::U32Id;
use std::collections::HashSet;
use voxcore::{BVoxValuePool, BVoxValuePoolValue, VoxExt, VoxMain};

/// Releases each of `value_ids` no material of any palette draws, in order and
/// once each.
pub fn release_undrawn_values<T: VoxExt>(
    main: &mut VoxMain<T>,
    value_ids: impl IntoIterator<Item = (U32Id<BVoxValuePool>, U32Id<BVoxValuePoolValue>)>,
) -> Result<()> {
    let mut drawn_value_ids = HashSet::new();

    for (_, palette) in main.iter_palettes() {
        for (property_id, property) in palette.iter_properties() {
            for material_id in palette.iter_materials() {
                let value_id = palette
                    .value_id(material_id, property_id)
                    .expect("a live material holds a value for every property");

                drawn_value_ids.insert((property.value_pool_id, value_id));
            }
        }
    }

    let mut released_value_ids = HashSet::new();

    for (value_pool_id, value_id) in value_ids {
        if !drawn_value_ids.contains(&(value_pool_id, value_id))
            && released_value_ids.insert((value_pool_id, value_id))
        {
            main.release_value_pool_value(value_pool_id, value_id)?;
        }
    }

    Ok(())
}
