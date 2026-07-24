use voxcore::VoxPalette;

/// A palette's property keys as display labels, in id order.
pub(crate) fn property_labels(palette: &VoxPalette) -> Vec<String> {
    palette
        .iter_array_properties()
        .map(|(_, property)| property.name.clone())
        .collect()
}
