use crate::{
    ByteReader, Result, invalid, parse_u32, read_chunk, take, take_bool, take_f32, take_i32,
    take_u32, take_vec3f, take_vec3i,
};
use mvox::{
    MVoxCamera, MVoxColor, MVoxDict, MVoxFile, MVoxFrame, MVoxGroupNode, MVoxLayer, MVoxMaterial,
    MVoxMaterialType, MVoxModel, MVoxNodeAttributes, MVoxPalette, MVoxRenderObject, MVoxRotation,
    MVoxSceneNode, MVoxSceneNodeBody, MVoxShapeModel, MVoxShapeNode, MVoxTransformNode,
    MVoxUnknownChunk, MVoxVoxel,
};

/// The byte length of an `RGBA` chunk: 256 colors of four bytes each.
const RGBA_BYTES: usize = 256 * 4;

/// Parses the bytes of a MagicaVoxel `.vox` file into an [`MVoxFile`].
///
/// The header magic and version are read, then every chunk under the root
/// `MAIN` is dispatched into typed fields; a chunk this crate does not model is
/// preserved on [`MVoxFile::unknown_chunks`]. Parsing is bounds-checked, so a
/// truncated or malformed file is rejected with an error rather than masked.
pub fn from_mvox_file_bytes(bytes: &[u8]) -> Result<MVoxFile> {
    let mut reader = ByteReader::new(bytes);

    let magic = reader.read_array::<4>()?;
    if magic != *b"VOX " {
        return Err(invalid(format!(
            "not a .vox file: expected magic \"VOX \", found {magic:?}"
        )));
    }
    let version = reader.read_u32()?;

    let main = read_chunk(&mut reader)?;
    if main.id != *b"MAIN" {
        return Err(invalid(format!(
            "expected a MAIN chunk, found {:?}",
            main.id
        )));
    }

    let mut file = MVoxFile {
        version,
        ..Default::default()
    };
    let mut children = ByteReader::new(main.children);
    // A SIZE chunk announces the model its following XYZI chunk fills.
    let mut pending_size: Option<[u32; 3]> = None;
    // PACK, when present, redundantly states the model count; it is validated
    // against the SIZE/XYZI pairs and otherwise discarded.
    let mut pack_count: Option<u32> = None;

    while !children.is_empty() {
        let chunk = read_chunk(&mut children)?;
        let mut content = ByteReader::new(chunk.content);
        // Content-based chunks must consume their whole content region; the
        // slice-validated (RGBA / IMAP) and verbatim (unknown) chunks opt out.
        let mut exhaustive = true;
        match &chunk.id {
            b"PACK" => pack_count = Some(content.read_u32()?),
            b"SIZE" => {
                if pending_size.is_some() {
                    return Err(invalid(
                        "two SIZE chunks without an intervening XYZI chunk".to_owned(),
                    ));
                }
                pending_size = Some([
                    content.read_u32()?,
                    content.read_u32()?,
                    content.read_u32()?,
                ]);
            }
            b"XYZI" => {
                let size = pending_size.take().ok_or_else(|| {
                    invalid("XYZI chunk without a preceding SIZE chunk".to_owned())
                })?;
                file.models.push(read_model(size, &mut content)?);
            }
            b"RGBA" => {
                file.palette = Some(read_rgba(chunk.content)?);
                exhaustive = false;
            }
            b"nTRN" => file.scene_nodes.push(read_transform_node(&mut content)?),
            b"nGRP" => file.scene_nodes.push(read_group_node(&mut content)?),
            b"nSHP" => file.scene_nodes.push(read_shape_node(&mut content)?),
            b"MATL" => file.materials.push(read_material(&mut content)?),
            b"LAYR" => file.layers.push(read_layer(&mut content)?),
            b"rOBJ" => file.render_objects.push(MVoxRenderObject {
                attributes: MVoxDict(content.read_dict()?),
            }),
            b"rCAM" => file.cameras.push(read_camera(&mut content)?),
            b"NOTE" => file.palette_notes = read_note(&mut content)?,
            b"IMAP" => {
                file.index_map = Some(read_imap(chunk.content)?);
                exhaustive = false;
            }
            // Any other chunk is preserved verbatim so it survives the round
            // trip. This includes the legacy MATT material chunk, which this
            // crate does not model; MATL supersedes it.
            _ => {
                file.unknown_chunks.push(MVoxUnknownChunk {
                    id: chunk.id,
                    content: chunk.content.to_vec(),
                    children: chunk.children.to_vec(),
                });
                exhaustive = false;
            }
        }
        if exhaustive && !content.is_empty() {
            return Err(invalid(format!(
                "{:?} chunk has {} unexpected trailing content bytes",
                chunk.id,
                content.remaining().len()
            )));
        }
    }

    if pending_size.is_some() {
        return Err(invalid(
            "SIZE chunk without a following XYZI chunk".to_owned(),
        ));
    }
    if let Some(count) = pack_count
        && count as usize != file.models.len()
    {
        return Err(invalid(format!(
            "PACK declares {count} models but the file has {}",
            file.models.len()
        )));
    }

    Ok(file)
}

