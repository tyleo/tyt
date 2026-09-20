use meshconv::gltf::GltfContainer;
use std::path::Path;

/// The container a mesh writes: `to` when given, else the one `output`'s
/// extension implies, else `.glb`.
pub(crate) fn resolve_gltf_container(
    to: Option<GltfContainer>,
    output: Option<&Path>,
) -> GltfContainer {
    to.or_else(|| {
        let extension = output?.extension()?.to_str()?;
        GltfContainer::from_extension(extension)
    })
    .unwrap_or(GltfContainer::Glb)
}

#[cfg(test)]
mod tests {
    use super::resolve_gltf_container;
    use meshconv::gltf::GltfContainer;
    use std::path::Path;

    #[test]
    fn to_beats_the_output_extension_beats_glb() {
        assert_eq!(
            resolve_gltf_container(Some(GltfContainer::Gltf), Some(Path::new("out.glb"))),
            GltfContainer::Gltf
        );
        assert_eq!(
            resolve_gltf_container(None, Some(Path::new("out.gltf"))),
            GltfContainer::Gltf
        );
        assert_eq!(
            resolve_gltf_container(None, Some(Path::new("out.mesh"))),
            GltfContainer::Glb
        );
        assert_eq!(resolve_gltf_container(None, None), GltfContainer::Glb);
    }
}
