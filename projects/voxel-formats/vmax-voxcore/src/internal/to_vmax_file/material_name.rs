/// The palette display name Voxel Max shows: the preserved name, or its default
/// when a synthesized palette has none.
pub(crate) fn material_name(name: &str) -> String {
    if name.is_empty() {
        "Palette #1".to_owned()
    } else {
        name.to_owned()
    }
}