/// Reads an `XYZI` chunk's voxels, paired with its `SIZE`.
fn read_model(size: [u32; 3], content: &mut ByteReader) -> Result<MVoxModel> {
    let count = content.read_u32()? as usize;
    // Read the whole voxel block with one bounds check, then split it into
    // four-byte voxels. A bogus count overruns the slice, so read_bytes rejects it.
    let byte_len = count.checked_mul(4).ok_or_else(|| {
        invalid(format!(
            "XYZI voxel count {count} overflows the addressable byte range"
        ))
    })?;
    let voxels = content
        .read_bytes(byte_len)?
        .chunks_exact(4)
        .map(|voxel| MVoxVoxel {
            x: voxel[0],
            y: voxel[1],
            z: voxel[2],
            color_index: voxel[3],
        })
        .collect();
    Ok(MVoxModel { size, voxels })
}

/// Reads an `RGBA` chunk into a palette, undoing the on-disk index shift so file
/// color `j` lands at `colors[j + 1]`; the chunk's final color is unused.
fn read_rgba(content: &[u8]) -> Result<MVoxPalette> {
    if content.len() != RGBA_BYTES {
        return Err(invalid(format!(
            "RGBA chunk has {} bytes, expected {RGBA_BYTES}",
            content.len()
        )));
    }
    let mut colors = [MVoxColor::default(); 256];
    for index in 0..255usize {
        let offset = index * 4;
        colors[index + 1] = MVoxColor::new(
            content[offset],
            content[offset + 1],
            content[offset + 2],
            content[offset + 3],
        );
    }
    Ok(MVoxPalette { colors })
}

/// Reads an `IMAP` chunk's 256 palette-index associations. Files store these as
/// either 256 bytes or 256 little-endian `i32`s; both are accepted.
fn read_imap(content: &[u8]) -> Result<[u8; 256]> {
    match content.len() {
        256 => Ok(content.try_into().expect("length is 256")),
        1024 => {
            let mut map = [0u8; 256];
            for (index, slot) in map.iter_mut().enumerate() {
                *slot = content[index * 4];
            }
            Ok(map)
        }
        other => Err(invalid(format!(
            "IMAP chunk has {other} bytes, expected 256 or 1024"
        ))),
    }
}

/// Reads the node-attributes `DICT` shared by every scene node.
fn read_node_attributes(content: &mut ByteReader) -> Result<MVoxNodeAttributes> {
    let mut dict = content.read_dict()?;
    let name = take(&mut dict, "_name");
    let hidden = take_bool(&mut dict, "_hidden");
    Ok(MVoxNodeAttributes {
        name,
        hidden,
        extra: MVoxDict(dict),
    })
}

/// Reads one transform-node keyframe.
fn read_frame(content: &mut ByteReader) -> Result<MVoxFrame> {
    let mut dict = content.read_dict()?;
    let rotation = match take(&mut dict, "_r") {
        Some(value) => {
            let raw = parse_u32(&value)?;
            if raw > u8::MAX as u32 {
                return Err(invalid(format!(
                    "_r rotation {raw} is out of range 0..=255"
                )));
            }
            MVoxRotation(raw as u8)
        }
        None => MVoxRotation::IDENTITY,
    };
    let translation = take_vec3i(&mut dict, "_t")?.unwrap_or([0, 0, 0]);
    let frame_index = take_u32(&mut dict, "_f")?;
    Ok(MVoxFrame {
        rotation,
        translation,
        frame_index,
        extra: MVoxDict(dict),
    })
}

/// Reads an `nTRN` transform node.
fn read_transform_node(content: &mut ByteReader) -> Result<MVoxSceneNode> {
    let id = content.read_i32()?;
    let attributes = read_node_attributes(content)?;
    let child = content.read_i32()?;
    let _reserved = content.read_i32()?;
    let layer = content.read_i32()?;
    let frame_count = content.read_u32()? as usize;
    let mut frames = Vec::with_capacity(frame_count.min(content.remaining().len() / 4));
    for _ in 0..frame_count {
        frames.push(read_frame(content)?);
    }
    Ok(MVoxSceneNode {
        id,
        attributes,
        body: MVoxSceneNodeBody::Transform(MVoxTransformNode {
            child,
            layer,
            frames,
        }),
    })
}

