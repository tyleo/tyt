use crate::{
    BLOCK_IMAGE_SIZE, ByteReader, Result, decode_png_rgba, invalid, read_chunk, read_dict, take,
    take_bool, take_color, take_f32, take_i32, take_mat4, take_string, take_vec3f, take_vec4f,
};
use goxl::{
    GoxlBlock, GoxlCamera, GoxlDict, GoxlFile, GoxlImage, GoxlLayer, GoxlLayerBlock, GoxlLight,
    GoxlMaterial, GoxlPreview, GoxlShape, GoxlUnknownChunk, GoxlVoxel,
};

/// Parses the bytes of a Goxel `.gox` file into a [`GoxlFile`].
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
/// byte-for-byte through [`to_gox_file_bytes`](crate::to_gox_file_bytes)
/// (modulo PNG re-compression, which is lossless for the pixels).
pub fn from_gox_file_bytes(bytes: &[u8]) -> Result<GoxlFile> {
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
            b"PREV" => file.preview = Some(read_preview(chunk.data)?),
            b"BL16" => file.blocks.push(read_block(chunk.data)?),
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
fn read_preview(data: &[u8]) -> Result<GoxlPreview> {
    let (width, height, rgba) = decode_png_rgba(data)?;
    let pixels = rgba
        .chunks_exact(4)
        .map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]])
        .collect();
    Ok(GoxlPreview {
        width,
        height,
        pixels,
    })
}

/// Reads a `BL16` chunk's `64 x 64` PNG into a `16 x 16 x 16` voxel block. Each
/// pixel maps directly to the voxel at the same index.
fn read_block(data: &[u8]) -> Result<GoxlBlock> {
    let (width, height, rgba) = decode_png_rgba(data)?;
    if width != BLOCK_IMAGE_SIZE || height != BLOCK_IMAGE_SIZE {
        return Err(invalid(format!(
            "BL16 block PNG is {width}x{height}, expected {BLOCK_IMAGE_SIZE}x{BLOCK_IMAGE_SIZE}"
        )));
    }
    let voxels = rgba
        .chunks_exact(4)
        .map(|pixel| GoxlVoxel {
            r: pixel[0],
            g: pixel[1],
            b: pixel[2],
            a: pixel[3],
        })
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

#[cfg(test)]
mod tests {
    use crate::{from_gox_file_bytes, to_gox_file_bytes};
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
        let bytes = to_gox_file_bytes(&file);
        let decoded = from_gox_file_bytes(&bytes).unwrap();
        assert_eq!(decoded, file);
    }

    #[test]
    fn round_trips_default_file() {
        let file = GoxlFile::default();
        let bytes = to_gox_file_bytes(&file);
        assert_eq!(from_gox_file_bytes(&bytes).unwrap(), file);
    }

    #[test]
    fn round_trips_block_voxels_exactly() {
        let file = GoxlFile {
            blocks: vec![block()],
            ..Default::default()
        };
        let decoded = from_gox_file_bytes(&to_gox_file_bytes(&file)).unwrap();
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
        let decoded = from_gox_file_bytes(&to_gox_file_bytes(&file)).unwrap();
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
        let decoded = from_gox_file_bytes(&to_gox_file_bytes(&file)).unwrap();
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
        let decoded = from_gox_file_bytes(&to_gox_file_bytes(&file)).unwrap();
        assert_eq!(decoded.preview, None);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = b"BOX ".to_vec();
        bytes.extend_from_slice(&2i32.to_le_bytes());
        assert!(from_gox_file_bytes(&bytes).is_err());
    }

    #[test]
    fn rejects_truncated_header() {
        assert!(from_gox_file_bytes(b"").is_err());
        assert!(from_gox_file_bytes(b"GOX ").is_err());
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

        let file = from_gox_file_bytes(&bytes).unwrap();
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

        let file = from_gox_file_bytes(&bytes).unwrap();
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

        let file = from_gox_file_bytes(&bytes).unwrap();
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
        assert!(from_gox_file_bytes(&bytes).is_err());
    }

    #[test]
    fn rejects_wrong_size_dict_value() {
        // A material "metallic" value that is not four bytes is malformed.
        let data = dict_entry("metallic", &[1, 2]);
        let bytes = gox_file(&chunk(b"MATE", &data));
        assert!(from_gox_file_bytes(&bytes).is_err());
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
            let file = from_gox_file_bytes(&bytes).unwrap();
            assert_eq!(file.layers[0].shape, Some(shape));
        }
    }
}
