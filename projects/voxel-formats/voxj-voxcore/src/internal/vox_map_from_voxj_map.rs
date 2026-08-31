use crate::{Error, Result, vox_value_from_voxj_value};
use voxcore::{VoxMap, VoxValue};
use voxj::VoxjMap;

/// Converts a [`VoxjMap`] into a [`VoxMap`], recursing through its values.
/// Rejects non-finite numbers and repeated keys, which a parsed document
/// never carries; the checks guard maps built in memory.
pub fn vox_map_from_voxj_map(map: &VoxjMap) -> Result<VoxMap> {
    let mut entries: Vec<(String, VoxValue)> = Vec::with_capacity(map.0.len());
    for (key, value) in &map.0 {
        if entries.iter().any(|(existing, _)| existing == key) {
            return Err(Error::invalid(format!(
                "json object key `{key}` must be unique"
            )));
        }
        entries.push((key.clone(), vox_value_from_voxj_value(value)?));
    }
    Ok(VoxMap(entries))
}
