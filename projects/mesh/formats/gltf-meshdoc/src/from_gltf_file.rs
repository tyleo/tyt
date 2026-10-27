use crate::{
    DecodeBase64, Error, GltfExt, GltfExtAnimation, GltfExtAnimationChannel,
    GltfExtAnimationOutput, GltfExtAnimationSampler, GltfExtAsset, GltfExtImage,
    GltfExtImageSource, GltfExtMaterial, GltfExtMesh, GltfExtMorphTarget, GltfExtNode,
    GltfExtPrimitive, GltfExtSampler, GltfExtScene, GltfExtSkin, GltfExtTexture, GltfFile,
    GltfMeshMain, Result, VxlExtras, data_uri_payload, file_uris_of_root, value_from_extras,
};
use branded_id::U32Id;
use gltf::{
    Accessor, Buffer, Document, Material, Node, Primitive, Semantic, Texture,
    accessor::{DataType, Dimensions, Item, Iter},
    animation::{
        Interpolation,
        util::{ReadOutputs, Reader as AnimationReader},
    },
    buffer::Source as BufferSource,
    image::Source,
    material::{AlphaMode, NormalTexture, OcclusionTexture},
    mesh::Mode,
    texture::{Info, MagFilter, MinFilter, WrappingMode},
};
use meshdoc::{
    BMeshFile, BMeshHierarchyNode, BMeshImage, BMeshMaterial, BMeshTexture, MeshAlphaMode,
    MeshAttributeComponents, MeshFile, MeshHierarchyNode, MeshImage, MeshImageMediaType,
    MeshImageSource, MeshMagFilter, MeshMain, MeshMaterial, MeshMinFilter, MeshObject,
    MeshPrimitive, MeshProperty, MeshPropertyValue, MeshTexture, MeshTextureRef, MeshTriangle,
    MeshVertexAttribute, MeshWrap,
};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashSet, btree_map::Entry};
use ty_math::{
    TyLinSrgbF64, TyLinSrgbaF64, TyQuaternionF64, TyTransformF64, TyVector2F64, TyVector3F64,
    TyVector4F64,
};

