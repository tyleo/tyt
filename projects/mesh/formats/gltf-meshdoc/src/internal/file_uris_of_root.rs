use crate::{Result, VxlExtras, data_uri_payload, value_from_extras};
use gltf::json::Root;
use serde_json::Value;

/// The relative URIs of the files a root's document holds: every image URI
/// that is not a data URI, then every file a material's or mesh's
/// `extras.vxl.values` entry references, each once, in reference order.
pub fn file_uris_of_root(root: &Root) -> Result<Vec<String>> {
    let mut uris: Vec<String> = Vec::new();

    for uri in root.images.iter().filter_map(|image| image.uri.as_deref()) {
        if data_uri_payload(uri).is_none()
            && !uri.starts_with("data:")
            && !uris.iter().any(|listed| listed == uri)
        {
            uris.push(uri.to_owned());
        }
    }

    let extras = root
        .materials
        .iter()
        .map(|material| &material.extras)
        .chain(root.meshes.iter().map(|mesh| &mesh.extras));

    for extras in extras {
        let (_, vxl) = VxlExtras::split(&value_from_extras(extras)?)?;

        for value in vxl.values.values() {
            let Value::Object(object) = value else {
                continue;
            };

            if let Some(uri) = object.get("uri").and_then(Value::as_str)
                && !uris.iter().any(|listed| listed == uri)
            {
                uris.push(uri.to_owned());
            }
        }
    }

    Ok(uris)
}
