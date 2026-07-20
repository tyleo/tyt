use voxcore::VoxPalette;

/// A palette's attribute keys, in array-property id order.
pub(crate) fn attribute_names(palette: &VoxPalette) -> Vec<&str> {
    palette
        .iter_array_properties()
        .map(|(_, property)| property.name.as_str())
        .collect()
}
