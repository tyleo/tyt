use voxcore::VoxPalette;

/// A palette's attribute keys, in binding id order.
pub(crate) fn attribute_names(palette: &VoxPalette) -> Vec<&str> {
    palette
        .iter_bindings()
        .map(|(_, binding)| binding.attribute.as_str())
        .collect()
}
