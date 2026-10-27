use crate::{BLOCK_IMAGE_SIZE, ByteReader, Chunk, DecodePng, Error, Result, invalid};
use goxl::{
    GoxlBlock, GoxlCamera, GoxlDict, GoxlFile, GoxlImage, GoxlLayer, GoxlLayerBlock, GoxlLight,
    GoxlMaterial, GoxlPreview, GoxlShape, GoxlUnknownChunk, GoxlVoxel,
};

/// Parses the bytes of a Goxel `.gox` file into a [`GoxlFile`] through
/// `dependencies`.
///
/// The `"GOX "` magic and version are read, then every chunk is dispatched into
/// typed fields: `IMG ` image metadata, the `PREV` preview, shared `BL16` voxel
/// blocks, `MATE` materials, `LAYR` layers, `CAMR` cameras, and the `LIGH`
/// settings. A `BL16` block and the preview are decoded from their PNGs into
/// pixel/voxel arrays. A chunk this crate does not model is preserved verbatim
/// on [`GoxlFile::unknown_chunks`]. Parsing is bounds-checked, so a truncated or
/// malformed file is rejected with an error rather than masked.
///
/// Block positions are kept exactly as stored; the legacy version-1 coordinate
/// shift Goxel applies at load time is not performed, so a file round-trips
/// byte-for-byte through [`to_gox_file_bytes`](crate::to_gox_file_bytes())
/// (modulo PNG re-compression, which is lossless for the pixels).
pub fn from_gox_file_bytes<D: DecodePng>(dependencies: &D, bytes: &[u8]) -> Result<GoxlFile> {
    let mut reader = ByteReader::new(bytes);

    let magic = reader.read_array::<4>()?;
    if magic != *b"GOX " {
        return Err(invalid(format!(
            "not a .gox file: expected magic \"GOX \", found {magic:?}"
        )));
    }
    let version = reader.read_i32()?;

    let mut file = GoxlFile {
        version,
        ..Default::default()
    };

    while !reader.is_empty() {
        let chunk = read_chunk(&mut reader)?;
        match &chunk.id {
            b"IMG " => file.image = read_image(chunk.data)?,

            b"PREV" => file.preview = Some(read_preview(dependencies, chunk.data)?),

            b"BL16" => file.blocks.push(read_block(dependencies, chunk.data)?),

            b"MATE" => file.materials.push(read_material(chunk.data)?),

            b"LAYR" => file.layers.push(read_layer(chunk.data)?),

            b"CAMR" => file.cameras.push(read_camera(chunk.data)?),

            b"LIGH" => file.light = Some(read_light(chunk.data)?),

            // Any other chunk is preserved verbatim so it survives the round
            // trip.
            _ => file.unknown_chunks.push(GoxlUnknownChunk {
                id: chunk.id,
                data: chunk.data.to_vec(),
            }),
        }
    }

    Ok(file)
}

/// Reads an `IMG ` chunk's image metadata.
fn read_image(data: &[u8]) -> Result<GoxlImage> {
    let mut dict = read_dict(&mut ByteReader::new(data))?;
    let bounding_box = take_mat4(&mut dict, "box")?;
    Ok(GoxlImage {
        bounding_box,
        extra: GoxlDict(dict),
    })
}

/// Reads a `PREV` chunk's preview PNG into `RGBA` pixels.
fn read_preview<D: DecodePng>(dependencies: &D, data: &[u8]) -> Result<GoxlPreview> {
    let image = dependencies.decode_png(data).map_err(Error::Png)?;
    Ok(GoxlPreview {
        width: image.width,
        height: image.height,
        pixels: image.pixels,
    })
}

/// Reads a `BL16` chunk's `64 x 64` PNG into a `16 x 16 x 16` voxel block. Each
/// pixel maps directly to the voxel at the same index.
fn read_block<D: DecodePng>(dependencies: &D, data: &[u8]) -> Result<GoxlBlock> {
    let image = dependencies.decode_png(data).map_err(Error::Png)?;
    let (width, height) = (image.width, image.height);
    if width != BLOCK_IMAGE_SIZE || height != BLOCK_IMAGE_SIZE {
        return Err(invalid(format!(
            "BL16 block PNG is {width}x{height}, expected {BLOCK_IMAGE_SIZE}x{BLOCK_IMAGE_SIZE}"
        )));
    }

    let voxels = image
        .pixels
        .into_iter()
        .map(|[r, g, b, a]| GoxlVoxel { r, g, b, a })
        .collect();
    Ok(GoxlBlock { voxels })
}