/// Reads an `nGRP` group node.
fn read_group_node(content: &mut ByteReader) -> Result<MVoxSceneNode> {
    let id = content.read_i32()?;
    let attributes = read_node_attributes(content)?;
    let child_count = content.read_u32()? as usize;
    let mut children = Vec::with_capacity(child_count.min(content.remaining().len() / 4));
    for _ in 0..child_count {
        children.push(content.read_i32()?);
    }
    Ok(MVoxSceneNode {
        id,
        attributes,
        body: MVoxSceneNodeBody::Group(MVoxGroupNode { children }),
    })
}

/// Reads an `nSHP` shape node.
fn read_shape_node(content: &mut ByteReader) -> Result<MVoxSceneNode> {
    let id = content.read_i32()?;
    let attributes = read_node_attributes(content)?;
    let model_count = content.read_u32()? as usize;
    let mut models = Vec::with_capacity(model_count.min(content.remaining().len() / 4));
    for _ in 0..model_count {
        let model = content.read_u32()?;
        let mut dict = content.read_dict()?;
        let frame_index = take_u32(&mut dict, "_f")?;
        models.push(MVoxShapeModel {
            model,
            frame_index,
            extra: MVoxDict(dict),
        });
    }
    Ok(MVoxSceneNode {
        id,
        attributes,
        body: MVoxSceneNodeBody::Shape(MVoxShapeNode { models }),
    })
}

/// Reads a `MATL` material.
fn read_material(content: &mut ByteReader) -> Result<MVoxMaterial> {
    let id = content.read_i32()?;
    let mut dict = content.read_dict()?;
    let material_type = take(&mut dict, "_type").map(|value| parse_material_type(&value));
    let weight = take_f32(&mut dict, "_weight")?;
    let rough = take_f32(&mut dict, "_rough")?;
    let spec = take_f32(&mut dict, "_spec")?;
    let ior = take_f32(&mut dict, "_ior")?;
    let att = take_f32(&mut dict, "_att")?;
    let flux = take_f32(&mut dict, "_flux")?;
    Ok(MVoxMaterial {
        id,
        material_type,
        weight,
        rough,
        spec,
        ior,
        att,
        flux,
        extra: MVoxDict(dict),
    })
}

/// Maps a `_type` value to its variant, preserving unrecognized values.
fn parse_material_type(value: &str) -> MVoxMaterialType {
    match value {
        "_diffuse" => MVoxMaterialType::Diffuse,
        "_metal" => MVoxMaterialType::Metal,
        "_glass" => MVoxMaterialType::Glass,
        "_emit" => MVoxMaterialType::Emit,
        other => MVoxMaterialType::Other(other.to_owned()),
    }
}

/// Reads a `LAYR` layer.
fn read_layer(content: &mut ByteReader) -> Result<MVoxLayer> {
    let id = content.read_i32()?;
    let mut dict = content.read_dict()?;
    let name = take(&mut dict, "_name");
    let hidden = take_bool(&mut dict, "_hidden");
    let _reserved = content.read_i32()?;
    Ok(MVoxLayer {
        id,
        name,
        hidden,
        extra: MVoxDict(dict),
    })
}

/// Reads an `rCAM` render camera.
fn read_camera(content: &mut ByteReader) -> Result<MVoxCamera> {
    let id = content.read_i32()?;
    let mut dict = content.read_dict()?;
    let mode = take(&mut dict, "_mode");
    let focus = take_vec3f(&mut dict, "_focus")?;
    let angle = take_vec3f(&mut dict, "_angle")?;
    let radius = take_i32(&mut dict, "_radius")?;
    let frustum = take_f32(&mut dict, "_frustum")?;
    let fov = take_i32(&mut dict, "_fov")?;
    Ok(MVoxCamera {
        id,
        mode,
        focus,
        angle,
        radius,
        frustum,
        fov,
        extra: MVoxDict(dict),
    })
}

/// Reads a `NOTE` chunk's palette color names.
fn read_note(content: &mut ByteReader) -> Result<Vec<String>> {
    let count = content.read_u32()? as usize;
    let mut names = Vec::with_capacity(count.min(content.remaining().len() / 4));
    for _ in 0..count {
        names.push(content.read_string()?);
    }
    Ok(names)
}