/// Loads a glTF [`GltfFile`] into a [`GltfMeshMain`], the inverse of
/// [`to_gltf_file`](crate::to_gltf_file()). The images become images, the
/// textures textures, the materials materials, the meshes objects of one
/// primitive per glTF primitive, and the nodes hierarchy nodes, with every
/// scene's nodes and every parentless node as the roots. The loose files the
/// images and the `extras.vxl.values` entries reference become the document's
/// files, in name order, because glTF has no file listing and the writer lands
/// them in that order. An image at a relative URI becomes an image over its
/// file. A material's or mesh's `extras.vxl.values` become its properties and a
/// primitive's `extras.vxl.name` its name. Properties and further vertex
/// attributes load in name order because glTF keys both by name. Positions,
/// normals, tangents, and node transforms copy straight because meshdoc shares
/// glTF's frame. The rest of the file goes to the ext. `dependencies` decodes
/// the data URIs.
///
/// Errors on:
///
/// 1. a reference the document cannot resolve
/// 2. a primitive of points or lines
/// 3. a sparse accessor
/// 4. a `vxl` extras entry of the wrong shape
/// 5. a cross-reference the checked insertions reject
pub fn from_gltf_file<D: DecodeBase64>(dependencies: &D, file: &GltfFile) -> Result<GltfMeshMain> {
    let document = Document::from_json(file.root.clone())?;

    let buffers = resolve_buffers(dependencies, file, &document)?;

    let get = |buffer: Buffer| buffers.get(buffer.index()).map(Vec::as_slice);

    let mut main = MeshMain::default();
    let mut ext = GltfExt {
        asset: asset_of(&document)?,
        extensions_used: document.extensions_used().map(str::to_owned).collect(),
        extensions_required: document.extensions_required().map(str::to_owned).collect(),
        extras: value_from_extras(&file.root.extras)?,
        extensions: extension_map(document.extensions()),
        buffer_uri: buffer_uri_of(&document),
        cameras: file
            .root
            .cameras
            .iter()
            .map(|camera| {
                serde_json::to_value(camera)
                    .map_err(|error| Error::invalid(format!("a camera is not JSON: {error}")))
            })
            .collect::<Result<_>>()?,
        ..Default::default()
    };

    // Each loose file is retained once under its URI, in name order.
    let mut file_uris = file_uris_of_root(&file.root)?;
    file_uris.sort();

    let mut file_ids: BTreeMap<String, U32Id<BMeshFile>> = BTreeMap::new();
    for uri in file_uris {
        let file_id = main.retain_file(MeshFile {
            name: uri.clone(),
            bytes: resolve_uri(dependencies, file, &uri)?,
        })?;

        file_ids.insert(uri, file_id);
    }

    let file_id_for_uri = |uri: &str| -> Result<U32Id<BMeshFile>> {
        Ok(*file_ids
            .get(uri)
            .expect("every referenced file URI was retained"))
    };

    // Every entity takes the id of its index, retained in order, so every
    // later reference is the index as an id.
    let mut image_ids = Vec::new();
    for image in document.images() {
        let (bytes, source, declared) = match image.source() {
            Source::View { view, mime_type } => {
                let buffer = get(view.buffer()).expect("a validated view names a resolved buffer");
                let bytes = buffer
                    .get(view.offset()..view.offset() + view.length())
                    .ok_or_else(|| {
                        Error::invalid(format!("image {} runs past its buffer", image.index()))
                    })?
                    .to_vec();
                (
                    MeshImageSource::Bytes(bytes),
                    Some(GltfExtImageSource::BufferView),
                    Some(mime_type),
                )
            }

            Source::Uri { uri, mime_type } => {
                let (bytes, source) = match data_uri_payload(uri) {
                    Some(_) => (
                        MeshImageSource::Bytes(resolve_uri(dependencies, file, uri)?),
                        Some(GltfExtImageSource::DataUri),
                    ),

                    None => (MeshImageSource::File(file_id_for_uri(uri)?), None),
                };
                let declared =
                    mime_type.or_else(|| data_uri_payload(uri).map(|(media_type, _)| media_type));
                (bytes, source, declared)
            }
        };

        let raw = match &bytes {
            MeshImageSource::Bytes(bytes) => bytes.as_slice(),

            MeshImageSource::File(file_id) => {
                &main
                    .file(*file_id)
                    .expect("the file was just retained")
                    .bytes
            }
        };

        let media_type = declared
            .and_then(MeshImageMediaType::from_media_type)
            .or_else(|| media_type_from_bytes(raw))
            .ok_or_else(|| {
                Error::invalid(format!(
                    "image {} is neither a PNG nor a JPEG",
                    image.index()
                ))
            })?;

        let image_id = main.retain_image(MeshImage {
            name: image.name().unwrap_or_default().to_owned(),
            media_type,
            source: bytes,
        })?;

        ext.images.insert(
            image_id,
            GltfExtImage {
                source,
                extras: value_from_extras(image.extras())?,
                extensions: extension_map(image.extensions()),
            },
        );

        image_ids.push(image_id);
    }

    let mut texture_ids = Vec::new();
    for texture in document.textures() {
        let texture_id = main.retain_texture(mesh_texture_of(&texture, &image_ids))?;

        let sampler = texture.sampler();
        let sampler = sampler.index().map(|_| GltfExtSampler {
            name: sampler.name().map(str::to_owned),
            extras: value_from_extras(sampler.extras()).unwrap_or_default(),
            extensions: extension_map(sampler.extensions()),
        });

        ext.textures.insert(
            texture_id,
            GltfExtTexture {
                name: texture.name().map(str::to_owned),
                sampler,
                extras: value_from_extras(texture.extras())?,
                extensions: extension_map(texture.extensions()),
            },
        );

        texture_ids.push(texture_id);
    }

    let mut material_ids = Vec::new();
    for material in document.materials() {
        let (extras, vxl) = VxlExtras::split(&value_from_extras(material.extras())?)?;

        if vxl.name.is_some() {
            return Err(Error::invalid(format!(
                "material {} carries extras `vxl.name`, which only a primitive has",
                material.index().expect("a listed material has an index")
            )));
        }

        let mut mesh_material = mesh_material_of(&material, &texture_ids);
        mesh_material.properties = read_properties(&vxl, &texture_ids, &file_id_for_uri)?;

        let material_id = main.retain_material(mesh_material)?;

        ext.materials.insert(
            material_id,
            GltfExtMaterial {
                extras,
                extensions: extension_map(material.extensions()),
            },
        );

        material_ids.push(material_id);
    }

    let mut object_ids = Vec::new();
    for mesh in document.meshes() {
        let (extras, vxl) = VxlExtras::split(&value_from_extras(mesh.extras())?)?;

        if vxl.name.is_some() {
            return Err(Error::invalid(format!(
                "mesh {} carries extras `vxl.name`, which only a primitive has",
                mesh.index()
            )));
        }

        let mut object = MeshObject::new(mesh.name().unwrap_or_default().to_owned());
        object.set_properties(read_properties(&vxl, &texture_ids, &file_id_for_uri)?);

        let mut primitives = BTreeMap::new();

        for primitive in mesh.primitives() {
            let (mesh_primitive, ext_primitive) = read_primitive(&primitive, &material_ids, get)?;
            let primitive_id = object.retain_primitive(mesh_primitive);
            primitives.insert(primitive_id, ext_primitive);
        }

        let object_id = main.retain_object(object)?;

        ext.meshes.insert(
            object_id,
            GltfExtMesh {
                weights: mesh.weights().map(<[f32]>::to_vec),
                primitives,
                extras,
                extensions: extension_map(mesh.extensions()),
            },
        );

        object_ids.push(object_id);
    }

    // Every node is retained in one batch, so children reference their ids
    // before they exist.
    let nodes = document
        .nodes()
        .map(|node| MeshHierarchyNode {
            name: node.name().unwrap_or_default().to_owned(),
            transform: transform_from_gltf(&node),
            child_node_ids: node
                .children()
                .map(|child| node_id_of(child.index()))
                .collect(),
            child_object_ids: node
                .mesh()
                .map(|mesh| object_ids[mesh.index()])
                .into_iter()
                .collect(),
        })
        .collect();

    let node_ids = main.retain_hierarchy_nodes(nodes)?;

    for node in document.nodes() {
        ext.nodes.insert(
            node_ids[node.index()],
            GltfExtNode {
                camera: node.camera().map(|camera| camera.index() as u32),
                skin: node.skin().map(|skin| skin.index() as u32),
                weights: node.weights().map(<[f32]>::to_vec),
                extras: value_from_extras(node.extras())?,
                extensions: extension_map(node.extensions()),
            },
        );
    }

    for scene in document.scenes() {
        ext.scenes.push(GltfExtScene {
            name: scene.name().map(str::to_owned),
            node_ids: scene.nodes().map(|node| node_ids[node.index()]).collect(),
            extras: value_from_extras(scene.extras())?,
            extensions: extension_map(scene.extensions()),
        });
    }

    ext.default_scene = document.default_scene().map(|scene| scene.index() as u32);

    main.set_root_hierarchy_node_ids(root_ids_of(&document, &node_ids))?;

    for skin in document.skins() {
        let matrices = skin
            .reader(get)
            .read_inverse_bind_matrices()
            .map(|matrices| {
                matrices
                    .map(|matrix| {
                        let mut flat = [0.0; 16];
                        for (column, values) in matrix.iter().enumerate() {
                            flat[column * 4..column * 4 + 4].copy_from_slice(values);
                        }
                        flat
                    })
                    .collect()
            });

        ext.skins.push(GltfExtSkin {
            name: skin.name().map(str::to_owned),
            inverse_bind_matrices: matrices,
            joint_node_ids: skin.joints().map(|joint| node_ids[joint.index()]).collect(),
            skeleton_node_id: skin.skeleton().map(|node| node_ids[node.index()]),
            extras: value_from_extras(skin.extras())?,
            extensions: extension_map(skin.extensions()),
        });
    }

    for animation in document.animations() {
        ext.animations
            .push(read_animation(&animation, &node_ids, get)?);
    }

    Ok(main.put_ext(ext))
}

