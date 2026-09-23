use crate::{
    EncodeBase64, Error, GltfBlob, GltfExtAnimationOutput, GltfExtImageSource, GltfExtSampler,
    GltfFile, GltfImageStorage, GltfMeshMain, GltfWriteOptions, Result, VxlExtras,
    extras_from_value, f32_bytes, property_value_to_json, push_accessor, transform_to_gltf,
};
use branded_id::U32Id;
use gltf::json::{
    Accessor, Animation, Asset, Buffer, Camera, Image, Index, Material, Mesh, Node, Root, Scene,
    Skin, Texture,
    accessor::{ComponentType, Type},
    animation::{Channel, Interpolation, Property, Sampler as AnimationSampler, Target},
    buffer::Target as BufferTarget,
    extensions,
    image::MimeType,
    material::{
        AlphaCutoff, AlphaMode, EmissiveFactor, NormalTexture, OcclusionTexture,
        PbrBaseColorFactor, PbrMetallicRoughness, StrengthFactor,
    },
    mesh::{MorphTarget, Primitive, Semantic},
    scene::UnitQuaternion,
    texture::{Info, MagFilter, MinFilter, Sampler, WrappingMode},
    validation::Checked,
};
use meshdoc::{
    BMeshFile, BMeshHierarchyNode, BMeshImage, BMeshMaterial, BMeshObject, BMeshTexture,
    MeshAlphaMode, MeshAttributeComponents, MeshImage, MeshImageSource, MeshMagFilter,
    MeshMinFilter, MeshPrimitive, MeshProperty, MeshState, MeshTextureRef, MeshWrap,
    material::{EMISSIVE_STRENGTH_DEFAULT, IOR_DEFAULT, TRANSMISSION_DEFAULT},
};
use serde_json::{Map, Value, json};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::Path,
};

/// The extension the writer declares for a material's emissive strength.
const EMISSIVE_STRENGTH_EXTENSION: &str = "KHR_materials_emissive_strength";

/// The extension the writer declares for a material's index of refraction.
const IOR_EXTENSION: &str = "KHR_materials_ior";

/// The extension the writer declares for a material's transmission.
const TRANSMISSION_EXTENSION: &str = "KHR_materials_transmission";

