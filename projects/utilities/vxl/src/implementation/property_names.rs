use voxsmith::voxcore::VoxPalette;

/// A palette's property keys, in id order.
pub(crate) fn property_names(palette: &VoxPalette) -> Vec<&str> {
    palette
        .iter_properties()
        .map(|(_, property)| property.name.as_str())
        .collect()
}