/// The properties `vxl.values` entries give, in entry order.
fn read_properties(
    vxl: &VxlExtras,
    texture_ids: &[U32Id<BMeshTexture>],
    file_id_for_uri: &impl Fn(&str) -> Result<U32Id<BMeshFile>>,
) -> Result<Vec<MeshProperty>> {
    vxl.values
        .iter()
        .map(|(name, value)| {
            Ok(MeshProperty {
                name: name.clone(),
                value: property_value_from_json(name, value, texture_ids, file_id_for_uri)?,
            })
        })
        .collect()
}

/// The `asset` block.
fn asset_of(document: &Document) -> Result<GltfExtAsset> {
    let asset = &document.as_json().asset;

    Ok(GltfExtAsset {
        version: asset.version.clone(),
        min_version: asset.min_version.clone(),
        generator: asset.generator.clone(),
        copyright: asset.copyright.clone(),
        extras: value_from_extras(&asset.extras)?,
    })
}

/// The relative URI of the first buffer when it references a loose file.
fn buffer_uri_of(document: &Document) -> Option<String> {
    let uri = document.as_json().buffers.first()?.uri.as_deref()?;

    (data_uri_payload(uri).is_none() && !uri.starts_with("data:")).then(|| uri.to_owned())
}

/// The extensions an object carries outside the modeled set, or none.
fn extension_map(extensions: Option<&Map<String, Value>>) -> Map<String, Value> {
    extensions.cloned().unwrap_or_default()
}

/// The node id of the node at `index`: nodes take their index as their id.
fn node_id_of(index: usize) -> U32Id<BMeshHierarchyNode> {
    U32Id::from_u32(
        u32::try_from(index).expect("a validated document has fewer than u32::MAX nodes"),
    )
}

/// The roots: every scene's nodes in scene order, then every node no node
/// lists as a child, each once.
fn root_ids_of(
    document: &Document,
    node_ids: &[U32Id<BMeshHierarchyNode>],
) -> Vec<U32Id<BMeshHierarchyNode>> {
    let mut roots = Vec::new();
    let mut seen = HashSet::new();

    for scene in document.scenes() {
        for node in scene.nodes() {
            if seen.insert(node.index()) {
                roots.push(node_ids[node.index()]);
            }
        }
    }

    let children: HashSet<usize> = document
        .nodes()
        .flat_map(|node| node.children().map(|child| child.index()))
        .collect();

    for node in document.nodes() {
        if !children.contains(&node.index()) && seen.insert(node.index()) {
            roots.push(node_ids[node.index()]);
        }
    }

    roots
}

/// The meshdoc texture for a glTF texture.
fn mesh_texture_of(texture: &Texture, image_ids: &[U32Id<BMeshImage>]) -> MeshTexture {
    let sampler = texture.sampler();

    MeshTexture {
        image_id: image_ids[texture.source().index()],
        mag_filter: sampler.mag_filter().map(|filter| match filter {
            MagFilter::Nearest => MeshMagFilter::Nearest,
            MagFilter::Linear => MeshMagFilter::Linear,
        }),
        min_filter: sampler.min_filter().map(|filter| match filter {
            MinFilter::Nearest => MeshMinFilter::Nearest,
            MinFilter::Linear => MeshMinFilter::Linear,
            MinFilter::NearestMipmapNearest => MeshMinFilter::NearestMipmapNearest,
            MinFilter::LinearMipmapNearest => MeshMinFilter::LinearMipmapNearest,
            MinFilter::NearestMipmapLinear => MeshMinFilter::NearestMipmapLinear,
            MinFilter::LinearMipmapLinear => MeshMinFilter::LinearMipmapLinear,
        }),
        wrap_s: wrap_of(sampler.wrap_s()),
        wrap_t: wrap_of(sampler.wrap_t()),
    }
}

