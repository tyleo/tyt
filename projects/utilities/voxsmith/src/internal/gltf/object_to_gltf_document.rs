use crate::{MeshMethod, object_to_mesh_geometry};
use serde_json::{Value, json};
use ty_math::TyVector3F32;
use voxcore::VoxObject;

/// Meshes `object` with `method` and lays its geometry out as a glTF JSON
/// document paired with the packed binary buffer the document's accessors read.
///
/// The returned document has one buffer with a `byteLength` but no `uri`, so a
/// GLB writer references it as the BIN chunk while a text-glTF writer adds a
/// base64 data URI. Vertex positions are baked into glTF's Y-up meter space:
/// each grid position is rotated Z-up to Y-up by `(x, y, z) -> (x, z, -y)`, the
/// inverse of the voxelizer's mapping, then scaled by `scale` meters per voxel;
/// normals take the rotation only. An object with no geometry yields an empty
/// scene and an empty buffer.
pub(crate) fn object_to_gltf_document(
    object: &VoxObject,
    method: MeshMethod,
    scale: f64,
) -> (Value, Vec<u8>) {
    let geometry = object_to_mesh_geometry(object, method);
    let scale = scale as f32;

    // Bake Z-up grid space to Y-up meter space: rotate then scale positions,
    // rotate the already-unit normals. `(x, y, z) -> (x, z, -y)` is the inverse
    // of the voxelizer's Y-up-to-Z-up mapping.
    let position = |p: TyVector3F32| p.zup_to_yup() * scale;
    let direction = |n: TyVector3F32| n.zup_to_yup();

    if geometry.indices.is_empty() {
        let empty = json!({
            "asset": { "version": "2.0", "generator": "voxsmith" },
            "scene": 0,
            "scenes": [ { "nodes": [] } ]
        });
        return (empty, Vec::new());
    }

    // One buffer holding positions, then normals, then indices, each region a
    // separate buffer view. All are 4-byte quantities, so the regions stay
    // naturally aligned with no padding.
    let mut blob = Vec::new();
    let mut min = TyVector3F32::splat(f32::INFINITY);
    let mut max = TyVector3F32::splat(f32::NEG_INFINITY);
    for &point in &geometry.positions {
        let point = position(point);
        min = min.component_min_with(&point);
        max = max.component_max_with(&point);

        for value in point.to_array() {
            blob.extend_from_slice(&value.to_le_bytes());
        }
    }

    let normals_offset = blob.len();

    for &normal in &geometry.normals {
        for value in direction(normal).to_array() {
            blob.extend_from_slice(&value.to_le_bytes());
        }
    }

    let indices_offset = blob.len();
    for &index in &geometry.indices {
        blob.extend_from_slice(&index.to_le_bytes());
    }

    let positions_len = normals_offset;
    let normals_len = indices_offset - normals_offset;
    let indices_len = blob.len() - indices_offset;
    let vertex_count = geometry.positions.len();
    let index_count = geometry.indices.len();
    let min = min.to_array().to_vec();
    let max = max.to_array().to_vec();

    // Component types: 5126 FLOAT, 5125 UNSIGNED_INT. Targets: 34962
    // ARRAY_BUFFER, 34963 ELEMENT_ARRAY_BUFFER. Primitive mode 4 is TRIANGLES.
    let document = json!({
        "asset": { "version": "2.0", "generator": "voxsmith" },
        "scene": 0,
        "scenes": [ { "nodes": [0] } ],
        "nodes": [ { "mesh": 0 } ],
        "meshes": [ {
            "primitives": [ {
                "attributes": { "POSITION": 0, "NORMAL": 1 },
                "indices": 2,
                "mode": 4
            } ]
        } ],
        "accessors": [
            {
                "bufferView": 0, "componentType": 5126, "count": vertex_count,
                "type": "VEC3", "min": min, "max": max
            },
            { "bufferView": 1, "componentType": 5126, "count": vertex_count, "type": "VEC3" },
            { "bufferView": 2, "componentType": 5125, "count": index_count, "type": "SCALAR" }
        ],
        "bufferViews": [
            { "buffer": 0, "byteOffset": 0, "byteLength": positions_len, "target": 34962 },
            { "buffer": 0, "byteOffset": normals_offset, "byteLength": normals_len, "target": 34962 },
            { "buffer": 0, "byteOffset": indices_offset, "byteLength": indices_len, "target": 34963 }
        ],
        "buffers": [ { "byteLength": blob.len() } ]
    });

    (document, blob)
}