/// Reads a `MATE` chunk's material, defaulting any absent key.
fn read_material(data: &[u8]) -> Result<GoxlMaterial> {
    let mut dict = read_dict(&mut ByteReader::new(data))?;
    let mut material = GoxlMaterial::default();
    if let Some(value) = take_string(&mut dict, "name")? {
        material.name = value;
    }
    if let Some(value) = take_vec4f(&mut dict, "color")? {
        material.base_color = value;
    }
    if let Some(value) = take_f32(&mut dict, "metallic")? {
        material.metallic = value;
    }
    if let Some(value) = take_f32(&mut dict, "roughness")? {
        material.roughness = value;
    }
    if let Some(value) = take_vec3f(&mut dict, "emission")? {
        material.emission = value;
    }
    material.extra = GoxlDict(dict);
    Ok(material)
}

/// Reads a `LAYR` chunk: the placed-block list, then the layer dictionary.
fn read_layer(data: &[u8]) -> Result<GoxlLayer> {
    let mut reader = ByteReader::new(data);

    let block_count = reader.read_u32()? as usize;
    // Each placed block is five `i32`s (index, x, y, z, reserved), so cap the
    // pre-allocation at what the remaining bytes allow.
    let mut blocks = Vec::with_capacity(block_count.min(reader.remaining().len() / 20));
    for _ in 0..block_count {
        let block_index = reader.read_i32()?;
        let position = [reader.read_i32()?, reader.read_i32()?, reader.read_i32()?];
        let _reserved = reader.read_i32()?;
        blocks.push(GoxlLayerBlock {
            block_index,
            position,
        });
    }

    let mut dict = read_dict(&mut reader)?;
    let mut layer = GoxlLayer {
        blocks,
        ..Default::default()
    };
    if let Some(value) = take_string(&mut dict, "name")? {
        layer.name = value;
    }
    if let Some(value) = take_mat4(&mut dict, "mat")? {
        layer.transform = value;
    }
    if let Some(value) = take_i32(&mut dict, "id")? {
        layer.id = value;
    }
    if let Some(value) = take_i32(&mut dict, "base_id")? {
        layer.base_id = value;
    }
    if let Some(value) = take_i32(&mut dict, "material")? {
        layer.material = value;
    }
    if let Some(value) = take_i32(&mut dict, "mode")? {
        layer.mode = value;
    }
    if let Some(value) = take_string(&mut dict, "img-path")? {
        layer.image_path = Some(value);
    }
    if let Some(value) = take_mat4(&mut dict, "box")? {
        layer.bounding_box = Some(value);
    }
    layer.shape = take_shape(&mut dict);
    if let Some(value) = take_color(&mut dict, "color")? {
        layer.color = Some(value);
    }
    if let Some(value) = take_bool(&mut dict, "visible")? {
        layer.visible = value;
    }
    layer.extra = GoxlDict(dict);
    Ok(layer)
}

/// Reads a `CAMR` chunk's camera, defaulting any absent key.
fn read_camera(data: &[u8]) -> Result<GoxlCamera> {
    let mut dict = read_dict(&mut ByteReader::new(data))?;
    let mut camera = GoxlCamera::default();
    if let Some(value) = take_string(&mut dict, "name")? {
        camera.name = value;
    }
    if let Some(value) = take_f32(&mut dict, "dist")? {
        camera.distance = value;
    }
    if let Some(value) = take_bool(&mut dict, "ortho")? {
        camera.orthographic = value;
    }
    if let Some(value) = take_mat4(&mut dict, "mat")? {
        camera.transform = value;
    }
    // The active-camera flag is a present-but-empty marker.
    camera.active = take(&mut dict, "active").is_some();
    camera.extra = GoxlDict(dict);
    Ok(camera)
}

