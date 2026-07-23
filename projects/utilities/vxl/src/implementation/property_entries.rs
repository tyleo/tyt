use treegrid::TreeGridLabel;
use voxcore::VoxPalette;

/// A palette's property keys as grid entries, array then scalar in id order,
/// each a quoted label paired with the `(scalar)` annotation on a scalar key.
pub(crate) fn property_entries(palette: &VoxPalette) -> Vec<(TreeGridLabel, Option<String>)> {
    palette
        .iter_array_properties()
        .map(|(_, property)| (TreeGridLabel::quoted(property.name.clone()), None))
        .chain(palette.iter_scalar_properties().map(|(_, property)| {
            (
                TreeGridLabel::quoted(property.name.clone()),
                Some("(scalar)".to_owned()),
            )
        }))
        .collect()
}