/// The meshdoc wrap for a glTF wrapping mode.
fn wrap_of(mode: WrappingMode) -> MeshWrap {
    match mode {
        WrappingMode::ClampToEdge => MeshWrap::ClampToEdge,
        WrappingMode::MirroredRepeat => MeshWrap::MirroredRepeat,
        WrappingMode::Repeat => MeshWrap::Repeat,
    }
}

/// The meshdoc material for a glTF material.
fn mesh_material_of(material: &Material, texture_ids: &[U32Id<BMeshTexture>]) -> MeshMaterial {
    let pbr = material.pbr_metallic_roughness();
    let base = pbr.base_color_factor().map(f64::from);
    let emissive = material.emissive_factor().map(f64::from);

    let texture_ref = |info: Info| MeshTextureRef {
        texture_id: texture_ids[info.texture().index()],
        uv_stream_id: U32Id::from_u32(info.tex_coord()),
    };

    let normal_ref = |info: &NormalTexture| MeshTextureRef {
        texture_id: texture_ids[info.texture().index()],
        uv_stream_id: U32Id::from_u32(info.tex_coord()),
    };

    let occlusion_ref = |info: &OcclusionTexture| MeshTextureRef {
        texture_id: texture_ids[info.texture().index()],
        uv_stream_id: U32Id::from_u32(info.tex_coord()),
    };

    let normal_texture = material.normal_texture();
    let occlusion_texture = material.occlusion_texture();
    let transmission = material.transmission();

    MeshMaterial {
        name: material.name().unwrap_or_default().to_owned(),
        base_color_factor: TyLinSrgbaF64::new(base[0], base[1], base[2], base[3]),
        base_color_texture: pbr.base_color_texture().map(texture_ref),
        metallic_factor: f64::from(pbr.metallic_factor()),
        roughness_factor: f64::from(pbr.roughness_factor()),
        metallic_roughness_texture: pbr.metallic_roughness_texture().map(texture_ref),
        normal_texture: normal_texture.as_ref().map(normal_ref),
        normal_scale: normal_texture
            .as_ref()
            .map_or(1.0, |info| f64::from(info.scale())),
        occlusion_texture: occlusion_texture.as_ref().map(occlusion_ref),
        occlusion_strength: occlusion_texture
            .as_ref()
            .map_or(1.0, |info| f64::from(info.strength())),
        emissive_factor: TyLinSrgbF64::new(emissive[0], emissive[1], emissive[2]),
        emissive_texture: material.emissive_texture().map(texture_ref),
        emissive_strength: material.emissive_strength().map_or(1.0, f64::from),
        alpha_mode: match material.alpha_mode() {
            AlphaMode::Opaque => MeshAlphaMode::Opaque,
            AlphaMode::Mask => MeshAlphaMode::Mask,
            AlphaMode::Blend => MeshAlphaMode::Blend,
        },
        alpha_cutoff: material.alpha_cutoff().map_or(0.5, f64::from),
        double_sided: material.double_sided(),
        ior: material.ior().map_or(1.5, f64::from),
        transmission_factor: transmission.as_ref().map_or(0.0, |transmission| {
            f64::from(transmission.transmission_factor())
        }),
        transmission_texture: transmission
            .as_ref()
            .and_then(|transmission| transmission.transmission_texture())
            .map(texture_ref),
        properties: Vec::new(),
    }
}