/// Reads a `LIGH` chunk's light settings, defaulting any absent key.
fn read_light(data: &[u8]) -> Result<GoxlLight> {
    let mut dict = read_dict(&mut ByteReader::new(data))?;
    let mut light = GoxlLight::default();
    if let Some(value) = take_f32(&mut dict, "pitch")? {
        light.pitch = value;
    }
    if let Some(value) = take_f32(&mut dict, "yaw")? {
        light.yaw = value;
    }
    if let Some(value) = take_f32(&mut dict, "intensity")? {
        light.intensity = value;
    }
    if let Some(value) = take_bool(&mut dict, "fixed")? {
        light.fixed = value;
    }
    if let Some(value) = take_f32(&mut dict, "ambient")? {
        light.ambient = value;
    }
    if let Some(value) = take_f32(&mut dict, "shadow")? {
        light.shadow = value;
    }
    light.extra = GoxlDict(dict);
    Ok(light)
}

/// Reads the layer `"shape"` key. A recognized shape name is lifted onto the
/// typed field; an unrecognized value is left in the dictionary so it survives
/// in `extra` rather than being lost.
fn take_shape(dict: &mut Vec<(String, Vec<u8>)>) -> Option<GoxlShape> {
    let value = dict.iter().find(|(key, _)| key == "shape")?.1.clone();
    let shape = match value.as_slice() {
        b"sphere" => GoxlShape::Sphere,
        b"cube" => GoxlShape::Cube,
        b"cylinder" => GoxlShape::Cylinder,
        _ => return None,
    };
    dict.retain(|(key, _)| key != "shape");
    Some(shape)
}

/// Reads one chunk from `reader`, advancing past the whole chunk including its
/// trailing `CRC` word. The CRC is read and discarded: Goxel writes it as zero
/// and does not validate it.
fn read_chunk<'a>(reader: &mut ByteReader<'a>) -> Result<Chunk<'a>> {
    let id = reader.read_array::<4>()?;
    let length = reader.read_u32()? as usize;
    let data = reader.read_bytes(length)?;
    let _crc = reader.read_u32()?;
    Ok(Chunk { id, data })
}

/// Reads a chunk's trailing `DICT` from `reader`, consuming it to the end.
///
/// Each entry is a `u32` key length, the UTF-8 key bytes, a `u32` value length,
/// then the value bytes; values are kept as raw bytes since the format stores
/// each as a typed binary blob. Reading stops at the end of `reader` or, as
/// Goxel's reader does, at a zero-length key marker. Goxel never writes that
/// marker, so a dictionary normally runs exactly to the chunk's end.
fn read_dict(reader: &mut ByteReader) -> Result<Vec<(String, Vec<u8>)>> {
    let mut pairs = Vec::new();
    while !reader.is_empty() {
        let key_len = reader.read_u32()? as usize;
        if key_len == 0 {
            break;
        }
        let key = reader.read_bytes(key_len)?;
        let key = String::from_utf8(key.to_vec())
            .map_err(|error| invalid(format!("dict key is not valid UTF-8: {error}")))?;
        let value_len = reader.read_u32()? as usize;
        let value = reader.read_bytes(value_len)?.to_vec();
        pairs.push((key, value));
    }
    Ok(pairs)
}

