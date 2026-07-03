use voxcore::VoxPalette;

/// A palette's attribute keys, in id order.
pub(crate) fn attribute_names(palette: &VoxPalette) -> Vec<&str> {
    palette.iter_attributes().map(|(_, name)| name).collect()
}