/// A primitive as a meshdoc primitive and its ext entry. Errors on a mode
/// of points or lines, or a stream with no buffer data.
fn read_primitive<'a, 's, F>(
    primitive: &'a Primitive<'a>,
    material_ids: &[U32Id<BMeshMaterial>],
    get: F,
) -> Result<(MeshPrimitive, GltfExtPrimitive)>
where
    F: Clone + Fn(Buffer<'a>) -> Option<&'s [u8]>,
{
    let reader = primitive.reader(get.clone());
    let missing = |what: &str| Error::invalid(format!("a primitive's {what} have no buffer data"));

    let positions: Vec<TyVector3F64> = reader
        .read_positions()
        .ok_or_else(|| Error::invalid("a primitive has no positions"))?
        .map(|position| TyVector3F64::from_array(position.map(f64::from)))
        .collect();

    let vertex_count = positions.len();

    let indices: Vec<u32> = match reader.read_indices() {
        Some(indices) => indices.into_u32().collect(),
        None => (0..vertex_count as u32).collect(),
    };

    let triangles = triangles_of(primitive.mode(), &indices)?;

    let mut mesh_primitive = MeshPrimitive::new(positions, triangles)?;

    if let Some(normals) = reader.read_normals() {
        mesh_primitive.set_normals(Some(
            normals
                .map(|normal| TyVector3F64::from_array(normal.map(f64::from)))
                .collect(),
        ))?;
    }

    if let Some(tangents) = reader.read_tangents() {
        mesh_primitive.set_tangents(Some(
            tangents
                .map(|tangent| TyVector4F64::from_array(tangent.map(f64::from)))
                .collect(),
        ))?;
    }

    // Streams are named by set, and the sets are contiguous from 0.
    let mut uv_sets = Vec::new();
    let mut color_set = None;
    let mut joint_sets = Vec::new();
    let mut weight_sets = Vec::new();
    let mut extra_attributes = Vec::new();

    for (semantic, accessor) in primitive.attributes() {
        match semantic {
            Semantic::TexCoords(set) => uv_sets.push(set),

            Semantic::Colors(0) => color_set = Some(0),

            Semantic::Colors(set) => {
                return Err(Error::invalid(format!(
                    "a primitive carries COLOR_{set}, and only COLOR_0 is supported"
                )));
            }

            Semantic::Joints(set) => joint_sets.push(set),

            Semantic::Weights(set) => weight_sets.push(set),

            Semantic::Extras(name) => extra_attributes.push((format!("_{name}"), accessor)),

            Semantic::Positions | Semantic::Normals | Semantic::Tangents => {}
        }
    }

    uv_sets.sort_unstable();
    for (expected, set) in uv_sets.iter().enumerate() {
        if *set as usize != expected {
            return Err(Error::invalid(format!(
                "a primitive's TEXCOORD sets are not contiguous from 0: TEXCOORD_{set} follows {expected} sets"
            )));
        }

        let uvs: Vec<TyVector2F64> = reader
            .read_tex_coords(*set)
            .ok_or_else(|| missing("texture coordinates"))?
            .into_f32()
            .map(|uv| TyVector2F64::new(f64::from(uv[0]), f64::from(uv[1])))
            .collect();

        mesh_primitive.push_uv_stream(uvs)?;
    }

    if let Some(set) = color_set {
        let colors: Vec<TyLinSrgbaF64> = reader
            .read_colors(set)
            .ok_or_else(|| missing("colors"))?
            .into_rgba_f32()
            .map(|color| TyLinSrgbaF64::from(color.map(f64::from)))
            .collect();

        mesh_primitive.set_colors(Some(colors))?;
    }

    for (name, accessor) in extra_attributes {
        let (width, components) = read_attribute(accessor, get.clone())?;

        mesh_primitive.push_vertex_attribute(MeshVertexAttribute {
            name,
            width,
            components,
        })?;
    }

    mesh_primitive.set_material_id(
        primitive
            .material()
            .index()
            .map(|index| material_ids[index]),
    );

    let (extras, vxl) = VxlExtras::split(&value_from_extras(primitive.extras())?)?;

    if !vxl.values.is_empty() {
        return Err(Error::invalid(
            "a primitive carries extras `vxl.values`, which only a material or a mesh has",
        ));
    }

    if let Some(name) = vxl.name {
        mesh_primitive.set_name(name);
    }

    let mut ext_primitive = GltfExtPrimitive {
        extras,
        extensions: extension_map(primitive.extensions()),
        ..Default::default()
    };

    for set in joint_sets {
        let joints = reader
            .read_joints(set)
            .ok_or_else(|| missing("joints"))?
            .into_u16()
            .collect();
        ext_primitive.joints.insert(set, joints);
    }

    for set in weight_sets {
        let weights = reader
            .read_weights(set)
            .ok_or_else(|| missing("weights"))?
            .into_f32()
            .collect();
        ext_primitive.weights.insert(set, weights);
    }

    for (positions, normals, tangents) in reader.read_morph_targets() {
        ext_primitive.targets.push(GltfExtMorphTarget {
            positions: positions.map(Iterator::collect),
            normals: normals.map(Iterator::collect),
            tangents: tangents.map(Iterator::collect),
        });
    }

    Ok((mesh_primitive, ext_primitive))
}

/// The triangle list of `indices` drawn in `mode`: a list as is, a strip or
/// fan unrolled. Points and lines have no triangles and error.
fn triangles_of(mode: Mode, indices: &[u32]) -> Result<Vec<MeshTriangle>> {
    let triangle = |a: u32, b: u32, c: u32| MeshTriangle {
        vertex_ids: [U32Id::from_u32(a), U32Id::from_u32(b), U32Id::from_u32(c)],
    };

    Ok(match mode {
        Mode::Triangles => indices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|corners| triangle(corners[0], corners[1], corners[2]))
            .collect(),

        Mode::TriangleStrip => indices
            .windows(3)
            .enumerate()
            .map(|(index, corners)| {
                if index % 2 == 0 {
                    triangle(corners[0], corners[1], corners[2])
                } else {
                    triangle(corners[1], corners[0], corners[2])
                }
            })
            .collect(),

        Mode::TriangleFan => indices
            .windows(2)
            .skip(1)
            .map(|corners| triangle(indices[0], corners[0], corners[1]))
            .collect(),

        Mode::Points | Mode::Lines | Mode::LineLoop | Mode::LineStrip => {
            return Err(Error::invalid(format!(
                "a primitive of mode {mode:?} has no triangles"
            )));
        }
    })
}