#[cfg(test)]
mod tests {
    use crate::{from_mvox_file_bytes, to_mvox_file_bytes};
    use mvox::{
        MVoxCamera, MVoxColor, MVoxDict, MVoxFile, MVoxFrame, MVoxGroupNode, MVoxLayer,
        MVoxMaterial, MVoxMaterialType, MVoxModel, MVoxNodeAttributes, MVoxPalette,
        MVoxRenderObject, MVoxRotation, MVoxSceneNode, MVoxSceneNodeBody, MVoxShapeModel,
        MVoxShapeNode, MVoxTransformNode, MVoxUnknownChunk, MVoxVoxel,
    };
    use std::array;

    fn pair(key: &str, value: &str) -> (String, String) {
        (key.to_owned(), value.to_owned())
    }

    /// A file exercising every modeled chunk, including `extra` dictionaries,
    /// non-default frames, and an unknown chunk.
    fn sample_file() -> MVoxFile {
        let mut colors = MVoxPalette::default().colors;
        colors[1] = MVoxColor::new(10, 20, 30, 40);
        colors[255] = MVoxColor::new(1, 2, 3, 4);

        MVoxFile {
            version: 150,
            models: vec![
                MVoxModel {
                    size: [2, 1, 1],
                    voxels: vec![
                        MVoxVoxel {
                            x: 0,
                            y: 0,
                            z: 0,
                            color_index: 1,
                        },
                        MVoxVoxel {
                            x: 1,
                            y: 0,
                            z: 0,
                            color_index: 255,
                        },
                    ],
                },
                MVoxModel {
                    size: [1, 1, 1],
                    voxels: vec![MVoxVoxel {
                        x: 0,
                        y: 0,
                        z: 0,
                        color_index: 2,
                    }],
                },
            ],
            palette: Some(MVoxPalette { colors }),
            scene_nodes: vec![
                MVoxSceneNode {
                    id: 0,
                    attributes: MVoxNodeAttributes {
                        name: Some("root".to_owned()),
                        hidden: Some(false),
                        extra: MVoxDict(vec![pair("_meta", "x")]),
                    },
                    body: MVoxSceneNodeBody::Transform(MVoxTransformNode {
                        child: 1,
                        layer: -1,
                        frames: vec![
                            MVoxFrame {
                                rotation: MVoxRotation(105),
                                translation: [1, -2, 3],
                                frame_index: Some(0),
                                extra: MVoxDict(vec![pair("_custom", "1")]),
                            },
                            MVoxFrame::default(),
                        ],
                    }),
                },
                MVoxSceneNode {
                    id: 1,
                    attributes: MVoxNodeAttributes::default(),
                    body: MVoxSceneNodeBody::Group(MVoxGroupNode { children: vec![2] }),
                },
                MVoxSceneNode {
                    id: 2,
                    attributes: MVoxNodeAttributes {
                        name: Some("shape".to_owned()),
                        hidden: Some(true),
                        extra: MVoxDict::default(),
                    },
                    body: MVoxSceneNodeBody::Shape(MVoxShapeNode {
                        models: vec![MVoxShapeModel {
                            model: 0,
                            frame_index: Some(0),
                            extra: MVoxDict(vec![pair("_x", "y")]),
                        }],
                    }),
                },
            ],
            materials: vec![MVoxMaterial {
                id: 1,
                material_type: Some(MVoxMaterialType::Glass),
                weight: Some(0.5),
                rough: Some(0.1),
                spec: None,
                ior: Some(1.5),
                att: None,
                flux: None,
                extra: MVoxDict(vec![pair("_g", "0.8")]),
            }],
            layers: vec![
                MVoxLayer {
                    id: 0,
                    name: Some("Layer 0".to_owned()),
                    hidden: None,
                    extra: MVoxDict::default(),
                },
                MVoxLayer {
                    id: 1,
                    name: None,
                    hidden: Some(true),
                    extra: MVoxDict(vec![pair("_color", "255 0 0")]),
                },
            ],
            render_objects: vec![MVoxRenderObject {
                attributes: MVoxDict(vec![pair("_type", "_ground"), pair("_color", "1 1 1")]),
            }],
            cameras: vec![MVoxCamera {
                id: 0,
                mode: Some("pers".to_owned()),
                focus: Some([1.0, 2.5, -3.0]),
                angle: Some([0.0, 0.0, 0.0]),
                radius: Some(8),
                frustum: Some(0.25),
                fov: Some(45),
                extra: MVoxDict(vec![pair("_aperture", "0")]),
            }],
            palette_notes: vec!["red".to_owned(), "green".to_owned()],
            index_map: Some(array::from_fn(|i| i as u8)),
            unknown_chunks: vec![MVoxUnknownChunk {
                id: *b"MATT",
                content: vec![1, 2, 3, 4],
                children: Vec::new(),
            }],
        }
    }

