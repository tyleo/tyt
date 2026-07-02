use crate::{
    MaterialMap, MaterialMeshRequest, MaterialSlot, ResourceStorage, Result, atlas_dimensions,
    bake_atlas_pixels, encode_rgba8_png, mesh_slices, resolve_used_materials, texel_center,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Map, Value, json};
use ty_math::TyVector3F32;
use voxcore::{VoxMain, VoxObject};

/// Which glTF container the document is assembled for, so images embed the way
/// that container carries them.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum MeshTarget {
    /// Binary glTF: embedded images share the one binary buffer.
    Glb,

    /// Text glTF: embedded images are base64 data URIs.
    Gltf,
}

/// A built glTF document with its packed binary buffer and any loose files.
pub(crate) struct MaterialDocument {
    /// The glTF JSON document.
    pub(crate) document: Value,

    /// The one binary buffer the accessors read, plus any images the GLB
    /// container embeds; empty when the object has no geometry.
    pub(crate) blob: Vec<u8>,

    /// Loose files to write beside the mesh, each a relative name and bytes.
    pub(crate) sidecars: Vec<(String, Vec<u8>)>,
}

/// Meshes `object` material-aware and lays it out as a glTF document with a
/// texel-center UV set and a material sampling the baked palette atlas. Each
/// requested map becomes one image and texture; recognized maps fill their glTF
/// slot and the rest are listed under the material's `extras`. `target` decides
/// how embedded images travel, and `request.storage` whether they are embedded,
/// loose, or both. An object with no geometry yields an empty scene.
pub(crate) fn build_material_document(
    state: &VoxMain,
    object: &VoxObject,
    request: &MaterialMeshRequest,
    target: MeshTarget,
) -> Result<MaterialDocument> {
    let used = resolve_used_materials(object);

    let geometry = mesh_slices(
        object,
        request.method,
        &|voxel| used.material_index(voxel).unwrap_or(0),
        true,
    );

    if geometry.indices.is_empty() {
        return Ok(MaterialDocument {
            document: json!({
                "asset": { "version": "2.0", "generator": "voxsmith" },
                "scene": 0,
                "scenes": [ { "nodes": [] } ]
            }),
            blob: Vec::new(),
            sidecars: Vec::new(),
        });
    }

    let (atlas_width, atlas_height) = atlas_dimensions(used.len());

    let scale = request.scale as f32;

    // Bake Z-up grid space to Y-up meter space, the inverse of the voxelizer's
    // mapping: rotate then scale positions, rotate the unit normals.
    let position = |p: TyVector3F32| TyVector3F32::new(p.x * scale, p.z * scale, -p.y * scale);
    let direction = |n: TyVector3F32| TyVector3F32::new(n.x, n.z, -n.y);

    // Positions, then normals, then UVs, then indices, each a buffer view. All
    // are 4-byte quantities, so the regions stay naturally aligned.
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

    let uvs_offset = blob.len();

    for &material in &geometry.materials {
        for value in texel_center(material, atlas_width, atlas_height) {
            blob.extend_from_slice(&value.to_le_bytes());
        }
    }

    let indices_offset = blob.len();

    for &index in &geometry.indices {
        blob.extend_from_slice(&index.to_le_bytes());
    }

    let positions_len = normals_offset;
    let normals_len = uvs_offset - normals_offset;
    let uvs_len = indices_offset - uvs_offset;
    let indices_len = blob.len() - indices_offset;
    let vertex_count = geometry.positions.len();
    let index_count = geometry.indices.len();

    let mut buffer_views = vec![
        json!({ "buffer": 0, "byteOffset": 0, "byteLength": positions_len, "target": 34962 }),
        json!({ "buffer": 0, "byteOffset": normals_offset, "byteLength": normals_len, "target": 34962 }),
        json!({ "buffer": 0, "byteOffset": uvs_offset, "byteLength": uvs_len, "target": 34962 }),
        json!({ "buffer": 0, "byteOffset": indices_offset, "byteLength": indices_len, "target": 34963 }),
    ];

    // Bake each map into an image, place it per the storage mode, and build the
    // parallel images / textures arrays plus any sidecar files.
    let mut images = Vec::with_capacity(request.maps.len());
    let mut textures = Vec::with_capacity(request.maps.len());
    let mut sidecars = Vec::new();

    for map in &request.maps {
        let pixels = bake_atlas_pixels(state, &used, &map.bake, atlas_width, atlas_height)?;
        let png = encode_rgba8_png(atlas_width, atlas_height, &pixels)?;

        let image = match placement(request.storage, target) {
            Placement::Buffer => {
                // Pad to a 4-byte boundary, then append the PNG as its own view.
                while blob.len() % 4 != 0 {
                    blob.push(0);
                }
                let offset = blob.len();
                blob.extend_from_slice(&png);
                let view = buffer_views.len();
                buffer_views.push(json!({
                    "buffer": 0, "byteOffset": offset, "byteLength": png.len()
                }));
                json!({ "bufferView": view, "mimeType": "image/png" })
            }
            Placement::DataUri => {
                json!({ "uri": format!("data:image/png;base64,{}", STANDARD.encode(&png)) })
            }
            Placement::ExternalUri => json!({ "uri": map.name }),
        };

        if writes_sidecar(request.storage) {
            sidecars.push((map.name.clone(), png));
        }

        // One texture per image, sharing the single nearest-neighbor sampler.
        textures.push(json!({ "sampler": 0, "source": images.len() }));
        images.push(image);
    }

    let material = build_material(&request.maps);

    let document = json!({
        "asset": { "version": "2.0", "generator": "voxsmith" },
        "scene": 0,
        "scenes": [ { "nodes": [0] } ],
        "nodes": [ { "mesh": 0 } ],
        "meshes": [ {
            "primitives": [ {
                "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 },
                "indices": 3,
                "material": 0,
                "mode": 4
            } ]
        } ],
        "accessors": [
            {
                "bufferView": 0, "componentType": 5126, "count": vertex_count,
                "type": "VEC3", "min": min.to_array().to_vec(), "max": max.to_array().to_vec()
            },
            { "bufferView": 1, "componentType": 5126, "count": vertex_count, "type": "VEC3" },
            { "bufferView": 2, "componentType": 5126, "count": vertex_count, "type": "VEC2" },
            { "bufferView": 3, "componentType": 5125, "count": index_count, "type": "SCALAR" }
        ],
        "bufferViews": buffer_views,
        "buffers": [ { "byteLength": blob.len() } ],
        "materials": [ material ],
        "textures": textures,
        "images": images,
        "samplers": [ {
            "magFilter": 9728, "minFilter": 9728, "wrapS": 33071, "wrapT": 33071
        } ]
    });

    Ok(MaterialDocument {
        document,
        blob,
        sidecars,
    })
}