/// Writes a [`GltfMeshMain`] to a glTF [`GltfFile`], the inverse of
/// [`from_gltf_file`](crate::from_gltf_file). A loaded file writes back exactly
/// through its ext. A state [`to_gltf_mesh_main`](crate::to_gltf_mesh_main)
/// gave its ext writes as a file synthesized from the document. Every entity
/// writes at its listing index. The geometry packs into one buffer, one view
/// per stream. Positions, normals, tangents, and node transforms copy straight
/// because meshdoc shares glTF's frame. Each root in no scene joins the default
/// scene, so a node retained after the load is reachable. The document's files
/// land as the loose files under their names. A material's and an object's
/// properties land in its `extras.vxl.values`, and a primitive's name in its
/// `extras.vxl.name`. A primitive with no vertices leaves the file because
/// glTF forbids an empty accessor. `options` picks where the images go.
/// `dependencies` encodes the data URIs of the images that go there.
///
/// Errors if:
///
/// 1. the ext is out of step with the document: an entity has no entry, or an
///    entry references a camera, skin, or sampler that does not exist
/// 2. a material, object, or primitive has `vxl` entries to write and ext
///    `extras` that are not a JSON object
pub fn to_gltf_file<D: EncodeBase64>(
    dependencies: &D,
    main: &GltfMeshMain,
    options: &GltfWriteOptions,
) -> Result<GltfFile> {
    let ext = main.ext();

    let mut root = Root {
        asset: Asset {
            copyright: ext.asset.copyright.clone(),
            extensions: None,
            extras: extras_from_value(&ext.asset.extras),
            generator: ext.asset.generator.clone(),
            min_version: ext.asset.min_version.clone(),
            version: ext.asset.version.clone(),
        },
        extensions: (!ext.extensions.is_empty()).then(|| extensions::root::Root {
            others: ext.extensions.clone(),
        }),
        extras: extras_from_value(&ext.extras),
        extensions_required: ext.extensions_required.clone(),
        ..Default::default()
    };

    let mut blob = GltfBlob::default();
    let mut loose_files: BTreeMap<String, Vec<u8>> = main
        .iter_files()
        .map(|(_, file)| (file.name.clone(), file.bytes.clone()))
        .collect();
    let mut used_extensions: Vec<String> = ext.extensions_used.clone();

    // Every entity writes at its listing index, so a reference is the index
    // of the referenced entity.
    let image_indices: HashMap<U32Id<BMeshImage>, u32> =
        indices(main.iter_images().map(|(id, _)| id));
    let texture_indices: HashMap<U32Id<BMeshTexture>, u32> =
        indices(main.iter_textures().map(|(id, _)| id));
    let material_indices: HashMap<U32Id<BMeshMaterial>, u32> =
        indices(main.iter_materials().map(|(id, _)| id));
    let object_indices: HashMap<U32Id<BMeshObject>, u32> =
        indices(main.iter_objects().map(|(id, _)| id));
    let node_indices: HashMap<U32Id<BMeshHierarchyNode>, u32> =
        indices(main.iter_hierarchy_nodes().map(|(id, _)| id));

    let mut used_names: HashSet<String> = loose_files.keys().cloned().collect();
    for (index, (image_id, image)) in main.iter_images().enumerate() {
        let entry = ext.images.get(&image_id).ok_or_else(|| {
            Error::invalid(format!(
                "gltf ext has no image entry for image {}",
                image_id.to_u32()
            ))
        })?;

        let bytes = main
            .image_bytes(image_id)
            .expect("an iterated image is one of the state's");

        let (buffer_view, uri) = match placement(options.images, image, entry.source) {
            Placement::BufferView => (Some(blob.push_view(bytes, None)), None),
            Placement::DataUri => (
                None,
                Some(format!(
                    "data:{};base64,{}",
                    image.media_type,
                    dependencies.encode_base64(bytes)
                )),
            ),
            Placement::File(file_id) => (
                None,
                Some(
                    main.file(file_id)
                        .expect("an image reads a live file in a valid state")
                        .name
                        .clone(),
                ),
            ),
            Placement::Loose => {
                let name = loose_name(image, index, &mut used_names);
                loose_files.insert(name.clone(), bytes.to_vec());
                (None, Some(name))
            }
        };

        root.images.push(Image {
            buffer_view,
            mime_type: Some(MimeType(image.media_type.as_str().to_owned())),
            name: (!image.name.is_empty()).then(|| image.name.clone()),
            uri,
            extensions: (!entry.extensions.is_empty()).then(|| extensions::image::Image {
                others: entry.extensions.clone(),
            }),
            extras: extras_from_value(&entry.extras),
        });
    }

    for (texture_id, texture) in main.iter_textures() {
        let entry = ext.textures.get(&texture_id).ok_or_else(|| {
            Error::invalid(format!(
                "gltf ext has no texture entry for texture {}",
                texture_id.to_u32()
            ))
        })?;

        // A texture that referenced a sampler, or that filters or wraps away
        // from the defaults, writes its own sampler.
        let default_sampler = texture.mag_filter.is_none()
            && texture.min_filter.is_none()
            && texture.wrap_s == MeshWrap::Repeat
            && texture.wrap_t == MeshWrap::Repeat;

        let sampler = match &entry.sampler {
            Some(sampler) => Some(sampler.clone()),
            None if default_sampler => None,
            None => Some(GltfExtSampler::default()),
        }
        .map(|sampler| {
            root.push(Sampler {
                mag_filter: texture.mag_filter.map(|filter| {
                    Checked::Valid(match filter {
                        MeshMagFilter::Nearest => MagFilter::Nearest,
                        MeshMagFilter::Linear => MagFilter::Linear,
                    })
                }),
                min_filter: texture.min_filter.map(|filter| {
                    Checked::Valid(match filter {
                        MeshMinFilter::Nearest => MinFilter::Nearest,
                        MeshMinFilter::Linear => MinFilter::Linear,
                        MeshMinFilter::NearestMipmapNearest => MinFilter::NearestMipmapNearest,
                        MeshMinFilter::LinearMipmapNearest => MinFilter::LinearMipmapNearest,
                        MeshMinFilter::NearestMipmapLinear => MinFilter::NearestMipmapLinear,
                        MeshMinFilter::LinearMipmapLinear => MinFilter::LinearMipmapLinear,
                    })
                }),
                name: sampler.name.clone(),
                wrap_s: Checked::Valid(wrapping_mode(texture.wrap_s)),
                wrap_t: Checked::Valid(wrapping_mode(texture.wrap_t)),
                extensions: (!sampler.extensions.is_empty()).then(|| {
                    extensions::texture::Sampler {
                        others: sampler.extensions.clone(),
                    }
                }),
                extras: extras_from_value(&sampler.extras),
            })
        });

        root.textures.push(Texture {
            name: entry.name.clone(),
            sampler,
            source: Index::new(image_indices[&texture.image_id]),
            extensions: (!entry.extensions.is_empty()).then(|| extensions::texture::Texture {
                others: entry.extensions.clone(),
            }),
            extras: extras_from_value(&entry.extras),
        });
    }

    for (material_id, material) in main.iter_materials() {
        let entry = ext.materials.get(&material_id).ok_or_else(|| {
            Error::invalid(format!(
                "gltf ext has no material entry for material {}",
                material_id.to_u32()
            ))
        })?;

        let texture_ref = |texture_ref: &MeshTextureRef| Info {
            index: Index::new(texture_indices[&texture_ref.texture_id]),
            tex_coord: texture_ref.uv_stream_id.to_u32(),
            extensions: None,
            extras: Default::default(),
        };

        let mut material_extensions = extensions::material::Material {
            others: entry.extensions.clone(),
            ..Default::default()
        };

        if material.emissive_strength != EMISSIVE_STRENGTH_DEFAULT {
            material_extensions.emissive_strength = Some(extensions::material::EmissiveStrength {
                emissive_strength: extensions::material::EmissiveStrengthFactor(
                    material.emissive_strength as f32,
                ),
            });
            declare(&mut used_extensions, EMISSIVE_STRENGTH_EXTENSION);
        }

        if material.ior != IOR_DEFAULT {
            material_extensions.ior = Some(extensions::material::Ior {
                ior: extensions::material::IndexOfRefraction(material.ior as f32),
                extras: Default::default(),
            });
            declare(&mut used_extensions, IOR_EXTENSION);
        }

        if material.transmission_factor != TRANSMISSION_DEFAULT
            || material.transmission_texture.is_some()
        {
            material_extensions.transmission = Some(extensions::material::Transmission {
                transmission_factor: extensions::material::TransmissionFactor(
                    material.transmission_factor as f32,
                ),
                transmission_texture: material.transmission_texture.as_ref().map(texture_ref),
                extras: Default::default(),
            });
            declare(&mut used_extensions, TRANSMISSION_EXTENSION);
        }

        let has_extensions = material_extensions.emissive_strength.is_some()
            || material_extensions.ior.is_some()
            || material_extensions.transmission.is_some()
            || !material_extensions.others.is_empty();

        let base = <[f64; 4]>::from(material.base_color_factor).map(|value| value as f32);
        let emissive = <[f64; 3]>::from(material.emissive_factor).map(|value| value as f32);

        root.materials.push(Material {
            alpha_cutoff: (material.alpha_mode == MeshAlphaMode::Mask)
                .then_some(AlphaCutoff(material.alpha_cutoff as f32)),
            alpha_mode: Checked::Valid(match material.alpha_mode {
                MeshAlphaMode::Opaque => AlphaMode::Opaque,
                MeshAlphaMode::Mask => AlphaMode::Mask,
                MeshAlphaMode::Blend => AlphaMode::Blend,
            }),
            double_sided: material.double_sided,
            name: (!material.name.is_empty()).then(|| material.name.clone()),
            pbr_metallic_roughness: PbrMetallicRoughness {
                base_color_factor: PbrBaseColorFactor(base),
                base_color_texture: material.base_color_texture.as_ref().map(texture_ref),
                metallic_factor: StrengthFactor(material.metallic_factor as f32),
                roughness_factor: StrengthFactor(material.roughness_factor as f32),
                metallic_roughness_texture: material
                    .metallic_roughness_texture
                    .as_ref()
                    .map(texture_ref),
                extensions: None,
                extras: Default::default(),
            },
            normal_texture: material
                .normal_texture
                .as_ref()
                .map(|texture_ref| NormalTexture {
                    index: Index::new(texture_indices[&texture_ref.texture_id]),
                    scale: material.normal_scale as f32,
                    tex_coord: texture_ref.uv_stream_id.to_u32(),
                    extensions: None,
                    extras: Default::default(),
                }),
            occlusion_texture: material.occlusion_texture.as_ref().map(|texture_ref| {
                OcclusionTexture {
                    index: Index::new(texture_indices[&texture_ref.texture_id]),
                    strength: StrengthFactor(material.occlusion_strength as f32),
                    tex_coord: texture_ref.uv_stream_id.to_u32(),
                    extensions: None,
                    extras: Default::default(),
                }
            }),
            emissive_texture: material.emissive_texture.as_ref().map(texture_ref),
            emissive_factor: EmissiveFactor(emissive),
            extensions: has_extensions.then_some(material_extensions),
            extras: extras_from_value(&vxl_extras(
                &material.properties,
                None,
                &entry.extras,
                &texture_indices,
                main.state(),
            )?),
        });
    }

    for (object_id, object) in main.iter_objects() {
        let entry = ext.meshes.get(&object_id).ok_or_else(|| {
            Error::invalid(format!(
                "gltf ext has no mesh entry for object {}",
                object_id.to_u32()
            ))
        })?;

        let mut primitives = Vec::new();

        for (primitive_id, primitive) in object.iter_primitives() {
            let primitive_entry = entry.primitives.get(&primitive_id).ok_or_else(|| {
                Error::invalid(format!(
                    "gltf ext has no primitive entry for object {} primitive {}",
                    object_id.to_u32(),
                    primitive_id.to_u32()
                ))
            })?;

            if primitive.vertex_count() == 0 {
                continue;
            }

            let mut attributes = BTreeMap::new();

            attributes.insert(
                Checked::Valid(Semantic::Positions),
                push_positions(&mut root, &mut blob, primitive),
            );

            if let Some(normals) = primitive.normals() {
                let data = f32_bytes(
                    normals
                        .iter()
                        .flat_map(|normal| normal.to_array().map(|value| value as f32)),
                );
                attributes.insert(
                    Checked::Valid(Semantic::Normals),
                    push_f32(&mut root, &mut blob, &data, normals.len(), Type::Vec3),
                );
            }

            if let Some(tangents) = primitive.tangents() {
                let data = f32_bytes(
                    tangents
                        .iter()
                        .flat_map(|tangent| tangent.to_array().map(|value| value as f32)),
                );
                attributes.insert(
                    Checked::Valid(Semantic::Tangents),
                    push_f32(&mut root, &mut blob, &data, tangents.len(), Type::Vec4),
                );
            }

            for (stream_id, uvs) in primitive.iter_uv_streams() {
                let data = f32_bytes(uvs.iter().flat_map(|uv| [uv.x as f32, uv.y as f32]));
                attributes.insert(
                    Checked::Valid(Semantic::TexCoords(stream_id.to_u32())),
                    push_f32(&mut root, &mut blob, &data, uvs.len(), Type::Vec2),
                );
            }

            if let Some(colors) = primitive.colors() {
                let data = f32_bytes(
                    colors
                        .iter()
                        .flat_map(|color| <[f64; 4]>::from(*color).map(|value| value as f32)),
                );
                attributes.insert(
                    Checked::Valid(Semantic::Colors(0)),
                    push_f32(&mut root, &mut blob, &data, colors.len(), Type::Vec4),
                );
            }

            for (_, attribute) in primitive.iter_vertex_attributes() {
                let type_ = match attribute.width {
                    1 => Type::Scalar,
                    2 => Type::Vec2,
                    3 => Type::Vec3,
                    4 => Type::Vec4,
                    9 => Type::Mat3,
                    16 => Type::Mat4,
                    width => {
                        return Err(Error::invalid(format!(
                            "vertex attribute \"{}\" is {width} wide, which glTF has no type for",
                            attribute.name
                        )));
                    }
                };
                let (data, component_type) = match &attribute.components {
                    MeshAttributeComponents::F64(values) => (
                        f32_bytes(values.iter().map(|value| *value as f32)),
                        ComponentType::F32,
                    ),
                    MeshAttributeComponents::U8(values) => (values.clone(), ComponentType::U8),
                    MeshAttributeComponents::U16(values) => (
                        values
                            .iter()
                            .flat_map(|value| value.to_le_bytes())
                            .collect(),
                        ComponentType::U16,
                    ),
                };
                // The semantic prints the underscore glTF requires of a custom
                // attribute, so the name sheds it here.
                let Some(semantic) = attribute.name.strip_prefix('_') else {
                    return Err(Error::invalid(format!(
                        "vertex attribute \"{}\" has no leading underscore, which glTF \
                         requires of a custom attribute",
                        attribute.name
                    )));
                };

                attributes.insert(
                    Checked::Valid(Semantic::Extras(semantic.to_owned())),
                    push_accessor(
                        &mut root,
                        &mut blob,
                        &data,
                        primitive.vertex_count(),
                        component_type,
                        type_,
                        false,
                        Some(BufferTarget::ArrayBuffer),
                        None,
                        None,
                    ),
                );
            }

            for (set, joints) in &primitive_entry.joints {
                let data: Vec<u8> = joints
                    .iter()
                    .flat_map(|joint| joint.iter().flat_map(|value| value.to_le_bytes()))
                    .collect();
                attributes.insert(
                    Checked::Valid(Semantic::Joints(*set)),
                    push_accessor(
                        &mut root,
                        &mut blob,
                        &data,
                        joints.len(),
                        ComponentType::U16,
                        Type::Vec4,
                        false,
                        Some(BufferTarget::ArrayBuffer),
                        None,
                        None,
                    ),
                );
            }

            for (set, weights) in &primitive_entry.weights {
                let data = f32_bytes(weights.iter().flatten().copied());
                attributes.insert(
                    Checked::Valid(Semantic::Weights(*set)),
                    push_f32(&mut root, &mut blob, &data, weights.len(), Type::Vec4),
                );
            }

            let indices: Vec<u8> = primitive
                .triangles()
                .iter()
                .flat_map(|triangle| triangle.vertex_ids)
                .flat_map(|vertex_id| vertex_id.to_u32().to_le_bytes())
                .collect();

            let index_accessor = push_accessor(
                &mut root,
                &mut blob,
                &indices,
                primitive.triangle_count() * 3,
                ComponentType::U32,
                Type::Scalar,
                false,
                Some(BufferTarget::ElementArrayBuffer),
                None,
                None,
            );

            let targets: Vec<MorphTarget> = primitive_entry
                .targets
                .iter()
                .map(|target| MorphTarget {
                    positions: target
                        .positions
                        .as_ref()
                        .map(|values| push_vec3s(&mut root, &mut blob, values)),
                    normals: target
                        .normals
                        .as_ref()
                        .map(|values| push_vec3s(&mut root, &mut blob, values)),
                    tangents: target
                        .tangents
                        .as_ref()
                        .map(|values| push_vec3s(&mut root, &mut blob, values)),
                })
                .collect();

            primitives.push(Primitive {
                attributes,
                extensions: (!primitive_entry.extensions.is_empty()).then(|| {
                    extensions::mesh::Primitive {
                        others: primitive_entry.extensions.clone(),
                    }
                }),
                extras: extras_from_value(&vxl_extras(
                    &[],
                    (!primitive.name().is_empty()).then(|| primitive.name().to_owned()),
                    &primitive_entry.extras,
                    &texture_indices,
                    main.state(),
                )?),
                indices: Some(index_accessor),
                material: primitive
                    .material_id()
                    .map(|material_id| Index::new(material_indices[&material_id])),
                mode: Checked::Valid(gltf::json::mesh::Mode::Triangles),
                targets: (!targets.is_empty()).then_some(targets),
            });
        }

        root.meshes.push(Mesh {
            extensions: (!entry.extensions.is_empty()).then(|| extensions::mesh::Mesh {
                others: entry.extensions.clone(),
            }),
            extras: extras_from_value(&vxl_extras(
                object.properties(),
                None,
                &entry.extras,
                &texture_indices,
                main.state(),
            )?),
            name: (!object.name().is_empty()).then(|| object.name().to_owned()),
            primitives,
            weights: entry.weights.clone(),
        });
    }

    for (node_id, node) in main.iter_hierarchy_nodes() {
        let entry = ext.nodes.get(&node_id).ok_or_else(|| {
            Error::invalid(format!(
                "gltf ext has no node entry for node {}",
                node_id.to_u32()
            ))
        })?;

        if let Some(camera) = entry.camera
            && camera as usize >= ext.cameras.len()
        {
            return Err(Error::invalid(format!(
                "gltf ext node {} names camera {camera}, which the ext does not hold",
                node_id.to_u32()
            )));
        }

        if let Some(skin) = entry.skin
            && skin as usize >= ext.skins.len()
        {
            return Err(Error::invalid(format!(
                "gltf ext node {} names skin {skin}, which the ext does not hold",
                node_id.to_u32()
            )));
        }

        // A node places at most one mesh. A node placing several objects is
        // written as the node and one child per further object.
        let mut mesh_ids = node.child_object_ids.iter();
        let (translation, rotation, scale) = transform_to_gltf(&node.transform);

        root.nodes.push(Node {
            camera: entry.camera.map(Index::new),
            children: (!node.child_node_ids.is_empty()).then(|| {
                node.child_node_ids
                    .iter()
                    .map(|child_id| Index::new(node_indices[child_id]))
                    .collect()
            }),
            extensions: (!entry.extensions.is_empty()).then(|| extensions::scene::Node {
                others: entry.extensions.clone(),
            }),
            extras: extras_from_value(&entry.extras),
            matrix: None,
            mesh: mesh_ids
                .next()
                .map(|object_id| Index::new(object_indices[object_id])),
            name: (!node.name.is_empty()).then(|| node.name.clone()),
            rotation: (rotation != [0.0, 0.0, 0.0, 1.0]).then_some(UnitQuaternion(rotation)),
            scale: (scale != [1.0, 1.0, 1.0]).then_some(scale),
            translation: (translation != [0.0, 0.0, 0.0]).then_some(translation),
            skin: entry.skin.map(Index::new),
            weights: entry.weights.clone(),
        });

        if mesh_ids.len() > 0 {
            return Err(Error::invalid(format!(
                "node {} places {} objects, and a glTF node holds one mesh",
                node_id.to_u32(),
                node.child_object_ids.len()
            )));
        }
    }

    // The scenes as the ext holds them, then every root in no scene joins
    // the default scene.
    let mut scenes: Vec<Scene> = ext
        .scenes
        .iter()
        .map(|scene| Scene {
            extensions: (!scene.extensions.is_empty()).then(|| extensions::scene::Scene {
                others: scene.extensions.clone(),
            }),
            extras: extras_from_value(&scene.extras),
            name: scene.name.clone(),
            nodes: scene
                .node_ids
                .iter()
                .map(|node_id| Index::new(node_indices[node_id]))
                .collect(),
        })
        .collect();

    let placed: HashSet<U32Id<BMeshHierarchyNode>> = ext
        .scenes
        .iter()
        .flat_map(|scene| scene.node_ids.iter().copied())
        .collect();

    let unplaced: Vec<Index<Node>> = main
        .root_hierarchy_node_ids()
        .iter()
        .filter(|root_id| !placed.contains(root_id))
        .map(|root_id| Index::new(node_indices[root_id]))
        .collect();

    let mut default_scene = ext.default_scene;

    if !unplaced.is_empty() {
        let index = match default_scene {
            Some(index) if (index as usize) < scenes.len() => index as usize,
            Some(index) => {
                return Err(Error::invalid(format!(
                    "gltf ext names default scene {index}, which the ext does not hold"
                )));
            }
            None if scenes.is_empty() => {
                scenes.push(Scene {
                    extensions: None,
                    extras: Default::default(),
                    name: None,
                    nodes: Vec::new(),
                });
                default_scene = Some(0);
                0
            }
            None => 0,
        };

        scenes[index].nodes.extend(unplaced);
    }

    root.scenes = scenes;
    root.scene = default_scene.map(Index::new);

    for skin in &ext.skins {
        let inverse_bind_matrices = skin.inverse_bind_matrices.as_ref().map(|matrices| {
            let data = f32_bytes(matrices.iter().flatten().copied());
            push_accessor(
                &mut root,
                &mut blob,
                &data,
                matrices.len(),
                ComponentType::F32,
                Type::Mat4,
                false,
                None,
                None,
                None,
            )
        });

        root.skins.push(Skin {
            extensions: (!skin.extensions.is_empty()).then(|| extensions::skin::Skin {
                others: skin.extensions.clone(),
            }),
            extras: extras_from_value(&skin.extras),
            inverse_bind_matrices,
            joints: skin
                .joint_node_ids
                .iter()
                .map(|node_id| Index::new(node_indices[node_id]))
                .collect(),
            name: skin.name.clone(),
            skeleton: skin
                .skeleton_node_id
                .map(|node_id| Index::new(node_indices[&node_id])),
        });
    }

    for animation in &ext.animations {
        let mut samplers = Vec::new();

        for sampler in &animation.samplers {
            let input = f32_bytes(sampler.input.iter().copied());
            let input = push_accessor(
                &mut root,
                &mut blob,
                &input,
                sampler.input.len(),
                ComponentType::F32,
                Type::Scalar,
                false,
                None,
                min_max(&sampler.input),
                min_max(&sampler.input).map(|_| {
                    json!([sampler
                        .input
                        .iter()
                        .copied()
                        .fold(f32::NEG_INFINITY, f32::max)])
                }),
            );

            let (data, count, type_) = match &sampler.output {
                GltfExtAnimationOutput::Translations(values)
                | GltfExtAnimationOutput::Scales(values) => (
                    f32_bytes(values.iter().flatten().copied()),
                    values.len(),
                    Type::Vec3,
                ),
                GltfExtAnimationOutput::Rotations(values) => (
                    f32_bytes(values.iter().flatten().copied()),
                    values.len(),
                    Type::Vec4,
                ),
                GltfExtAnimationOutput::MorphTargetWeights(values) => (
                    f32_bytes(values.iter().copied()),
                    values.len(),
                    Type::Scalar,
                ),
            };

            let output = push_accessor(
                &mut root,
                &mut blob,
                &data,
                count,
                ComponentType::F32,
                type_,
                false,
                None,
                None,
                None,
            );

            samplers.push(AnimationSampler {
                extensions: None,
                extras: extras_from_value(&sampler.extras),
                input,
                interpolation: Checked::Valid(match sampler.interpolation.as_str() {
                    "STEP" => Interpolation::Step,
                    "CUBICSPLINE" => Interpolation::CubicSpline,
                    _ => Interpolation::Linear,
                }),
                output,
            });
        }

        let channels = animation
            .channels
            .iter()
            .map(|channel| {
                let sampler = animation
                    .samplers
                    .get(channel.sampler as usize)
                    .ok_or_else(|| {
                        Error::invalid(format!(
                            "gltf ext animation channel names sampler {}, which the animation does not hold",
                            channel.sampler
                        ))
                    })?;

                Ok(Channel {
                    sampler: Index::new(channel.sampler),
                    target: Target {
                        extensions: None,
                        extras: Default::default(),
                        node: Index::new(node_indices[&channel.target_node_id]),
                        path: Checked::Valid(match sampler.output {
                            GltfExtAnimationOutput::Translations(_) => Property::Translation,
                            GltfExtAnimationOutput::Rotations(_) => Property::Rotation,
                            GltfExtAnimationOutput::Scales(_) => Property::Scale,
                            GltfExtAnimationOutput::MorphTargetWeights(_) => {
                                Property::MorphTargetWeights
                            }
                        }),
                    },
                    extensions: None,
                    extras: extras_from_value(&channel.extras),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        root.animations.push(Animation {
            extensions: (!animation.extensions.is_empty()).then(|| {
                extensions::animation::Animation {
                    others: animation.extensions.clone(),
                }
            }),
            extras: extras_from_value(&animation.extras),
            channels,
            name: animation.name.clone(),
            samplers,
        });
    }

    root.cameras = ext
        .cameras
        .iter()
        .map(|camera| {
            serde_json::from_value::<Camera>(camera.clone()).map_err(|error| {
                Error::invalid(format!("a gltf ext camera is not a camera: {error}"))
            })
        })
        .collect::<Result<_>>()?;

    root.extensions_used = used_extensions;

    let blob = (!blob.bytes.is_empty()).then(|| {
        root.buffers.push(Buffer {
            byte_length: blob.bytes.len().into(),
            name: None,
            uri: None,
            extensions: None,
            extras: Default::default(),
        });
        root.buffer_views = blob.views;
        blob.bytes
    });

    Ok(GltfFile {
        root,
        blob,
        loose_files,
    })
}

/// Where an image goes, by the write option, its source, and its recorded
/// embedding.
enum Placement {
    /// A buffer view in the one buffer.
    BufferView,

    /// A data URI on the image.
    DataUri,

    /// A reference to one of the document's files, which lands loose under
    /// its name.
    File(U32Id<BMeshFile>),

    /// A loose file of the image's own bytes, named by the image.
    Loose,
}

/// The placement `storage` gives `image`, whose embedded bytes were loaded
/// from `source`.
fn placement(
    storage: GltfImageStorage,
    image: &MeshImage,
    source: Option<GltfExtImageSource>,
) -> Placement {
    match (storage, &image.source, source) {
        (GltfImageStorage::Embedded, _, Some(GltfExtImageSource::DataUri)) => Placement::DataUri,
        (GltfImageStorage::Embedded, _, _) => Placement::BufferView,
        (_, MeshImageSource::File(file_id), _) => Placement::File(*file_id),
        (GltfImageStorage::Loose, MeshImageSource::Bytes(_), _) => Placement::Loose,
        (
            GltfImageStorage::AsLoaded,
            MeshImageSource::Bytes(_),
            Some(GltfExtImageSource::DataUri),
        ) => Placement::DataUri,
        (GltfImageStorage::AsLoaded, MeshImageSource::Bytes(_), _) => Placement::BufferView,
    }
}

/// The relative URI a loose image of its own bytes takes: its name when
/// that is a bare file name, with the media type's extension added when it
/// has none, else `image<index>.<extension>`. A repeat falls back to the
/// indexed name.
fn loose_name(image: &MeshImage, index: usize, used: &mut HashSet<String>) -> String {
    let extension = image.media_type.extension();
    let indexed = format!("image{index}.{extension}");

    let bare_name = !image.name.is_empty()
        && !image.name.contains(['/', '\\'])
        && image.name != "."
        && image.name != "..";

    let preferred = if !bare_name {
        indexed.clone()
    } else if Path::new(&image.name).extension().is_some() {
        image.name.clone()
    } else {
        format!("{}.{extension}", image.name)
    };

    let name = if used.contains(&preferred) {
        indexed
    } else {
        preferred
    };

    used.insert(name.clone());
    name
}

/// The listing index of each id, in `ids` order.
fn indices<Brand>(ids: impl Iterator<Item = U32Id<Brand>>) -> HashMap<U32Id<Brand>, u32> {
    ids.enumerate()
        .map(|(index, id)| (id, index as u32))
        .collect()
}

/// Adds `extension` to the declared extensions once.
fn declare(used: &mut Vec<String>, extension: &str) {
    if !used.iter().any(|name| name == extension) {
        used.push(extension.to_owned());
    }
}

/// The glTF wrapping mode for a meshdoc wrap.
fn wrapping_mode(wrap: MeshWrap) -> WrappingMode {
    match wrap {
        MeshWrap::Repeat => WrappingMode::Repeat,
        MeshWrap::ClampToEdge => WrappingMode::ClampToEdge,
        MeshWrap::MirroredRepeat => WrappingMode::MirroredRepeat,
    }
}

/// An object's `extras`: the ext's, with `properties` as its `vxl.values`
/// and `name` as its `vxl.name` merged in. Errors when there is something
/// to merge and the ext `extras` are not a JSON object.
fn vxl_extras(
    properties: &[MeshProperty],
    name: Option<String>,
    extras: &Option<Value>,
    texture_indices: &HashMap<U32Id<BMeshTexture>, u32>,
    state: &MeshState,
) -> Result<Option<Value>> {
    let values: Map<String, Value> = properties
        .iter()
        .map(|property| {
            (
                property.name.clone(),
                property_value_to_json(&property.value, texture_indices, state),
            )
        })
        .collect();

    VxlExtras { values, name }.merge_into(extras)
}

/// Appends a primitive's positions with the min and max the spec requires.
fn push_positions(
    root: &mut Root,
    blob: &mut GltfBlob,
    primitive: &MeshPrimitive,
) -> Index<Accessor> {
    let positions: Vec<[f32; 3]> = primitive
        .positions()
        .iter()
        .map(|position| position.to_array().map(|value| value as f32))
        .collect();

    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for position in &positions {
        for axis in 0..3 {
            min[axis] = min[axis].min(position[axis]);
            max[axis] = max[axis].max(position[axis]);
        }
    }

    let (min, max) = if positions.is_empty() {
        (None, None)
    } else {
        (Some(json!(min)), Some(json!(max)))
    };

    push_accessor(
        root,
        blob,
        &f32_bytes(positions.iter().flatten().copied()),
        positions.len(),
        ComponentType::F32,
        Type::Vec3,
        false,
        Some(BufferTarget::ArrayBuffer),
        min,
        max,
    )
}

/// Appends a float vertex stream of `count` elements of `type_`.
fn push_f32(
    root: &mut Root,
    blob: &mut GltfBlob,
    data: &[u8],
    count: usize,
    type_: Type,
) -> Index<Accessor> {
    push_accessor(
        root,
        blob,
        data,
        count,
        ComponentType::F32,
        type_,
        false,
        Some(BufferTarget::ArrayBuffer),
        None,
        None,
    )
}

/// Appends a stream of three-float vectors as they are.
fn push_vec3s(root: &mut Root, blob: &mut GltfBlob, values: &[[f32; 3]]) -> Index<Accessor> {
    push_f32(
        root,
        blob,
        &f32_bytes(values.iter().flatten().copied()),
        values.len(),
        Type::Vec3,
    )
}

/// The `min` of a scalar keyframe input, which the spec requires beside its
/// `max`, or `None` for no keyframes.
fn min_max(input: &[f32]) -> Option<Value> {
    (!input.is_empty()).then(|| json!([input.iter().copied().fold(f32::INFINITY, f32::min)]))
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        DependenciesImpl, GltfExtPrimitive, GltfImageStorage, GltfMeshMain, GltfWriteOptions,
        from_gltf_file, snapshot, test_main, to_gltf_file, to_gltf_mesh_main,
    };
    use branded_id::U32Id;
    use meshdoc::{
        MeshHierarchyNode, MeshImageSource, MeshMaterial, MeshPrimitive, MeshProperty,
        MeshPropertyValue, MeshTexture,
    };
    use serde_json::json;

    /// A main written to a file and loaded back.
    fn round_trip(main: &GltfMeshMain, options: &GltfWriteOptions) -> GltfMeshMain {
        let file = to_gltf_file(&DependenciesImpl, main, options).unwrap();

        from_gltf_file(&DependenciesImpl, &file).unwrap()
    }

    /// A synthesized main writes and loads back with its geometry, materials,
    /// textures, and images equal, and a second pass keeps the ext too.
    #[test]
    fn round_trips_through_gltf_file() {
        let main = to_gltf_mesh_main(test_main());
        let options = GltfWriteOptions::default();

        let loaded = round_trip(&main, &options);
        loaded.validate().unwrap();

        let before = snapshot(&main);
        let after = snapshot(&loaded);

        assert_eq!(after.files, before.files);
        assert_eq!(after.images, before.images);
        assert_eq!(after.textures, before.textures);
        assert_eq!(after.materials, before.materials);
        assert_eq!(after.objects, before.objects);
        assert_eq!(after.roots, before.roots);

        // The properties and the primitive name rode the `vxl` extras, which
        // the reader took back out, leaving the ext extras empty.
        let file = to_gltf_file(&DependenciesImpl, &main, &options).unwrap();
        let material_extras = file.root.materials[0].extras.as_ref().unwrap().get();
        assert!(material_extras.contains("\"vxl\":{\"values\":{"));
        assert!(material_extras.contains("\"wear\":{\"uri\":\"values.json\"}"));
        assert!(material_extras.contains("\"heat\":{\"index\":1,\"texCoord\":1}"));
        assert!(
            file.root.meshes[0]
                .extras
                .as_ref()
                .unwrap()
                .get()
                .contains("\"albedo\":[[1.0,0.0,0.0,1.0],[0.0,0.0,1.0,1.0]]")
        );
        assert_eq!(
            file.root.meshes[0].primitives[0]
                .extras
                .as_ref()
                .unwrap()
                .get(),
            r#"{"vxl":{"name":"shell"}}"#
        );
        assert_eq!(after.ext.materials[&U32Id::from_u32(0)].extras, None);
        assert_eq!(after.ext.meshes[&U32Id::from_u32(0)].extras, None);

        // A custom attribute keeps its one underscore.
        let root = serde_json::to_string(&file.root).unwrap();
        assert!(root.contains("\"_CELL\""));
        assert!(!root.contains("\"__CELL\""));

        // The integer attributes kept their accessor types.
        let component_types: Vec<u32> = file
            .root
            .accessors
            .iter()
            .map(|accessor| accessor.component_type.unwrap().0.as_gl_enum())
            .collect();
        assert!(component_types.contains(&5121));
        assert!(component_types.contains(&5123));

        // A loaded file writes back exactly.
        let again = round_trip(&loaded, &options);
        assert_eq!(snapshot(&again), after);
    }

    /// Node transforms survive the write's `f32` narrowing.
    #[test]
    fn node_transforms_round_trip_within_float_precision() {
        let main = to_gltf_mesh_main(test_main());
        let loaded = round_trip(&main, &GltfWriteOptions::default());

        for ((_, before), (_, after)) in main
            .iter_hierarchy_nodes()
            .zip(loaded.iter_hierarchy_nodes())
        {
            assert_eq!(before.name, after.name);
            assert!((before.transform.position - after.transform.position).length() < 1e-6);
            assert!((before.transform.scale - after.transform.scale).length() < 1e-6);
            let dot = before
                .transform
                .rotation
                .dot(after.transform.rotation)
                .abs();
            assert!(dot > 1.0 - 1e-6, "{dot}");
        }
    }

    /// The document's files land beside the document, a file-backed image
    /// references its file, and `Loose` lands the byte-backed image beside
    /// them under its name; every image loads back over a file.
    #[test]
    fn loose_images_write_beside_the_document_and_read_back() {
        let main = to_gltf_mesh_main(test_main());

        // As loaded: the JPEG references its file, the PNG embeds.
        let file = to_gltf_file(&DependenciesImpl, &main, &GltfWriteOptions::default()).unwrap();
        let names: Vec<&String> = file.loose_files.keys().collect();
        assert_eq!(names, ["textures/detail.jpg", "values.json"]);
        assert_eq!(
            file.root.images[1].uri.as_deref(),
            Some("textures/detail.jpg")
        );
        assert!(file.root.images[0].buffer_view.is_some());

        let options = GltfWriteOptions {
            images: GltfImageStorage::Loose,
        };

        let file = to_gltf_file(&DependenciesImpl, &main, &options).unwrap();
        let names: Vec<&String> = file.loose_files.keys().collect();
        assert_eq!(names, ["skin.png", "textures/detail.jpg", "values.json"]);
        assert!(
            file.root
                .buffer_views
                .iter()
                .all(|view| view.byte_length.0 != 11)
        );

        let loaded = from_gltf_file(&DependenciesImpl, &file).unwrap();
        assert_eq!(loaded.file_count(), 3);
        assert!(matches!(
            loaded.image(U32Id::from_u32(0)).unwrap().source,
            MeshImageSource::File(_)
        ));
        assert_eq!(loaded.ext().images[&U32Id::from_u32(0)].source, None);
        assert_eq!(
            loaded.image_bytes(U32Id::from_u32(0)),
            main.image_bytes(U32Id::from_u32(0))
        );

        // As loaded, the images stay loose under their names.
        let again = to_gltf_file(&DependenciesImpl, &loaded, &GltfWriteOptions::default()).unwrap();
        assert_eq!(again.loose_files, file.loose_files);

        // Embedded pulls the bytes into the buffer; the files still land.
        let embedded = to_gltf_file(
            &DependenciesImpl,
            &loaded,
            &GltfWriteOptions {
                images: GltfImageStorage::Embedded,
            },
        )
        .unwrap();
        assert_eq!(embedded.loose_files, file.loose_files);
        assert!(
            embedded
                .root
                .images
                .iter()
                .all(|image| image.buffer_view.is_some() && image.uri.is_none())
        );
    }

    /// An entity retained after the load gets a synthesized entry, and a
    /// new root joins the default scene.
    #[test]
    fn entities_retained_after_the_load_write_with_synthesized_entries() {
        let mut main = round_trip(
            &to_gltf_mesh_main(test_main()),
            &GltfWriteOptions::default(),
        );

        let image_id = U32Id::from_u32(0);
        let texture_id = main.retain_texture(MeshTexture::new(image_id)).unwrap();
        let material_id = main
            .retain_material(MeshMaterial::new("later".to_owned()))
            .unwrap();
        let node_id = main
            .retain_hierarchy_node(MeshHierarchyNode {
                name: "later".to_owned(),
                ..Default::default()
            })
            .unwrap();
        main.push_root_hierarchy_node_id(node_id).unwrap();

        let file = to_gltf_file(&DependenciesImpl, &main, &GltfWriteOptions::default()).unwrap();
        assert_eq!(file.root.textures.len(), 3);
        assert_eq!(file.root.materials.len(), 3);
        assert_eq!(file.root.scenes[0].nodes.len(), 2);

        let loaded = from_gltf_file(&DependenciesImpl, &file).unwrap();
        assert!(loaded.texture(texture_id).is_some());
        assert!(loaded.material(material_id).is_some());
        assert_eq!(loaded.root_hierarchy_node_ids().len(), 2);
    }

    /// A released node leaves the scene, and gc rekeys the entries so the
    /// rebuilt file still writes.
    #[test]
    fn released_entities_leave_the_ext_aligned() {
        let mut main = round_trip(
            &to_gltf_mesh_main(test_main()),
            &GltfWriteOptions::default(),
        );

        let root_id = main.root_hierarchy_node_ids()[0];
        let leaf_id = main.hierarchy_node(root_id).unwrap().child_node_ids[0];

        main.set_root_hierarchy_node_ids(vec![leaf_id]).unwrap();
        main.release_hierarchy_node(root_id).unwrap();
        main.gc().unwrap();

        // The released root left the scene; the new root joins it on write.
        assert_eq!(main.ext().nodes.len(), 1);
        assert!(main.ext().scenes[0].node_ids.is_empty());

        let loaded = round_trip(&main, &GltfWriteOptions::default());
        assert_eq!(loaded.hierarchy_node_count(), 1);
        assert_eq!(loaded.ext().scenes[0].node_ids, vec![U32Id::from_u32(0)]);
        assert_eq!(
            loaded.hierarchy_node(U32Id::from_u32(0)).unwrap().name,
            "leaf"
        );
    }

    /// An empty primitive stays out of the file, and the ones around it keep
    /// their geometry.
    #[test]
    fn an_empty_primitive_leaves_the_file() {
        let mut main = to_gltf_mesh_main(test_main());
        let object_id = main.iter_objects().next().unwrap().0;
        let primitive_id = main
            .retain_primitive(object_id, MeshPrimitive::new(vec![], vec![]).unwrap())
            .unwrap();
        main.ext_mut()
            .meshes
            .get_mut(&object_id)
            .unwrap()
            .primitives
            .insert(primitive_id, GltfExtPrimitive::default());
        let primitive_count = main.object(object_id).unwrap().primitive_count();

        let file = to_gltf_file(&DependenciesImpl, &main, &GltfWriteOptions::default()).unwrap();
        assert_eq!(file.root.meshes[0].primitives.len(), primitive_count - 1);
        assert!(
            file.root
                .accessors
                .iter()
                .all(|accessor| accessor.count.0 > 0)
        );
        assert!(
            file.root
                .buffer_views
                .iter()
                .all(|view| view.byte_length.0 > 0)
        );

        let loaded = from_gltf_file(&DependenciesImpl, &file).unwrap();
        loaded.validate().unwrap();
        assert_eq!(
            loaded.object(object_id).unwrap().primitive_count(),
            primitive_count - 1
        );
    }

    /// A material with properties over non-object extras cannot land them.
    #[test]
    fn properties_over_non_object_extras_error() {
        let mut main = to_gltf_mesh_main(test_main());

        main.ext_mut()
            .materials
            .get_mut(&U32Id::from_u32(0))
            .unwrap()
            .extras = Some(json!(3));

        assert!(to_gltf_file(&DependenciesImpl, &main, &GltfWriteOptions::default()).is_err());

        let mut material = main.material(U32Id::from_u32(0)).unwrap().clone();
        material.properties = vec![MeshProperty {
            name: "flag".to_owned(),
            value: MeshPropertyValue::Bool(true),
        }];
        main.ext_mut()
            .materials
            .get_mut(&U32Id::from_u32(0))
            .unwrap()
            .extras = None;
        main.set_material(U32Id::from_u32(0), material).unwrap();

        let file = to_gltf_file(&DependenciesImpl, &main, &GltfWriteOptions::default()).unwrap();
        assert!(
            file.root.materials[0]
                .extras
                .as_ref()
                .unwrap()
                .get()
                .contains("\"flag\":true")
        );
    }
}