    #[test]
    fn round_trips_full_file() {
        let file = sample_file();
        let bytes = to_mvox_file_bytes(&file);
        let decoded = from_mvox_file_bytes(&bytes).unwrap();
        assert_eq!(decoded, file);
    }

    #[test]
    fn round_trips_empty_file() {
        let file = MVoxFile::default();
        let bytes = to_mvox_file_bytes(&file);
        assert_eq!(from_mvox_file_bytes(&bytes).unwrap(), file);
    }

    #[test]
    fn parses_hand_written_minimal_file() {
        // VOX header, version 150, MAIN wrapping one SIZE + one XYZI.
        let mut size = Vec::new();
        size.extend_from_slice(b"SIZE");
        size.extend_from_slice(&12u32.to_le_bytes());
        size.extend_from_slice(&0u32.to_le_bytes());
        size.extend_from_slice(&1u32.to_le_bytes());
        size.extend_from_slice(&1u32.to_le_bytes());
        size.extend_from_slice(&1u32.to_le_bytes());

        let mut xyzi = Vec::new();
        xyzi.extend_from_slice(b"XYZI");
        xyzi.extend_from_slice(&8u32.to_le_bytes());
        xyzi.extend_from_slice(&0u32.to_le_bytes());
        xyzi.extend_from_slice(&1u32.to_le_bytes());
        xyzi.extend_from_slice(&[0, 0, 0, 7]);

        let mut children = Vec::new();
        children.extend_from_slice(&size);
        children.extend_from_slice(&xyzi);

        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"VOX ");
        bytes.extend_from_slice(&150u32.to_le_bytes());
        bytes.extend_from_slice(b"MAIN");
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&(children.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&children);

        let file = from_mvox_file_bytes(&bytes).unwrap();
        assert_eq!(file.version, 150);
        assert_eq!(file.models.len(), 1);
        assert_eq!(file.models[0].size, [1, 1, 1]);
        assert_eq!(
            file.models[0].voxels,
            vec![MVoxVoxel {
                x: 0,
                y: 0,
                z: 0,
                color_index: 7
            }]
        );
        assert!(file.palette.is_none());
    }

    #[test]
    fn resolves_default_palette_when_absent() {
        let file = MVoxFile::default();
        assert_eq!(file.resolved_palette(), MVoxPalette::default());
    }

    #[test]
    fn rejects_bad_magic() {
        let bytes = b"XOX \x96\x00\x00\x00";
        assert!(from_mvox_file_bytes(bytes).is_err());
    }

    #[test]
    fn rejects_truncated_header() {
        assert!(from_mvox_file_bytes(b"VOX ").is_err());
        assert!(from_mvox_file_bytes(b"").is_err());
    }

    /// Wraps `children` bytes in a `VOX `/version/`MAIN` envelope.
    fn vox_file(children: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"VOX ");
        bytes.extend_from_slice(&150u32.to_le_bytes());
        bytes.extend_from_slice(b"MAIN");
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&(children.len() as u32).to_le_bytes());
        bytes.extend_from_slice(children);
        bytes
    }

    /// A leaf chunk: id, content length, no children, then `content`.
    fn chunk(id: &[u8; 4], content: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(id);
        bytes.extend_from_slice(&(content.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(content);
        bytes
    }

    fn size_content(x: u32, y: u32, z: u32) -> Vec<u8> {
        [x, y, z].iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    #[test]
    fn rejects_consecutive_size_chunks() {
        // SIZE, SIZE, XYZI: the second SIZE has no XYZI to pair the first with.
        let mut children = chunk(b"SIZE", &size_content(1, 1, 1));
        children.extend(chunk(b"SIZE", &size_content(2, 2, 2)));
        children.extend(chunk(b"XYZI", &0u32.to_le_bytes()));
        assert!(from_mvox_file_bytes(&vox_file(&children)).is_err());
    }

    #[test]
    fn rejects_trailing_content_bytes() {
        // A SIZE chunk padded with four trailing bytes is malformed.
        let mut content = size_content(1, 1, 1);
        content.extend_from_slice(&[0, 0, 0, 0]);
        assert!(from_mvox_file_bytes(&vox_file(&chunk(b"SIZE", &content))).is_err());
    }

    #[test]
    fn rejects_chunk_running_past_end() {
        // MAIN claims 100 child bytes but supplies none.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"VOX ");
        bytes.extend_from_slice(&150u32.to_le_bytes());
        bytes.extend_from_slice(b"MAIN");
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&100u32.to_le_bytes());
        assert!(from_mvox_file_bytes(&bytes).is_err());
    }
}
