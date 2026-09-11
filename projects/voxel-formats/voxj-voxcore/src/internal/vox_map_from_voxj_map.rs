use crate::{Error, Result, vox_value_from_voxj_value};
use voxcore::{VoxMap, VoxMapEntry};
use voxj::VoxjMap;

/// Converts a [`VoxjMap`] into a [`VoxMap`], recursing through its values.
/// Rejects non-finite numbers and repeated keys, which a parsed document
/// never carries; the checks guard maps built in memory.
pub fn vox_map_from_voxj_map(map: &VoxjMap) -> Result<VoxMap> {
    let mut entries: Vec<VoxMapEntry> = Vec::with_capacity(map.entries().len());
    for entry in map.entries() {
        if entries.iter().any(|existing| existing.key == entry.key) {
            return Err(Error::invalid(format!(
                "json object key `{}` must be unique",
                entry.key
            )));
        }
        entries.push(VoxMapEntry {
            key: entry.key.clone(),
            value: vox_value_from_voxj_value(&entry.value)?,
        });
    }
    Ok(VoxMap::new(entries))
}