/// Removes every pair whose key is `key`, returning the last such value. Taking
/// all occurrences keeps a modeled key from lingering in the leftover `extra`
/// dictionary when a malformed chunk repeats it; keeping the last mirrors Goxel's
/// reader, which copies each matching entry in turn so the final one wins.
fn take(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Option<Vec<u8>> {
    let mut value = None;
    pairs.retain(|(pair_key, pair_value)| {
        if pair_key != key {
            return true;
        }
        value = Some(pair_value.clone());
        false
    });
    value
}

/// [`take`], then read the value as a UTF-8 string.
fn take_string(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<String>> {
    take(pairs, key)
        .map(|value| as_string(key, &value))
        .transpose()
}

/// [`take`], then read the value as a little-endian `f32`.
fn take_f32(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<f32>> {
    take(pairs, key)
        .map(|value| as_f32(key, &value))
        .transpose()
}

/// [`take`], then read the value as a little-endian `i32`.
fn take_i32(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<i32>> {
    take(pairs, key)
        .map(|value| as_i32(key, &value))
        .transpose()
}

/// [`take`], then read the value as Goxel's one-byte boolean.
fn take_bool(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<bool>> {
    take(pairs, key)
        .map(|value| as_bool(key, &value))
        .transpose()
}

/// [`take`], then read the value as a `4 x 4` matrix of little-endian `f32`s.
fn take_mat4(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<[[f32; 4]; 4]>> {
    take(pairs, key)
        .map(|value| as_mat4(key, &value))
        .transpose()
}

/// [`take`], then read the value as four little-endian `f32`s.
fn take_vec4f(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<[f32; 4]>> {
    take(pairs, key)
        .map(|value| as_vec4f(key, &value))
        .transpose()
}

/// [`take`], then read the value as three little-endian `f32`s.
fn take_vec3f(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<[f32; 3]>> {
    take(pairs, key)
        .map(|value| as_vec3f(key, &value))
        .transpose()
}

/// [`take`], then read the value as four `[r, g, b, a]` bytes.
fn take_color(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<[u8; 4]>> {
    take(pairs, key)
        .map(|value| as_color(key, &value))
        .transpose()
}

/// Reads a value as a UTF-8 string.
fn as_string(key: &str, bytes: &[u8]) -> Result<String> {
    String::from_utf8(bytes.to_vec()).map_err(|error| {
        invalid(format!(
            "dict key {key:?} value is not valid UTF-8: {error}"
        ))
    })
}

/// Reads a value as a little-endian `f32`, requiring exactly four bytes.
fn as_f32(key: &str, bytes: &[u8]) -> Result<f32> {
    Ok(f32::from_le_bytes(fixed(key, bytes, "an f32")?))
}

/// Reads a value as a little-endian `i32`, requiring exactly four bytes.
fn as_i32(key: &str, bytes: &[u8]) -> Result<i32> {
    Ok(i32::from_le_bytes(fixed(key, bytes, "an i32")?))
}

/// Reads a value as Goxel's one-byte boolean, requiring exactly one byte; any
/// non-zero byte is `true`.
fn as_bool(key: &str, bytes: &[u8]) -> Result<bool> {
    Ok(fixed::<1>(key, bytes, "a bool")?[0] != 0)
}

/// Reads a value as a `4 x 4` matrix of little-endian `f32`s in row-major
/// (C array) order, requiring exactly 64 bytes.
fn as_mat4(key: &str, bytes: &[u8]) -> Result<[[f32; 4]; 4]> {
    let bytes = fixed::<64>(key, bytes, "a 4x4 matrix")?;
    let mut matrix = [[0.0f32; 4]; 4];
    for (index, cell) in matrix.iter_mut().flatten().enumerate() {
        let offset = index * 4;
        *cell = f32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
    }
    Ok(matrix)
}

/// Reads a value as four little-endian `f32`s, requiring exactly 16 bytes.
fn as_vec4f(key: &str, bytes: &[u8]) -> Result<[f32; 4]> {
    let bytes = fixed::<16>(key, bytes, "a 4-float vector")?;
    Ok([
        f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        f32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
    ])
}

/// Reads a value as three little-endian `f32`s, requiring exactly 12 bytes.
fn as_vec3f(key: &str, bytes: &[u8]) -> Result<[f32; 3]> {
    let bytes = fixed::<12>(key, bytes, "a 3-float vector")?;
    Ok([
        f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
    ])
}

/// Reads a value as four `[r, g, b, a]` bytes, requiring exactly four bytes.
fn as_color(key: &str, bytes: &[u8]) -> Result<[u8; 4]> {
    fixed(key, bytes, "an RGBA color")
}

/// Interprets `bytes` as exactly `N` bytes, mapping a length mismatch to an
/// error that names the key and the expected shape.
fn fixed<const N: usize>(key: &str, bytes: &[u8], expected: &str) -> Result<[u8; N]> {
    bytes.try_into().map_err(|_| {
        Error::Invalid(format!(
            "dict key {key:?} expected {N} bytes for {expected}, found {}",
            bytes.len()
        ))
    })
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{DependenciesImpl, from_gox_file_bytes, to_gox_file_bytes};
    use goxl::{
        GoxlBlock, GoxlCamera, GoxlDict, GoxlFile, GoxlImage, GoxlLayer, GoxlLayerBlock, GoxlLight,
        GoxlMaterial, GoxlPreview, GoxlShape, GoxlUnknownChunk, GoxlVoxel,
    };

    /// A `4 x 4` matrix with distinct float cells, for transform/box fields.
    fn matrix(base: f32) -> [[f32; 4]; 4] {
        let mut matrix = [[0.0f32; 4]; 4];
        for (index, cell) in matrix.iter_mut().flatten().enumerate() {
            *cell = base + index as f32 * 0.5;
        }
        matrix
    }

    /// A full `16 x 16 x 16` block: mostly empty, with a few tagged voxels so the
    /// PNG round trip is exercised on real pixel values.
    fn block() -> GoxlBlock {
        let mut voxels = vec![GoxlVoxel::default(); GoxlBlock::SIZE.pow(3) as usize];
        voxels[0] = GoxlVoxel::new(10, 20, 30);
        voxels[1] = GoxlVoxel::new(255, 0, 0);
        voxels[100] = GoxlVoxel {
            r: 1,
            g: 2,
            b: 3,
            a: 128,
        };
        *voxels.last_mut().unwrap() = GoxlVoxel::new(7, 8, 9);
        GoxlBlock { voxels }
    }

    /// A pair whose value is raw bytes, for `extra` dictionaries.
    fn extra(key: &str, value: &[u8]) -> (String, Vec<u8>) {
        (key.to_owned(), value.to_vec())
    }

    /// A file exercising every modeled chunk, including `extra` dictionaries,
    /// optional keys, a clone layer, a shape layer, a preview, and an unknown
    /// chunk.
    fn sample_file() -> GoxlFile {
        GoxlFile {
            version: 2,
            image: GoxlImage {
                bounding_box: Some(matrix(1.0)),
                extra: GoxlDict(vec![extra("vendor", &[1, 2, 3, 4])]),
            },
            preview: Some(GoxlPreview {
                width: 2,
                height: 3,
                pixels: vec![
                    [0, 0, 0, 0],
                    [255, 255, 255, 255],
                    [1, 2, 3, 4],
                    [5, 6, 7, 8],
                    [9, 10, 11, 12],
                    [13, 14, 15, 16],
                ],
            }),
            blocks: vec![
                block(),
                GoxlBlock {
                    voxels: vec![GoxlVoxel::new(50, 60, 70); GoxlBlock::SIZE.pow(3) as usize],
                },
            ],
            materials: vec![
                GoxlMaterial {
                    name: "Default".to_owned(),
                    base_color: [0.25, 0.5, 0.75, 1.0],
                    metallic: 0.1,
                    roughness: 0.9,
                    emission: [0.0, 0.5, 1.0],
                    extra: GoxlDict(vec![extra("_custom", &[9])]),
                },
                GoxlMaterial::default(),
            ],
            layers: vec![
                GoxlLayer {
                    name: "Layer 0".to_owned(),
                    id: 1,
                    base_id: 0,
                    material: 0,
                    mode: 0,
                    visible: true,
                    transform: matrix(2.0),
                    blocks: vec![
                        GoxlLayerBlock {
                            block_index: 0,
                            position: [0, 0, 0],
                        },
                        GoxlLayerBlock {
                            block_index: 1,
                            position: [-16, 16, -32],
                        },
                    ],
                    bounding_box: Some(matrix(3.0)),
                    image_path: Some("/tmp/ref.png".to_owned()),
                    shape: None,
                    color: None,
                    extra: GoxlDict(vec![extra("_layer", &[7, 7])]),
                },
                GoxlLayer {
                    name: "Clone".to_owned(),
                    id: 2,
                    base_id: 1,
                    material: 1,
                    mode: 1,
                    visible: false,
                    transform: matrix(4.0),
                    blocks: Vec::new(),
                    bounding_box: None,
                    image_path: None,
                    shape: None,
                    color: None,
                    extra: GoxlDict::default(),
                },
                GoxlLayer {
                    name: "Shape".to_owned(),
                    id: 3,
                    base_id: 0,
                    material: 0,
                    mode: 0,
                    visible: true,
                    transform: matrix(5.0),
                    blocks: Vec::new(),
                    bounding_box: None,
                    image_path: None,
                    shape: Some(GoxlShape::Cylinder),
                    color: Some([200, 150, 100, 255]),
                    extra: GoxlDict::default(),
                },
            ],
            cameras: vec![
                GoxlCamera {
                    name: "Camera".to_owned(),
                    distance: 12.5,
                    orthographic: true,
                    transform: matrix(6.0),
                    active: true,
                    extra: GoxlDict(vec![extra("_cam", &[3, 3, 3])]),
                },
                GoxlCamera::default(),
            ],
            light: Some(GoxlLight {
                pitch: 0.5,
                yaw: -0.25,
                intensity: 1.5,
                fixed: true,
                ambient: 0.2,
                shadow: 0.8,
                extra: GoxlDict(vec![extra("_light", &[1])]),
            }),
            unknown_chunks: vec![GoxlUnknownChunk {
                id: *b"XTRA",
                data: vec![9, 8, 7, 6, 5],
            }],
        }
    }

    #[test]
    fn round_trips_full_file() {
        let file = sample_file();
        let bytes = to_gox_file_bytes(&DependenciesImpl, &file);
        let decoded = from_gox_file_bytes(&DependenciesImpl, &bytes).unwrap();
        assert_eq!(decoded, file);
    }

    #[test]
    fn round_trips_default_file() {
        let file = GoxlFile::default();
        let bytes = to_gox_file_bytes(&DependenciesImpl, &file);
        assert_eq!(
            from_gox_file_bytes(&DependenciesImpl, &bytes).unwrap(),
            file
        );
    }

    #[test]
    fn round_trips_block_voxels_exactly() {
        let file = GoxlFile {
            blocks: vec![block()],
            ..Default::default()
        };
        let decoded = from_gox_file_bytes(
            &DependenciesImpl,
            &to_gox_file_bytes(&DependenciesImpl, &file),
        )
        .unwrap();
        assert_eq!(decoded.blocks, file.blocks);
    }

    #[test]
    fn round_trips_preview_pixels_exactly() {
        let preview = GoxlPreview {
            width: 4,
            height: 2,
            pixels: (0..8).map(|i| [i, i + 1, i + 2, i + 3]).collect(),
        };
        let file = GoxlFile {
            preview: Some(preview.clone()),
            ..Default::default()
        };
        let decoded = from_gox_file_bytes(
            &DependenciesImpl,
            &to_gox_file_bytes(&DependenciesImpl, &file),
        )
        .unwrap();
        assert_eq!(decoded.preview, Some(preview));
    }

    #[test]
    fn encodes_degenerate_preview_without_panicking() {
        // A zero-area preview cannot be a PNG; the writer must skip it rather than
        // panic, so the file decodes back with no preview.
        let file = GoxlFile {
            preview: Some(GoxlPreview::default()),
            ..Default::default()
        };
        let decoded = from_gox_file_bytes(
            &DependenciesImpl,
            &to_gox_file_bytes(&DependenciesImpl, &file),
        )
        .unwrap();
        assert_eq!(decoded.preview, None);
    }

    #[test]
    fn encodes_oversized_preview_without_panicking() {
        // Huge dimensions whose pixel count no real buffer can match must not
        // overflow or OOM the writer; the inconsistent preview is skipped.
        let file = GoxlFile {
            preview: Some(GoxlPreview {
                width: u32::MAX,
                height: u32::MAX,
                pixels: vec![[1, 2, 3, 4]],
            }),
            ..Default::default()
        };
        let decoded = from_gox_file_bytes(
            &DependenciesImpl,
            &to_gox_file_bytes(&DependenciesImpl, &file),
        )
        .unwrap();
        assert_eq!(decoded.preview, None);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = b"BOX ".to_vec();
        bytes.extend_from_slice(&2i32.to_le_bytes());
        assert!(from_gox_file_bytes(&DependenciesImpl, &bytes).is_err());
    }

    #[test]
    fn rejects_truncated_header() {
        assert!(from_gox_file_bytes(&DependenciesImpl, b"").is_err());
        assert!(from_gox_file_bytes(&DependenciesImpl, b"GOX ").is_err());
    }

    /// A chunk: id, data length, data, then a zero CRC word.
    fn chunk(id: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut bytes = id.to_vec();
        bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(data);
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes
    }

    /// One dict entry: key length, key, value length, value.
    fn dict_entry(key: &str, value: &[u8]) -> Vec<u8> {
        let mut bytes = (key.len() as u32).to_le_bytes().to_vec();
        bytes.extend_from_slice(key.as_bytes());
        bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
        bytes.extend_from_slice(value);
        bytes
    }

    /// Wraps chunk bytes in a `GOX `/version envelope.
    fn gox_file(chunks: &[u8]) -> Vec<u8> {
        let mut bytes = b"GOX ".to_vec();
        bytes.extend_from_slice(&2i32.to_le_bytes());
        bytes.extend_from_slice(chunks);
        bytes
    }

    #[test]
    fn parses_hand_written_light_chunk() {
        // A LIGH chunk built by hand, with one modeled key and one extra key,
        // to exercise dict decoding independently of the encoder.
        let mut data = dict_entry("pitch", &0.75f32.to_le_bytes());
        data.extend(dict_entry("_x", &[1, 2]));
        let bytes = gox_file(&chunk(b"LIGH", &data));

        let file = from_gox_file_bytes(&DependenciesImpl, &bytes).unwrap();
        let light = file.light.unwrap();
        assert_eq!(light.pitch, 0.75);
        assert_eq!(light.extra.get("_x"), Some(&[1, 2][..]));
    }

    #[test]
    fn duplicate_modeled_key_keeps_the_last() {
        // Goxel's reader copies each matching entry in turn, so a repeated modeled
        // key resolves to its final occurrence; the decoder matches that.
        let mut data = dict_entry("dist", &1.0f32.to_le_bytes());
        data.extend(dict_entry("dist", &2.0f32.to_le_bytes()));
        let bytes = gox_file(&chunk(b"CAMR", &data));

        let file = from_gox_file_bytes(&DependenciesImpl, &bytes).unwrap();
        assert_eq!(file.cameras[0].distance, 2.0);
        // Both occurrences are consumed, so none lingers in `extra`.
        assert!(file.cameras[0].extra.get("dist").is_none());
    }

    #[test]
    fn keeps_unknown_shape_in_extra() {
        // A shape value Goxel never writes stays raw in `extra` rather than being
        // dropped or lifted onto the typed field.
        let mut data = chunk_layer_prefix(0);
        data.extend(dict_entry("shape", b"pyramid"));
        let bytes = gox_file(&chunk(b"LAYR", &data));

        let file = from_gox_file_bytes(&DependenciesImpl, &bytes).unwrap();
        let layer = &file.layers[0];
        assert_eq!(layer.shape, None);
        assert_eq!(layer.extra.get("shape"), Some(&b"pyramid"[..]));
    }

    #[test]
    fn rejects_chunk_running_past_end() {
        // A chunk claiming 100 data bytes but supplying none.
        let mut bytes = gox_file(b"");
        bytes.extend_from_slice(b"IMG ");
        bytes.extend_from_slice(&100u32.to_le_bytes());
        assert!(from_gox_file_bytes(&DependenciesImpl, &bytes).is_err());
    }

    #[test]
    fn rejects_wrong_size_dict_value() {
        // A material "metallic" value that is not four bytes is malformed.
        let data = dict_entry("metallic", &[1, 2]);
        let bytes = gox_file(&chunk(b"MATE", &data));
        assert!(from_gox_file_bytes(&DependenciesImpl, &bytes).is_err());
    }

    /// The non-dict prefix of a LAYR chunk with `count` placed blocks omitted.
    fn chunk_layer_prefix(count: u32) -> Vec<u8> {
        count.to_le_bytes().to_vec()
    }

    #[test]
    fn unused_shape_variants_are_recognized() {
        for (shape, name) in [
            (GoxlShape::Sphere, "sphere"),
            (GoxlShape::Cube, "cube"),
            (GoxlShape::Cylinder, "cylinder"),
        ] {
            let mut data = chunk_layer_prefix(0);
            data.extend(dict_entry("shape", name.as_bytes()));
            let bytes = gox_file(&chunk(b"LAYR", &data));
            let file = from_gox_file_bytes(&DependenciesImpl, &bytes).unwrap();
            assert_eq!(file.layers[0].shape, Some(shape));
        }
    }
}