/// An animation and its keyframes, read through the channels that play its
/// samplers. A sampler no channel plays has no known kind and drops.
fn read_animation<'a, 's, F>(
    animation: &gltf::Animation<'a>,
    node_ids: &[U32Id<BMeshHierarchyNode>],
    get: F,
) -> Result<GltfExtAnimation>
where
    F: Clone + Fn(Buffer<'a>) -> Option<&'s [u8]>,
{
    let mut samplers: BTreeMap<usize, GltfExtAnimationSampler> = BTreeMap::new();
    let mut channels = Vec::new();

    for channel in animation.channels() {
        let sampler = channel.sampler();

        if let Entry::Vacant(entry) = samplers.entry(sampler.index()) {
            entry.insert(read_sampler(&sampler, &channel.reader(get.clone()))?);
        }

        channels.push(GltfExtAnimationChannel {
            sampler: sampler.index() as u32,
            target_node_id: node_ids[channel.target().node().index()],
            extras: value_from_extras(channel.extras())?,
        });
    }

    // Samplers keep their indices where every one is played; a dropped
    // sampler shifts the ones after it, so the channels are renumbered.
    let kept: Vec<usize> = samplers.keys().copied().collect();
    for channel in &mut channels {
        channel.sampler = kept
            .iter()
            .position(|&index| index == channel.sampler as usize)
            .expect("a channel's sampler was read") as u32;
    }

    Ok(GltfExtAnimation {
        name: animation.name().map(str::to_owned),
        channels,
        samplers: samplers.into_values().collect(),
        extras: value_from_extras(animation.extras())?,
        extensions: extension_map(animation.extensions()),
    })
}

/// An animation sampler's keyframes through `reader`, the reader of a
/// channel that plays it.
fn read_sampler<'a, 's, F>(
    sampler: &gltf::animation::Sampler<'a>,
    reader: &AnimationReader<'a, 's, F>,
) -> Result<GltfExtAnimationSampler>
where
    F: Clone + Fn(Buffer<'a>) -> Option<&'s [u8]>,
{
    let missing =
        |what: &str| Error::invalid(format!("an animation sampler's {what} have no buffer data"));

    let input = reader
        .read_inputs()
        .ok_or_else(|| missing("inputs"))?
        .collect();

    let output = match reader.read_outputs().ok_or_else(|| missing("outputs"))? {
        ReadOutputs::Translations(values) => GltfExtAnimationOutput::Translations(values.collect()),

        ReadOutputs::Rotations(values) => {
            GltfExtAnimationOutput::Rotations(values.into_f32().collect())
        }

        ReadOutputs::Scales(values) => GltfExtAnimationOutput::Scales(values.collect()),

        ReadOutputs::MorphTargetWeights(values) => {
            GltfExtAnimationOutput::MorphTargetWeights(values.into_f32().collect())
        }
    };

    Ok(GltfExtAnimationSampler {
        interpolation: match sampler.interpolation() {
            Interpolation::Linear => "LINEAR",
            Interpolation::Step => "STEP",
            Interpolation::CubicSpline => "CUBICSPLINE",
        }
        .to_owned(),
        input,
        output,
        extras: value_from_extras(sampler.extras())?,
    })
}

/// The bytes of every buffer, in buffer order: the blob for the buffer with
/// no URI, a decoded data URI, or the loose file at a relative URI.
fn resolve_buffers<D: DecodeBase64>(
    dependencies: &D,
    file: &GltfFile,
    document: &Document,
) -> Result<Vec<Vec<u8>>> {
    document
        .buffers()
        .map(|buffer| {
            let bytes = match buffer.source() {
                BufferSource::Bin => file.blob.clone().ok_or_else(|| {
                    Error::invalid("a buffer names no URI and the document has no binary chunk")
                })?,

                BufferSource::Uri(uri) => resolve_uri(dependencies, file, uri)?,
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
fn resolve_uri<D: DecodeBase64>(dependencies: &D, file: &GltfFile, uri: &str) -> Result<Vec<u8>> {
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

/// `uri` with its percent-encoded bytes decoded, the file name it points
/// to, or `None` when a `%` is not followed by two hex digits or the result
/// is not UTF-8.
fn percent_decode(uri: &str) -> Option<String> {
    let bytes = uri.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = uri.get(index + 1..index + 3)?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(decoded).ok()
}

/// The media type whose signature `bytes` start with, or `None` for neither.
fn media_type_from_bytes(bytes: &[u8]) -> Option<MeshImageMediaType> {
    [MeshImageMediaType::Png, MeshImageMediaType::Jpeg]
        .into_iter()
        .find(|media_type| media_type.matches(bytes))
}

/// A node's transform: its TRS, or its matrix decomposed.
fn transform_from_gltf(node: &Node) -> TyTransformF64 {
    let (translation, rotation, scale) = node.transform().decomposed();

    TyTransformF64 {
        position: TyVector3F64::from_array(translation.map(f64::from)),
        rotation: TyQuaternionF64::from_xyzw(
            f64::from(rotation[0]),
            f64::from(rotation[1]),
            f64::from(rotation[2]),
            f64::from(rotation[3]),
        ),
        scale: TyVector3F64::from_array(scale.map(f64::from)),
    }
}

/// The property value a `vxl.values` entry named `name` holds. A texture
/// entry holds a texture index into `texture_ids`, and a file entry a
/// relative URI that `file_id_for_uri` resolves. Errors on a shape no
/// property value has.
fn property_value_from_json(
    name: &str,
    value: &Value,
    texture_ids: &[U32Id<BMeshTexture>],
    file_id_for_uri: &impl Fn(&str) -> Result<U32Id<BMeshFile>>,
) -> Result<MeshPropertyValue> {
    let malformed = |what: &str| {
        Error::invalid(format!(
            "extras `vxl.values.{name}` {what}, which no property value holds"
        ))
    };

    Ok(match value {
        Value::Bool(value) => MeshPropertyValue::Bool(*value),

        Value::Number(number) => match number.as_i64() {
            Some(value) => MeshPropertyValue::Int(value),

            None => MeshPropertyValue::Float(
                number
                    .as_f64()
                    .ok_or_else(|| malformed("is a number outside f64"))?,
            ),
        },

        Value::String(value) => MeshPropertyValue::Text(value.clone()),

        Value::Array(entries) => list_from_json(entries).ok_or_else(|| malformed("mixes kinds"))?,

        Value::Object(object) => {
            if let Some(uri) = object.get("uri") {
                let uri = uri
                    .as_str()
                    .ok_or_else(|| malformed("has a `uri` that is not a string"))?;
                MeshPropertyValue::File(file_id_for_uri(uri)?)
            } else if let Some(index) = object.get("index") {
                let index = index
                    .as_u64()
                    .and_then(|index| usize::try_from(index).ok())
                    .ok_or_else(|| malformed("has an `index` that is not a texture index"))?;
                let texture_id = *texture_ids
                    .get(index)
                    .ok_or_else(|| malformed("names a texture index past the textures"))?;
                let uv_stream = match object.get("texCoord") {
                    None => 0,

                    Some(set) => set
                        .as_u64()
                        .and_then(|set| u32::try_from(set).ok())
                        .ok_or_else(|| malformed("has a `texCoord` that is not a set"))?,
                };
                MeshPropertyValue::Texture(MeshTextureRef {
                    texture_id,
                    uv_stream_id: U32Id::from_u32(uv_stream),
                })
            } else {
                return Err(malformed("is an object with neither `uri` nor `index`"));
            }
        }

        Value::Null => return Err(malformed("is null")),
    })
}

/// The list value of `entries`, all of one kind, or `None` when they mix.
/// An empty list, or a list of empty rows, reads as float.
fn list_from_json(entries: &[Value]) -> Option<MeshPropertyValue> {
    if entries.is_empty() {
        return Some(MeshPropertyValue::Floats(vec![]));
    }

    let rows: Option<Vec<&[Value]>> = entries
        .iter()
        .map(|entry| entry.as_array().map(Vec::as_slice))
        .collect();

    let Some(rows) = rows else {
        if let Some(values) = leaves(entries, Value::as_bool) {
            return Some(MeshPropertyValue::Bools(values));
        }

        if let Some(values) = leaves(entries, text) {
            return Some(MeshPropertyValue::Texts(values));
        }

        if let Some(values) = leaves(entries, Value::as_i64) {
            return Some(MeshPropertyValue::Ints(values));
        }

        return leaves(entries, Value::as_f64).map(MeshPropertyValue::Floats);
    };

    if rows.iter().all(|row| row.is_empty()) {
        return Some(MeshPropertyValue::FloatRows(vec![vec![]; rows.len()]));
    }

    if let Some(rows) = rows_of(&rows, Value::as_bool) {
        return Some(MeshPropertyValue::BoolRows(rows));
    }

    if let Some(rows) = rows_of(&rows, text) {
        return Some(MeshPropertyValue::TextRows(rows));
    }

    if let Some(rows) = rows_of(&rows, Value::as_i64) {
        return Some(MeshPropertyValue::IntRows(rows));
    }

    rows_of(&rows, Value::as_f64).map(MeshPropertyValue::FloatRows)
}

/// Every entry of `values` read by `leaf`, or `None` when one is another
/// kind.
fn leaves<T>(values: &[Value], leaf: impl Fn(&Value) -> Option<T>) -> Option<Vec<T>> {
    values.iter().map(leaf).collect()
}

/// Every row of `rows` read by `leaf`, or `None` when a leaf is another kind.
fn rows_of<T>(rows: &[&[Value]], leaf: impl Fn(&Value) -> Option<T>) -> Option<Vec<Vec<T>>> {
    rows.iter().map(|row| leaves(row, &leaf)).collect()
}

/// A string leaf, owned.
fn text(value: &Value) -> Option<String> {
    value.as_str().map(str::to_owned)
}

/// A further vertex attribute's components with its width: the accessor's
/// elements flattened. Unsigned 8- and 16-bit integers keep their type;
/// floats, the signed and 32-bit integers, and integers the accessor
/// normalizes read as floats.
fn read_attribute<'a, 's, F>(
    accessor: Accessor<'a>,
    get: F,
) -> Result<(usize, MeshAttributeComponents)>
where
    F: Clone + Fn(Buffer<'a>) -> Option<&'s [u8]>,
{
    let width = match accessor.dimensions() {
        Dimensions::Scalar => 1,
        Dimensions::Vec2 => 2,
        Dimensions::Vec3 => 3,
        Dimensions::Vec4 | Dimensions::Mat2 => 4,
        Dimensions::Mat3 => 9,
        Dimensions::Mat4 => 16,
    };

    let normalized = accessor.normalized();
    let dimensions = accessor.dimensions();

    let components = match accessor.data_type() {
        DataType::F32 => MeshAttributeComponents::F64(
            read_elements::<f32, _>(accessor, dimensions, get)?
                .into_iter()
                .map(f64::from)
                .collect(),
        ),

        DataType::U8 if !normalized => {
            MeshAttributeComponents::U8(read_elements::<u8, _>(accessor, dimensions, get)?)
        }

        DataType::U8 => MeshAttributeComponents::F64(
            read_elements::<u8, _>(accessor, dimensions, get)?
                .into_iter()
                .map(|value| normalize(f64::from(value), f64::from(u8::MAX)))
                .collect(),
        ),

        DataType::I8 => MeshAttributeComponents::F64(
            read_elements::<i8, _>(accessor, dimensions, get)?
                .into_iter()
                .map(|value| {
                    if normalized {
                        normalize(f64::from(value), f64::from(i8::MAX))
                    } else {
                        f64::from(value)
                    }
                })
                .collect(),
        ),

        DataType::U16 if !normalized => {
            MeshAttributeComponents::U16(read_elements::<u16, _>(accessor, dimensions, get)?)
        }

        DataType::U16 => MeshAttributeComponents::F64(
            read_elements::<u16, _>(accessor, dimensions, get)?
                .into_iter()
                .map(|value| normalize(f64::from(value), f64::from(u16::MAX)))
                .collect(),
        ),

        DataType::I16 => MeshAttributeComponents::F64(
            read_elements::<i16, _>(accessor, dimensions, get)?
                .into_iter()
                .map(|value| {
                    if normalized {
                        normalize(f64::from(value), f64::from(i16::MAX))
                    } else {
                        f64::from(value)
                    }
                })
                .collect(),
        ),

        DataType::U32 => MeshAttributeComponents::F64(
            read_elements::<u32, _>(accessor, dimensions, get)?
                .into_iter()
                .map(f64::from)
                .collect(),
        ),
    };

    Ok((width, components))
}

/// The accessor's elements of `T` components, flattened, iterated at the
/// element width `dimensions` gives.
fn read_elements<'a, 's, T, F>(
    accessor: Accessor<'a>,
    dimensions: Dimensions,
    get: F,
) -> Result<Vec<T>>
where
    T: Item + Copy,
    F: Clone + Fn(Buffer<'a>) -> Option<&'s [u8]>,
{
    let missing = || Error::invalid("a vertex attribute accessor has no buffer data");

    Ok(match dimensions {
        Dimensions::Scalar => Iter::<T>::new(accessor, get).ok_or_else(missing)?.collect(),

        Dimensions::Vec2 => Iter::<[T; 2]>::new(accessor, get)
            .ok_or_else(missing)?
            .flatten()
            .collect(),

        Dimensions::Vec3 => Iter::<[T; 3]>::new(accessor, get)
            .ok_or_else(missing)?
            .flatten()
            .collect(),

        Dimensions::Vec4 => Iter::<[T; 4]>::new(accessor, get)
            .ok_or_else(missing)?
            .flatten()
            .collect(),

        Dimensions::Mat2 => Iter::<[[T; 2]; 2]>::new(accessor, get)
            .ok_or_else(missing)?
            .flatten()
            .flatten()
            .collect(),

        Dimensions::Mat3 => Iter::<[[T; 3]; 3]>::new(accessor, get)
            .ok_or_else(missing)?
            .flatten()
            .flatten()
            .collect(),

        Dimensions::Mat4 => Iter::<[[T; 4]; 4]>::new(accessor, get)
            .ok_or_else(missing)?
            .flatten()
            .flatten()
            .collect(),
    })
}

/// `value` divided by `max`, clamped at `-1` for a signed type as the spec
/// says.
fn normalize(value: f64, max: f64) -> f64 {
    (value / max).max(-1.0)
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{DependenciesImpl, GltfFile, from_gltf_file, from_gltf_file::percent_decode};
    use gltf::json::Root;

    #[test]
    fn decodes_escapes_and_rejects_a_short_one() {
        assert_eq!(percent_decode("a%20b.png").as_deref(), Some("a b.png"));
        assert_eq!(percent_decode("plain.png").as_deref(), Some("plain.png"));
        assert_eq!(percent_decode("bad%2"), None);
    }

    #[test]
    fn rejects_a_root_that_fails_validation() {
        // A node referencing a mesh that does not exist.
        let root: Root = serde_json::from_str(
            r#"{ "asset": { "version": "2.0" }, "nodes": [ { "mesh": 3 } ] }"#,
        )
        .unwrap();

        let file = GltfFile {
            root,
            ..Default::default()
        };

        assert!(from_gltf_file(&DependenciesImpl, &file).is_err());
    }

    #[test]
    fn an_empty_document_loads_empty() {
        let root: Root = serde_json::from_str(r#"{ "asset": { "version": "2.0" } }"#).unwrap();

        let main = from_gltf_file(
            &DependenciesImpl,
            &GltfFile {
                root,
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(main.object_count(), 0);
        assert_eq!(main.ext().asset.version, "2.0");
    }
}
