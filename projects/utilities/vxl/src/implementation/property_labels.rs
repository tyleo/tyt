use voxcore::VoxPalette;

/// A palette's property keys as display labels, array then scalar in id
/// order, a scalar key marked by a ` (scalar)` suffix.
pub(crate) fn property_labels(palette: &VoxPalette) -> Vec<String> {
    palette
        .iter_array_properties()
        .map(|(_, property)| property.name.clone())
        .chain(
            palette
                .iter_scalar_properties()
                .map(|(_, property)| format!("{} (scalar)", property.name)),
        )
        .collect()
}
