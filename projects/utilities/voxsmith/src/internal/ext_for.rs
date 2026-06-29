use voxcore::{VoxMain, VoxValue};

/// The state's ext, but only when it is an [`Object`](VoxValue::Object)
/// carrying `key` as a top-level field. A foreign ext naming another format, or
/// no ext at all, returns `None`, which lets a `to_<format>` writer fall back
/// to synthesis without disturbing its lossless ext path.
pub fn ext_for<'a>(state: &'a VoxMain, key: &str) -> Option<&'a VoxValue> {
    let ext = state.ext()?;
    match ext {
        VoxValue::Object(map) if map.0.iter().any(|(name, _)| name == key) => Some(ext),
        _ => None,
    }
}
