use crate::{Result, data_uri_payload, file_uris_of_root, is_glb, parse_glb};
use gltf::json::Root;

/// The relative URIs the bytes of a `.gltf` or `.glb` file reference, the
/// loose files a caller reads beside the primary before
/// [`from_gltf_bytes`](crate::codec::from_gltf_bytes): every buffer and
/// image URI that is not a data URI, and every file a material's or mesh's
/// `extras.vxl.values` entry references, each once, in reference order.
pub fn gltf_loose_uris(bytes: &[u8]) -> Result<Vec<String>> {
    let json = if is_glb(bytes) {
        parse_glb(bytes)?.0
    } else {
        bytes.to_vec()
    };

    let root = Root::from_slice(&json).map_err(gltf::Error::Deserialize)?;

    let mut uris: Vec<String> = Vec::new();

    for uri in root
        .buffers
        .iter()
        .filter_map(|buffer| buffer.uri.as_deref())
    {
        if data_uri_payload(uri).is_none()
            && !uri.starts_with("data:")
            && !uris.iter().any(|listed| listed == uri)
        {
            uris.push(uri.to_owned());
        }
    }

    for uri in file_uris_of_root(&root)? {
        if !uris.iter().any(|listed| listed == &uri) {
            uris.push(uri);
        }
    }

    Ok(uris)
}

#[cfg(test)]
mod tests {
    use crate::codec::gltf_loose_uris;

    #[test]
    fn lists_relative_uris_once_and_skips_data_uris() {
        let json = br#"{
            "asset": { "version": "2.0" },
            "buffers": [ { "byteLength": 4, "uri": "scene.bin" } ],
            "images": [
                { "uri": "skin.png" },
                { "uri": "data:image/png;base64,AAAA" },
                { "uri": "skin.png" }
            ],
            "materials": [
                { "extras": { "vxl": { "values": { "rows": { "uri": "values.json" } } } } }
            ],
            "meshes": [
                {
                    "primitives": [ { "attributes": { "POSITION": 0 } } ],
                    "extras": { "vxl": { "values": { "rows": { "uri": "values.json" }, "n": 1 } } }
                }
            ],
            "accessors": [ { "componentType": 5126, "count": 0, "type": "VEC3" } ]
        }"#;

        assert_eq!(
            gltf_loose_uris(json).unwrap(),
            ["scene.bin", "skin.png", "values.json"]
        );
    }
}
