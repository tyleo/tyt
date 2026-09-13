use crate::{DecodeBase64, Error, GltfFile, Result, data_uri_payload, percent_decode};
use gltf::{Document, buffer::Source};

/// The bytes of every buffer, in buffer order: the blob for the buffer with
/// no URI, a decoded data URI, or the loose file at a relative URI.
pub fn resolve_buffers<D: DecodeBase64>(
    dependencies: &D,
    file: &GltfFile,
    document: &Document,
) -> Result<Vec<Vec<u8>>> {
    document
        .buffers()
        .map(|buffer| {
            let bytes = match buffer.source() {
                Source::Bin => file.blob.clone().ok_or_else(|| {
                    Error::invalid("a buffer names no URI and the document has no binary chunk")
                })?,
                Source::Uri(uri) => resolve_uri(dependencies, file, uri)?,
            };

            if bytes.len() < buffer.length() {
                return Err(Error::invalid(format!(
                    "buffer {} declares {} bytes but holds {}",
                    buffer.index(),
                    buffer.length(),
                    bytes.len()
                )));
            }

            Ok(bytes)
        })
        .collect()
}

/// The bytes at a URI: a decoded data URI, or the loose file under the URI,
/// raw or percent-decoded.
pub fn resolve_uri<D: DecodeBase64>(
    dependencies: &D,
    file: &GltfFile,
    uri: &str,
) -> Result<Vec<u8>> {
    if let Some((_, payload)) = data_uri_payload(uri) {
        return dependencies
            .decode_base64(payload)
            .map_err(|reason| Error::invalid(format!("a data URI is not base64: {reason}")));
    }

    if uri.starts_with("data:") {
        return Err(Error::invalid("a data URI is not base64 encoded"));
    }

    file.loose_files
        .get(uri)
        .or_else(|| percent_decode(uri).and_then(|decoded| file.loose_files.get(&decoded)))
        .cloned()
        .ok_or_else(|| Error::invalid(format!("the document has no file at `{uri}`")))
}