/// How one image is carried, given the storage mode and container.
enum Placement {
    /// A buffer view in the one binary buffer (GLB embedding).
    Buffer,

    /// A base64 data URI on the image (text-glTF embedding).
    DataUri,

    /// A relative URI to a loose file (external storage).
    ExternalUri,
}

/// The placement embedded images take: a buffer view under GLB, a data URI
/// under text glTF, and a loose-file URI when stored externally.
fn placement(storage: ResourceStorage, target: MeshTarget) -> Placement {
    match (storage, target) {
        (ResourceStorage::External, _) => Placement::ExternalUri,
        (_, MeshTarget::Glb) => Placement::Buffer,
        (_, MeshTarget::Gltf) => Placement::DataUri,
    }
}

/// Whether a loose file is written for each image: for external storage (the
/// referenced file) and for both (a working copy beside the embedded one).
fn writes_sidecar(storage: ResourceStorage) -> bool {
    matches!(storage, ResourceStorage::External | ResourceStorage::Both)
}

/// Builds the one glTF material, wiring each map that has a standard slot into
/// it and listing the rest under `extras.vxl.maps` by name.
fn build_material(maps: &[MaterialMap]) -> Value {
    let mut material = Map::new();
    let mut pbr = Map::new();
    let mut extras_maps = Map::new();

    for (index, map) in maps.iter().enumerate() {
        let reference = json!({ "index": index });

        match map.slot {
            MaterialSlot::BaseColor => {
                pbr.insert("baseColorTexture".to_owned(), reference);
            }

            MaterialSlot::MetallicRoughness => {
                pbr.insert("metallicRoughnessTexture".to_owned(), reference);
            }

            MaterialSlot::Occlusion => {
                material.insert("occlusionTexture".to_owned(), reference);
            }

            MaterialSlot::OcclusionMetallicRoughness => {
                material.insert("occlusionTexture".to_owned(), reference.clone());
                pbr.insert("metallicRoughnessTexture".to_owned(), reference);
            }

            MaterialSlot::Emissive => {
                material.insert("emissiveTexture".to_owned(), reference);
                material.insert("emissiveFactor".to_owned(), json!([1.0, 1.0, 1.0]));
            }

            MaterialSlot::None => {
                extras_maps.insert(map.name.clone(), json!(index));
            }
        }
    }

    if !pbr.is_empty() {
        material.insert("pbrMetallicRoughness".to_owned(), Value::Object(pbr));
    }

    if !extras_maps.is_empty() {
        material.insert(
            "extras".to_owned(),
            json!({ "vxl": { "maps": Value::Object(extras_maps) } }),
        );
    }

    Value::Object(material)
}
