use vmax::{VMaxFile, VMaxMaterial, VMaxObject};

/// An object's material-palette display name and its exact material list from
/// the settings sidecar, or empty when it has no sidecar or no materials.
pub(crate) fn material_list(serde: &VMaxFile, object: &VMaxObject) -> (String, Vec<VMaxMaterial>) {
    let Some(stem) = object.palette.strip_suffix(".png") else {
        return (String::new(), Vec::new());
    };
    let sidecar = format!("{stem}.settings.vmaxpsb");
    match serde.palette_settings_files.get(&sidecar) {
        Some(settings) => (settings.name.clone(), settings.materials.clone()),
        None => (String::new(), Vec::new()),
    }
}
