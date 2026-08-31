use crate::voxj_value_from_vox_value;
use voxcore::VoxMap;
use voxj::VoxjMap;

/// Converts a [`VoxMap`] into the equivalent [`VoxjMap`], recursing through
/// its values.
pub fn voxj_map_from_vox_map(map: &VoxMap) -> VoxjMap {
    VoxjMap(
        map.0
            .iter()
            .map(|(key, value)| (key.clone(), voxj_value_from_vox_value(value)))
            .collect(),
    )
}
