use crate::voxj_value_from_vox_value;
use voxcore::VoxMapEntry;
use voxj::{VoxjMap, VoxjMapEntry};

/// Converts the entries of a [`VoxMap`](voxcore::VoxMap) into the
/// equivalent [`VoxjMap`], recursing through their values.
pub fn voxj_map_from_vox_map_entries(entries: &[VoxMapEntry]) -> VoxjMap {
    VoxjMap::new(
        entries
            .iter()
            .map(|entry| VoxjMapEntry {
                key: entry.key.clone(),
                value: voxj_value_from_vox_value(&entry.value),
            })
            .collect(),
    )
}
