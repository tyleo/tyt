use treegrid::TreeGridLabel;
use voxcore::VoxPalette;

/// A palette's property keys as grid entries, quoted labels in id order.
pub(crate) fn property_entries(palette: &VoxPalette) -> Vec<(TreeGridLabel, Option<String>)> {
    palette
        .iter_properties()
        .map(|(_, property)| (TreeGridLabel::quoted(property.name.clone()), None))
        .collect()
}
