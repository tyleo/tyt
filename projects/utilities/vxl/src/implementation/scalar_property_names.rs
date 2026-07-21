use voxcore::VoxPalette;

/// A palette's scalar property keys, in id order.
pub(crate) fn scalar_property_names(palette: &VoxPalette) -> Vec<&str> {
    palette
        .iter_scalar_properties()
        .map(|(_, property)| property.name.as_str())
        .collect()
}
