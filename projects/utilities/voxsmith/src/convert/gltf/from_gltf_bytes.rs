use crate::{
    Error, Mesh, MeshBaseColorMap, MeshEmissiveMap, MeshMaterial, MeshMaterialMaps,
    MeshMetallicRoughnessMap, MeshOcclusionMap, MeshSampler, MeshTexture, MeshTriangle,
    MeshTriangleUvs, MeshWrap, Result,
};
use gltf::{
    Material, Node, Texture,
    buffer::Data,
    image::{Data as ImageData, Format as ImageFormat},
    import_slice,
    mesh::Mode,
    texture::WrappingMode,
};
use std::collections::HashMap;
use ty_math::{
    TyFloatExt, TyLinSrgbaF64, TyMatrix4x4F64, TySrgbaF64, TySrgbaU8, TyVector2F64, TyVector3Ext,
    TyVector3F64,
};

/// Reads a glTF or GLB byte slice into a [`Mesh`]: every triangle in world
/// space, tagged with its per-primitive material and per-vertex texture
/// coordinates, with glTF's Y-up axes converted to the Voxel Json Z-up
/// convention so a voxelized model stands upright.
///
/// Node and scene transforms are applied, so an authored scale bakes into the
/// points and two exports of one object at different scales voxelize alike.
/// Only `triangles`-mode primitives contribute; points, lines, and triangle
/// strips and fans are skipped. Distinct glTF materials are deduplicated by
/// index, so primitives sharing a material share one material entry, and every
/// primitive with no assigned material shares the glTF default. A material's
/// base-color, metallic-roughness, emissive, and occlusion textures are each
/// decoded once into the mesh texture table for per-texel sampling. A `.gltf`
/// that references external buffer or image files cannot be resolved from bytes
/// alone and errors.
pub fn from_gltf_bytes(bytes: &[u8]) -> Result<Mesh> {
    let (document, buffers, images) = import_slice(bytes)?;

    let scene = document
        .default_scene()
        .or_else(|| document.scenes().next())
        .ok_or_else(|| Error::invalid("glTF has no scene"))?;

    let mut mesh = Mesh {
        triangles: Vec::new(),
        materials: Vec::new(),
        maps: Vec::new(),
        textures: Vec::new(),
        name: None,
    };

    let mut slots: HashMap<Option<usize>, u32> = HashMap::new();

    let mut image_slots: HashMap<usize, usize> = HashMap::new();

    let identity = TyMatrix4x4F64::IDENTITY;

    for node in scene.nodes() {
        collect_node(
            &node,
            &identity,
            &buffers,
            &images,
            &mut mesh,
            &mut slots,
            &mut image_slots,
        );
    }

    Ok(mesh)
}

/// Appends `node`'s triangles in world space, then recurses into its children.
/// `parent` is the accumulated world transform of `node`'s parent.
#[allow(clippy::too_many_arguments)]
fn collect_node(
    node: &Node,
    parent: &TyMatrix4x4F64,
    buffers: &[Data],
    images: &[ImageData],
    mesh: &mut Mesh,
    slots: &mut HashMap<Option<usize>, u32>,
    image_slots: &mut HashMap<usize, usize>,
) {
    let local = TyMatrix4x4F64::from_cols_array_2d(
        &node
            .transform()
            .matrix()
            .map(|column| column.map(f64::from)),
    );
    let world = *parent * local;

    if let Some(node_mesh) = node.mesh() {
        // Name the object after the first mesh-bearing node, its own name
        // preferred over its mesh's.
        if mesh.name.is_none() {
            mesh.name = node.name().or_else(|| node_mesh.name()).map(str::to_owned);
        }

        for primitive in node_mesh.primitives() {
            if primitive.mode() != Mode::Triangles {
                continue;
            }

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()][..]));

            let Some(positions) = reader.read_positions() else {
                continue;
            };

            let positions: Vec<TyVector3F64> = positions
                .map(|p| world_z_up(&world, [p[0] as f64, p[1] as f64, p[2] as f64]))
                .collect();

            let gltf_material = primitive.material();

            let material = material_slot(&gltf_material, mesh, slots, images, image_slots);

            // The TEXCOORD set each map slot declares, in the fixed slot order
            // base color, metallic-roughness, emissive, occlusion.
            let pbr = gltf_material.pbr_metallic_roughness();
            let slot_sets = [
                pbr.base_color_texture().map(|info| info.tex_coord()),
                pbr.metallic_roughness_texture()
                    .map(|info| info.tex_coord()),
                gltf_material
                    .emissive_texture()
                    .map(|info| info.tex_coord()),
                gltf_material
                    .occlusion_texture()
                    .map(|info| info.tex_coord()),
            ];

            // Read each declared set once into per-vertex coordinates, so slots
            // sharing a set (the common case) do not re-read it.
            let mut set_cache: HashMap<u32, Option<Vec<TyVector2F64>>> = HashMap::new();
            let slot_uvs = slot_sets.map(|set| {
                let set = set?;
                set_cache
                    .entry(set)
                    .or_insert_with(|| {
                        reader.read_tex_coords(set).map(|coords| {
                            coords
                                .into_f32()
                                .map(|uv| TyVector2F64::new(uv[0] as f64, uv[1] as f64))
                                .collect()
                        })
                    })
                    .clone()
            });

            match reader.read_indices() {
                Some(indices) => {
                    let indices: Vec<u32> = indices.into_u32().collect();
                    for face in indices.chunks_exact(3) {
                        let corners = [face[0] as usize, face[1] as usize, face[2] as usize];
                        if let (Some(&a), Some(&b), Some(&c)) = (
                            positions.get(corners[0]),
                            positions.get(corners[1]),
                            positions.get(corners[2]),
                        ) {
                            mesh.triangles.push(MeshTriangle {
                                points: [a, b, c],
                                uvs: face_slot_uvs(&slot_uvs, corners),
                                material,
                            });
                        }
                    }
                }

                None => {
                    // Non-indexed: positions and texture coordinates run
                    // parallel, three vertices per triangle.
                    for (triangle, face) in positions.chunks_exact(3).enumerate() {
                        let base = triangle * 3;
                        mesh.triangles.push(MeshTriangle {
                            points: [face[0], face[1], face[2]],
                            uvs: face_slot_uvs(&slot_uvs, [base, base + 1, base + 2]),
                            material,
                        });
                    }
                }
            }
        }
    }

    for child in node.children() {
        collect_node(&child, &world, buffers, images, mesh, slots, image_slots);
    }
}

/// The per-map texture coordinates for a face's three vertex indices, one slot
/// per map in the `slots` order base color, metallic-roughness, emissive,
/// occlusion.
fn face_slot_uvs(slots: &[Option<Vec<TyVector2F64>>; 4], corners: [usize; 3]) -> MeshTriangleUvs {
    MeshTriangleUvs {
        base_color: face_uvs(slots[0].as_deref(), corners),
        metallic_roughness: face_uvs(slots[1].as_deref(), corners),
        emissive: face_uvs(slots[2].as_deref(), corners),
        occlusion: face_uvs(slots[3].as_deref(), corners),
    }
}

/// The texture-coordinate triple for a face's three vertex indices, or `None`
/// when the slot carried no coordinates or an index is out of range.
fn face_uvs(uvs: Option<&[TyVector2F64]>, corners: [usize; 3]) -> Option<[TyVector2F64; 3]> {
    let uvs = uvs?;
    Some([
        *uvs.get(corners[0])?,
        *uvs.get(corners[1])?,
        *uvs.get(corners[2])?,
    ])
}

/// The material-table slot for `material`, interning its flat factors and texture
/// maps on first sight so primitives sharing a glTF material share one entry.
fn material_slot(
    material: &Material,
    mesh: &mut Mesh,
    slots: &mut HashMap<Option<usize>, u32>,
    images: &[ImageData],
    image_slots: &mut HashMap<usize, usize>,
) -> u32 {
    if let Some(&slot) = slots.get(&material.index()) {
        return slot;
    }

    let slot = mesh.materials.len() as u32;

    mesh.materials.push(mesh_material_from_gltf(material));

    let maps = material_maps(material, &mut mesh.textures, images, image_slots);

    mesh.maps.push(maps);

    slots.insert(material.index(), slot);

    slot
}

/// Reads a glTF material's flat factors into a [`MeshMaterial`]:
/// 1. base color, metallic, and roughness from the metallic-roughness model;
///    the base color is sRGB-encoded, its alpha scaled straight.
/// 2. emissive color sRGB-encoded from `emissiveFactor`.
/// 3. emissive strength, ior, and transmission from their KHR extensions,
///    defaulting to 1, 1.5, and 0 when absent.
///
/// Occlusion has no flat glTF factor and defaults to 1.
fn mesh_material_from_gltf(material: &Material) -> MeshMaterial {
    let pbr = material.pbr_metallic_roughness();

    let base_color =
        TySrgbaF64::from_linear(linear_rgba(pbr.base_color_factor())).into_format::<u8, u8>();

    let emissive_factor = emissive_srgb(material.emissive_factor());

    MeshMaterial {
        base_color,
        metallic: pbr.metallic_factor() as f64,
        roughness: pbr.roughness_factor() as f64,
        emissive_factor,
        emissive_strength: material.emissive_strength().unwrap_or(1.0) as f64,
        occlusion: 1.0,
        ior: material.ior().unwrap_or(1.5) as f64,
        transmission: material
            .transmission()
            .map(|transmission| transmission.transmission_factor() as f64)
            .unwrap_or(0.0),
    }
}

/// A glTF linear-RGB emissive factor sRGB-encoded to a stored color, its alpha
/// held opaque since the emissive slot carries none.
fn emissive_srgb(factor: [f32; 3]) -> TySrgbaU8 {
    let linear = TyLinSrgbaF64::new(factor[0] as f64, factor[1] as f64, factor[2] as f64, 1.0);

    TySrgbaF64::from_linear(linear).into_format::<u8, u8>()
}

/// A material's texture bindings, decoding each referenced image into `textures`
/// on first sight and interning it. Absent maps are `None`. Each binding carries
/// the factor its sampled texel combines with, per the glTF metallic-roughness
/// model.
fn material_maps(
    material: &Material,
    textures: &mut Vec<MeshTexture>,
    images: &[ImageData],
    image_slots: &mut HashMap<usize, usize>,
) -> MeshMaterialMaps {
    let pbr = material.pbr_metallic_roughness();

    let base_color = pbr.base_color_texture().map(|info| MeshBaseColorMap {
        sampler: sampler_of(&info.texture(), textures, images, image_slots),
        factor: linear_rgba(pbr.base_color_factor()),
    });

    let metallic_roughness =
        pbr.metallic_roughness_texture()
            .map(|info| MeshMetallicRoughnessMap {
                sampler: sampler_of(&info.texture(), textures, images, image_slots),
                metallic: pbr.metallic_factor() as f64,
                roughness: pbr.roughness_factor() as f64,
            });

    let emissive = material.emissive_texture().map(|info| MeshEmissiveMap {
        sampler: sampler_of(&info.texture(), textures, images, image_slots),
        factor: material.emissive_factor().map(f64::from),
    });

    let occlusion = material.occlusion_texture().map(|info| MeshOcclusionMap {
        sampler: sampler_of(&info.texture(), textures, images, image_slots),
        strength: info.strength() as f64,
    });

    MeshMaterialMaps {
        base_color,
        metallic_roughness,
        emissive,
        occlusion,
    }
}

/// The sampler for a glTF texture: its image, decoded into `textures` on first
/// sight and interned by glTF image index, and its wrap modes.
fn sampler_of(
    texture: &Texture,
    textures: &mut Vec<MeshTexture>,
    images: &[ImageData],
    image_slots: &mut HashMap<usize, usize>,
) -> MeshSampler {
    let image_index = texture.source().index();

    let image = *image_slots.entry(image_index).or_insert_with(|| {
        textures.push(mesh_texture_from_gltf(&images[image_index]));
        textures.len() - 1
    });

    let sampler = texture.sampler();

    MeshSampler {
        image,
        wrap_s: wrap_of(sampler.wrap_s()),
        wrap_t: wrap_of(sampler.wrap_t()),
    }
}

/// A glTF linear-RGBA factor as a [`TyLinSrgbaF64`].
fn linear_rgba(factor: [f32; 4]) -> TyLinSrgbaF64 {
    TyLinSrgbaF64::new(
        factor[0] as f64,
        factor[1] as f64,
        factor[2] as f64,
        factor[3] as f64,
    )
}

/// The [`MeshWrap`] for a glTF wrapping mode.
fn wrap_of(mode: WrappingMode) -> MeshWrap {
    match mode {
        WrappingMode::Repeat => MeshWrap::Repeat,
        WrappingMode::ClampToEdge => MeshWrap::Clamp,
        WrappingMode::MirroredRepeat => MeshWrap::Mirror,
    }
}

/// Normalizes a decoded glTF image into a row-major RGBA8 [`MeshTexture`]. A
/// channel missing from the source format defaults, alpha to opaque and a lone
/// red channel replicated to gray; 16-bit channels keep their high byte and
/// float channels clamp to `[0, 1]`.
fn mesh_texture_from_gltf(image: &ImageData) -> MeshTexture {
    let pixels = image.width as usize * image.height as usize;

    let (channels, depth) = layout(image.format);

    let stride = channels * depth.bytes();

    let mut texels = Vec::with_capacity(pixels);

    for pixel in 0..pixels {
        let base = pixel * stride;
        let channel = |k: usize| depth.byte(&image.pixels, base + k * depth.bytes());

        let r = channel(0);
        let g = if channels >= 2 { channel(1) } else { r };
        let b = if channels >= 3 {
            channel(2)
        } else if channels == 1 {
            r
        } else {
            0
        };
        let a = if channels >= 4 { channel(3) } else { 255 };

        texels.push([r, g, b, a]);
    }

    MeshTexture::new(image.width, image.height, texels)
}

/// The channel count and per-channel bit depth of a decoded glTF image format.
fn layout(format: ImageFormat) -> (usize, Depth) {
    match format {
        ImageFormat::R8 => (1, Depth::Eight),
        ImageFormat::R8G8 => (2, Depth::Eight),
        ImageFormat::R8G8B8 => (3, Depth::Eight),
        ImageFormat::R8G8B8A8 => (4, Depth::Eight),
        ImageFormat::R16 => (1, Depth::Sixteen),
        ImageFormat::R16G16 => (2, Depth::Sixteen),
        ImageFormat::R16G16B16 => (3, Depth::Sixteen),
        ImageFormat::R16G16B16A16 => (4, Depth::Sixteen),
        ImageFormat::R32G32B32FLOAT => (3, Depth::Float),
        ImageFormat::R32G32B32A32FLOAT => (4, Depth::Float),
    }
}

/// The per-channel storage depth of a decoded glTF image.
#[derive(Clone, Copy)]
enum Depth {
    /// One byte per channel.
    Eight,

    /// Two bytes per channel, little-endian.
    Sixteen,

    /// Four bytes per channel, little-endian `f32`.
    Float,
}

impl Depth {
    /// The byte width of one channel.
    fn bytes(self) -> usize {
        match self {
            Depth::Eight => 1,
            Depth::Sixteen => 2,
            Depth::Float => 4,
        }
    }

    /// The 8-bit value of the channel whose bytes start at `offset`.
    fn byte(self, pixels: &[u8], offset: usize) -> u8 {
        match self {
            Depth::Eight => pixels[offset],
            Depth::Sixteen => pixels[offset + 1],
            Depth::Float => {
                let bytes = [
                    pixels[offset],
                    pixels[offset + 1],
                    pixels[offset + 2],
                    pixels[offset + 3],
                ];
                f32::from_le_bytes(bytes).to_unorm8()
            }
        }
    }
}

/// Transforms `point` by `world` (glTF Y-up) and converts the result to the
/// Voxel Json Z-up axes, a +90 degree rotation about X that sends glTF +Y to
/// +Z, preserving the right-handedness both formats use.
fn world_z_up(world: &TyMatrix4x4F64, point: [f64; 3]) -> TyVector3F64 {
    let [x, y, z] = point;
    let world = world.transform_point3(TyVector3F64::new(x, y, z));
    world.yup_to_zup()
}

#[cfg(test)]
mod tests {
    use crate::{
        BASE_COLOR_FACTOR, EMISSIVE_FACTOR, EMISSIVE_STRENGTH, FillMode, IOR, METALLIC_FACTOR,
        MaterialMode, OCCLUSION_STRENGTH, ROUGHNESS_FACTOR, Result, SurfaceMode,
        TRANSMISSION_FACTOR, from_gltf_bytes, voxelize_mesh,
    };
    use branded_id::U32Id;
    use png::{BitDepth, ColorType, Encoder};
    use ty_math::{TyFloatExt, TyHexColor, TySrgbaU8, TyVector3U32};
    use voxcore::{BVoxPoolValue, VoxMain, VoxValuePool};

    /// A minimal binary glTF (GLB) of an axis-aligned box spanning `[0, sx]`,
    /// `[0, sy]`, `[0, sz]` in glTF Y-up space, indexed triangles. When
    /// `material` is `Some((base_color, metallic, roughness))` the primitive
    /// carries that PBR material; otherwise it uses the glTF default. A
    /// `node_name` names the mesh-bearing node.
    fn box_glb(
        sx: f32,
        sy: f32,
        sz: f32,
        material: Option<([f32; 4], f32, f32)>,
        node_name: Option<&str>,
    ) -> Vec<u8> {
        let verts = [
            [0.0, 0.0, 0.0],
            [sx, 0.0, 0.0],
            [sx, sy, 0.0],
            [0.0, sy, 0.0],
            [0.0, 0.0, sz],
            [sx, 0.0, sz],
            [sx, sy, sz],
            [0.0, sy, sz],
        ];
        let faces = [
            [0u32, 1, 2],
            [0, 2, 3],
            [4, 6, 5],
            [4, 7, 6],
            [0, 4, 5],
            [0, 5, 1],
            [3, 2, 6],
            [3, 6, 7],
            [0, 3, 7],
            [0, 7, 4],
            [1, 5, 6],
            [1, 6, 2],
        ];
        let mut bin = Vec::new();
        for vertex in verts {
            for component in vertex {
                bin.extend_from_slice(&component.to_le_bytes());
            }
        }
        let positions_len = bin.len();
        for face in faces {
            for index in face {
                bin.extend_from_slice(&index.to_le_bytes());
            }
        }
        let material_ref = if material.is_some() {
            r#","material":0"#
        } else {
            ""
        };
        let materials = match material {
            Some((color, metallic, roughness)) => format!(
                concat!(
                    r#","materials":[{{"pbrMetallicRoughness":{{"#,
                    r#""baseColorFactor":[{r},{g},{b},{a}],"#,
                    r#""metallicFactor":{metallic},"roughnessFactor":{roughness}}}}}]"#,
                ),
                r = color[0],
                g = color[1],
                b = color[2],
                a = color[3],
                metallic = metallic,
                roughness = roughness,
            ),
            None => String::new(),
        };
        let node_name_attr = match node_name {
            Some(name) => format!(r#","name":"{name}""#),
            None => String::new(),
        };
        let json = format!(
            concat!(
                r#"{{"asset":{{"version":"2.0"}},"scene":0,"scenes":[{{"nodes":[0]}}],"#,
                r#""nodes":[{{"mesh":0{node_name_attr}}}],"#,
                r#""meshes":[{{"primitives":[{{"attributes":{{"POSITION":0}},"indices":1,"mode":4{material_ref}}}]}}]{materials},"#,
                r#""accessors":[{{"bufferView":0,"componentType":5126,"count":8,"type":"VEC3","#,
                r#""min":[0,0,0],"max":[{sx},{sy},{sz}]}},"#,
                r#"{{"bufferView":1,"componentType":5125,"count":36,"type":"SCALAR"}}],"#,
                r#""bufferViews":[{{"buffer":0,"byteOffset":0,"byteLength":{positions_len}}},"#,
                r#"{{"buffer":0,"byteOffset":{positions_len},"byteLength":{indices_len}}}],"#,
                r#""buffers":[{{"byteLength":{bin_len}}}]}}"#,
            ),
            material_ref = material_ref,
            materials = materials,
            node_name_attr = node_name_attr,
            sx = sx,
            sy = sy,
            sz = sz,
            positions_len = positions_len,
            indices_len = bin.len() - positions_len,
            bin_len = bin.len(),
        );

        let mut json_bytes = json.into_bytes();
        while json_bytes.len() % 4 != 0 {
            json_bytes.push(b' ');
        }
        while bin.len() % 4 != 0 {
            bin.push(0);
        }
        let total = 12 + 8 + json_bytes.len() + 8 + bin.len();

        let mut glb = Vec::new();
        glb.extend_from_slice(&0x4654_6C67u32.to_le_bytes()); // "glTF"
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&(total as u32).to_le_bytes());
        glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x4E4F_534Au32.to_le_bytes()); // "JSON"
        glb.extend_from_slice(&json_bytes);
        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x004E_4942u32.to_le_bytes()); // "BIN\0"
        glb.extend_from_slice(&bin);
        glb
    }

    /// A binary glTF of a unit box whose material carries the three KHR
    /// extensions this importer reads, at the given `emissive_strength`, `ior`,
    /// and `transmission`. Indexed triangles in glTF Y-up space.
    fn extended_material_glb(ior: f32, transmission: f32, emissive_strength: f32) -> Vec<u8> {
        let verts = [
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let faces = [
            [0u32, 1, 2],
            [0, 2, 3],
            [4, 6, 5],
            [4, 7, 6],
            [0, 4, 5],
            [0, 5, 1],
            [3, 2, 6],
            [3, 6, 7],
            [0, 3, 7],
            [0, 7, 4],
            [1, 5, 6],
            [1, 6, 2],
        ];
        let mut bin = Vec::new();
        for vertex in verts {
            for component in vertex {
                bin.extend_from_slice(&component.to_le_bytes());
            }
        }
        let positions_len = bin.len();
        for face in faces {
            for index in face {
                bin.extend_from_slice(&index.to_le_bytes());
            }
        }

        let json = format!(
            concat!(
                r#"{{"asset":{{"version":"2.0"}},"scene":0,"scenes":[{{"nodes":[0]}}],"#,
                r#""extensionsUsed":["KHR_materials_emissive_strength","KHR_materials_ior","KHR_materials_transmission"],"#,
                r#""nodes":[{{"mesh":0}}],"#,
                r#""meshes":[{{"primitives":[{{"attributes":{{"POSITION":0}},"indices":1,"mode":4,"material":0}}]}}],"#,
                r#""materials":[{{"pbrMetallicRoughness":{{"baseColorFactor":[1,0,0,1]}},"#,
                r#""emissiveFactor":[0,1,0],"extensions":{{"#,
                r#""KHR_materials_emissive_strength":{{"emissiveStrength":{emissive_strength}}},"#,
                r#""KHR_materials_ior":{{"ior":{ior}}},"#,
                r#""KHR_materials_transmission":{{"transmissionFactor":{transmission}}}}}}}],"#,
                r#""accessors":[{{"bufferView":0,"componentType":5126,"count":8,"type":"VEC3","#,
                r#""min":[0,0,0],"max":[1,1,1]}},"#,
                r#"{{"bufferView":1,"componentType":5125,"count":36,"type":"SCALAR"}}],"#,
                r#""bufferViews":[{{"buffer":0,"byteOffset":0,"byteLength":{positions_len}}},"#,
                r#"{{"buffer":0,"byteOffset":{positions_len},"byteLength":{indices_len}}}],"#,
                r#""buffers":[{{"byteLength":{bin_len}}}]}}"#,
            ),
            emissive_strength = emissive_strength,
            ior = ior,
            transmission = transmission,
            positions_len = positions_len,
            indices_len = bin.len() - positions_len,
            bin_len = bin.len(),
        );

        let mut json_bytes = json.into_bytes();
        while json_bytes.len() % 4 != 0 {
            json_bytes.push(b' ');
        }
        while bin.len() % 4 != 0 {
            bin.push(0);
        }
        let total = 12 + 8 + json_bytes.len() + 8 + bin.len();

        let mut glb = Vec::new();
        glb.extend_from_slice(&0x4654_6C67u32.to_le_bytes()); // "glTF"
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&(total as u32).to_le_bytes());
        glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x4E4F_534Au32.to_le_bytes()); // "JSON"
        glb.extend_from_slice(&json_bytes);
        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x004E_4942u32.to_le_bytes()); // "BIN\0"
        glb.extend_from_slice(&bin);
        glb
    }

    /// Loads `bytes` into a mesh and voxelizes it, the vxl pipeline in one call.
    #[allow(clippy::too_many_arguments)]
    fn voxelize(
        bytes: &[u8],
        counts: TyVector3U32,
        surface_mode: SurfaceMode,
        fill_mode: FillMode,
        material_mode: MaterialMode,
        fill_color: Option<[u8; 4]>,
        node_scale: f64,
        name: Option<&str>,
        fallback_name: &str,
    ) -> Result<VoxMain> {
        voxelize_mesh(
            &from_gltf_bytes(bytes)?,
            counts,
            surface_mode,
            fill_mode,
            material_mode,
            fill_color,
            node_scale,
            name,
            fallback_name,
        )
    }

    /// The value pool and value id one attribute resolves to on the material a
    /// given voxel samples, through the object's first layer.
    fn voxel_attribute<'a>(
        state: &'a VoxMain,
        position: TyVector3U32,
        attribute: &str,
    ) -> (&'a VoxValuePool, U32Id<BVoxPoolValue>) {
        let (_, object) = state.iter_objects().next().unwrap();
        let (layer, palette_id) = object.iter_layers().next().unwrap();
        let palette = state.palette(palette_id).unwrap();
        let property = palette.property_by_name(attribute).unwrap();
        let voxel = object.voxel_id(position).unwrap();
        let material = object.voxel_material(voxel, layer).unwrap();
        state
            .material_value(palette_id, material, property)
            .unwrap()
    }

    /// The `#RRGGBBAA` hex of the `baseColorFactor` a given voxel samples.
    fn voxel_hex(state: &VoxMain, position: TyVector3U32) -> String {
        let (pool, index) = voxel_attribute(state, position, BASE_COLOR_FACTOR);
        match pool {
            VoxValuePool::Srgba { values } => {
                let [r, g, b, a] = values[index.to_usize_id()];
                TySrgbaU8::from([byte(r), byte(g), byte(b), byte(a)]).to_hex()
            }
            other => panic!("unexpected baseColorFactor pool {other:?}"),
        }
    }

    /// The numeric value of one float attribute a given voxel samples.
    fn voxel_number(state: &VoxMain, position: TyVector3U32, attribute: &str) -> f64 {
        let (pool, index) = voxel_attribute(state, position, attribute);
        match pool {
            VoxValuePool::Float { values, .. } => values[index.to_usize_id()],
            other => panic!("expected a float pool for {attribute}, got {other:?}"),
        }
    }

    /// One sRGB float component in `[0, 1]` mapped to a byte.
    fn byte(component: f64) -> u8 {
        component.to_unorm8()
    }

    /// Encodes `texels` (row-major RGBA8) into a PNG of the given size.
    fn png_rgba(width: u32, height: u32, texels: &[[u8; 4]]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut encoder = Encoder::new(&mut out, width, height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let data: Vec<u8> = texels.iter().flatten().copied().collect();
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&data).unwrap();
        writer.finish().unwrap();
        out
    }

    /// A binary glTF of a unit quad in the glTF XY plane (Z-up XZ, `y = 0`),
    /// carrying `TEXCOORD_0` over the full `[0, 1]` square, a base-color texture
    /// of the embedded `png`, and the given linear `base_color_factor`. The quad
    /// maps `u` across the texture width, so a voxel spanning it averages the
    /// texels under it.
    fn textured_quad_glb(png: &[u8], base_color_factor: [f32; 4]) -> Vec<u8> {
        let positions: [[f32; 3]; 4] = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let uvs: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

        let mut bin = Vec::new();
        for vertex in positions {
            for component in vertex {
                bin.extend_from_slice(&component.to_le_bytes());
            }
        }
        for uv in uvs {
            for component in uv {
                bin.extend_from_slice(&component.to_le_bytes());
            }
        }
        for index in indices {
            bin.extend_from_slice(&index.to_le_bytes());
        }
        let image_offset = bin.len();
        bin.extend_from_slice(png);

        let json = format!(
            concat!(
                r#"{{"asset":{{"version":"2.0"}},"scene":0,"scenes":[{{"nodes":[0]}}],"#,
                r#""nodes":[{{"mesh":0}}],"#,
                r#""meshes":[{{"primitives":[{{"attributes":{{"POSITION":0,"TEXCOORD_0":1}},"#,
                r#""indices":2,"mode":4,"material":0}}]}}],"#,
                r#""materials":[{{"pbrMetallicRoughness":{{"baseColorFactor":[{fr},{fg},{fb},{fa}],"#,
                r#""baseColorTexture":{{"index":0}}}}}}],"#,
                r#""textures":[{{"source":0,"sampler":0}}],"#,
                r#""images":[{{"bufferView":3,"mimeType":"image/png"}}],"samplers":[{{}}],"#,
                r#""accessors":[{{"bufferView":0,"componentType":5126,"count":4,"type":"VEC3","#,
                r#""min":[0,0,0],"max":[1,1,0]}},"#,
                r#"{{"bufferView":1,"componentType":5126,"count":4,"type":"VEC2"}},"#,
                r#"{{"bufferView":2,"componentType":5125,"count":6,"type":"SCALAR"}}],"#,
                r#""bufferViews":[{{"buffer":0,"byteOffset":0,"byteLength":48}},"#,
                r#"{{"buffer":0,"byteOffset":48,"byteLength":32}},"#,
                r#"{{"buffer":0,"byteOffset":80,"byteLength":24}},"#,
                r#"{{"buffer":0,"byteOffset":{image_offset},"byteLength":{image_len}}}],"#,
                r#""buffers":[{{"byteLength":{bin_len}}}]}}"#,
            ),
            fr = base_color_factor[0],
            fg = base_color_factor[1],
            fb = base_color_factor[2],
            fa = base_color_factor[3],
            image_offset = image_offset,
            image_len = png.len(),
            bin_len = bin.len(),
        );

        let mut json_bytes = json.into_bytes();
        while json_bytes.len() % 4 != 0 {
            json_bytes.push(b' ');
        }
        while bin.len() % 4 != 0 {
            bin.push(0);
        }
        let total = 12 + 8 + json_bytes.len() + 8 + bin.len();

        let mut glb = Vec::new();
        glb.extend_from_slice(&0x4654_6C67u32.to_le_bytes()); // "glTF"
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&(total as u32).to_le_bytes());
        glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x4E4F_534Au32.to_le_bytes()); // "JSON"
        glb.extend_from_slice(&json_bytes);
        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x004E_4942u32.to_le_bytes()); // "BIN\0"
        glb.extend_from_slice(&bin);
        glb
    }

    /// The `(r, g, b)` bytes of a `#RRGGBBAA` hex string.
    fn rgb(hex: &str) -> (u8, u8, u8) {
        let color = TySrgbaU8::from_hex(hex).expect("a valid hex color");
        (color.red, color.green, color.blue)
    }

    /// One PBR texture map to attach to the test quad, its PNG, the TEXCOORD set
    /// it samples, and its factors.
    enum MapSpec<'a> {
        BaseColor {
            png: &'a [u8],
            tex_coord: u32,
            factor: [f32; 4],
        },
        MetallicRoughness {
            png: &'a [u8],
            tex_coord: u32,
            metallic: f32,
            roughness: f32,
        },
        Emissive {
            png: &'a [u8],
            tex_coord: u32,
            factor: [f32; 3],
        },
        Occlusion {
            png: &'a [u8],
            tex_coord: u32,
            strength: f32,
        },
    }

    impl MapSpec<'_> {
        /// The map's PNG bytes.
        fn png(&self) -> &[u8] {
            match self {
                MapSpec::BaseColor { png, .. }
                | MapSpec::MetallicRoughness { png, .. }
                | MapSpec::Emissive { png, .. }
                | MapSpec::Occlusion { png, .. } => png,
            }
        }
    }

    /// A binary glTF of a unit quad in the glTF XY plane (Z-up XZ, `y = 0`)
    /// carrying two TEXCOORD sets, `uv0` and `uv1`, and the given PBR maps. Each
    /// map is its own image, texture, and sampler, so a map samples the set it
    /// declares; a voxel spanning the quad averages the texels under it.
    fn pbr_quad_glb(uv0: [[f32; 2]; 4], uv1: [[f32; 2]; 4], maps: &[MapSpec]) -> Vec<u8> {
        let positions: [[f32; 3]; 4] = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

        let mut bin = Vec::new();
        for vertex in positions {
            for component in vertex {
                bin.extend_from_slice(&component.to_le_bytes());
            }
        }
        for uv in uv0 {
            for component in uv {
                bin.extend_from_slice(&component.to_le_bytes());
            }
        }
        for uv in uv1 {
            for component in uv {
                bin.extend_from_slice(&component.to_le_bytes());
            }
        }
        for index in indices {
            bin.extend_from_slice(&index.to_le_bytes());
        }

        // One image bufferView per map, appended after the four geometry views.
        let mut image_bvs = String::new();
        for map in maps {
            let offset = bin.len();
            bin.extend_from_slice(map.png());
            image_bvs.push_str(&format!(
                r#",{{"buffer":0,"byteOffset":{offset},"byteLength":{}}}"#,
                map.png().len()
            ));
        }

        let images: Vec<String> = (0..maps.len())
            .map(|i| format!(r#"{{"bufferView":{},"mimeType":"image/png"}}"#, 4 + i))
            .collect();
        let textures: Vec<String> = (0..maps.len())
            .map(|i| format!(r#"{{"source":{i},"sampler":{i}}}"#))
            .collect();
        let samplers: Vec<String> = (0..maps.len()).map(|_| "{}".to_owned()).collect();

        // The material's map references, split into the pbrMetallicRoughness
        // block and the top-level emissive and occlusion fields.
        let mut pbr = Vec::new();
        let mut top = Vec::new();
        for (i, map) in maps.iter().enumerate() {
            match *map {
                MapSpec::BaseColor {
                    tex_coord, factor, ..
                } => {
                    pbr.push(format!(
                        r#""baseColorFactor":[{},{},{},{}]"#,
                        factor[0], factor[1], factor[2], factor[3]
                    ));
                    pbr.push(format!(
                        r#""baseColorTexture":{{"index":{i},"texCoord":{tex_coord}}}"#
                    ));
                }
                MapSpec::MetallicRoughness {
                    tex_coord,
                    metallic,
                    roughness,
                    ..
                } => {
                    pbr.push(format!(r#""metallicFactor":{metallic}"#));
                    pbr.push(format!(r#""roughnessFactor":{roughness}"#));
                    pbr.push(format!(
                        r#""metallicRoughnessTexture":{{"index":{i},"texCoord":{tex_coord}}}"#
                    ));
                }
                MapSpec::Emissive {
                    tex_coord, factor, ..
                } => {
                    top.push(format!(
                        r#""emissiveFactor":[{},{},{}]"#,
                        factor[0], factor[1], factor[2]
                    ));
                    top.push(format!(
                        r#""emissiveTexture":{{"index":{i},"texCoord":{tex_coord}}}"#
                    ));
                }
                MapSpec::Occlusion {
                    tex_coord,
                    strength,
                    ..
                } => top.push(format!(
                    r#""occlusionTexture":{{"index":{i},"texCoord":{tex_coord},"strength":{strength}}}"#
                )),
            }
        }
        let mut material = Vec::new();
        if !pbr.is_empty() {
            material.push(format!(r#""pbrMetallicRoughness":{{{}}}"#, pbr.join(",")));
        }
        material.extend(top);
        let material = format!("{{{}}}", material.join(","));

        let json = format!(
            concat!(
                r#"{{"asset":{{"version":"2.0"}},"scene":0,"scenes":[{{"nodes":[0]}}],"#,
                r#""nodes":[{{"mesh":0}}],"#,
                r#""meshes":[{{"primitives":[{{"attributes":{{"POSITION":0,"TEXCOORD_0":1,"TEXCOORD_1":2}},"#,
                r#""indices":3,"mode":4,"material":0}}]}}],"#,
                r#""materials":[{material}],"textures":[{textures}],"images":[{images}],"#,
                r#""samplers":[{samplers}],"#,
                r#""accessors":[{{"bufferView":0,"componentType":5126,"count":4,"type":"VEC3","#,
                r#""min":[0,0,0],"max":[1,1,0]}},"#,
                r#"{{"bufferView":1,"componentType":5126,"count":4,"type":"VEC2"}},"#,
                r#"{{"bufferView":2,"componentType":5126,"count":4,"type":"VEC2"}},"#,
                r#"{{"bufferView":3,"componentType":5125,"count":6,"type":"SCALAR"}}],"#,
                r#""bufferViews":[{{"buffer":0,"byteOffset":0,"byteLength":48}},"#,
                r#"{{"buffer":0,"byteOffset":48,"byteLength":32}},"#,
                r#"{{"buffer":0,"byteOffset":80,"byteLength":32}},"#,
                r#"{{"buffer":0,"byteOffset":112,"byteLength":24}}{image_bvs}],"#,
                r#""buffers":[{{"byteLength":{bin_len}}}]}}"#,
            ),
            material = material,
            textures = textures.join(","),
            images = images.join(","),
            samplers = samplers.join(","),
            image_bvs = image_bvs,
            bin_len = bin.len(),
        );

        let mut json_bytes = json.into_bytes();
        while json_bytes.len() % 4 != 0 {
            json_bytes.push(b' ');
        }
        while bin.len() % 4 != 0 {
            bin.push(0);
        }
        let total = 12 + 8 + json_bytes.len() + 8 + bin.len();

        let mut glb = Vec::new();
        glb.extend_from_slice(&0x4654_6C67u32.to_le_bytes()); // "glTF"
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&(total as u32).to_le_bytes());
        glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x4E4F_534Au32.to_le_bytes()); // "JSON"
        glb.extend_from_slice(&json_bytes);
        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x004E_4942u32.to_le_bytes()); // "BIN\0"
        glb.extend_from_slice(&bin);
        glb
    }

    /// A full square UV layout over the quad, mapping each corner to a texture
    /// corner.
    fn full_square() -> [[f32; 2]; 4] {
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
    }

    #[test]
    fn extent_converts_gltf_y_up_to_voxel_json_z_up() {
        // A box tall on glTF +Y should be tall on Voxel Json +Z.
        let extent = from_gltf_bytes(&box_glb(1.0, 4.0, 1.0, None, None))
            .unwrap()
            .extent();
        assert!((extent.x - 1.0).abs() < 1e-6, "x extent {}", extent.x);
        assert!((extent.y - 1.0).abs() < 1e-6, "y extent {}", extent.y);
        assert!((extent.z - 4.0).abs() < 1e-6, "z extent {}", extent.z);
    }

    #[test]
    fn flat_paints_the_whole_body_one_color_over_the_gltf_attributes() {
        let state = voxelize(
            &box_glb(1.0, 4.0, 1.0, None, None),
            TyVector3U32::new(1, 1, 4),
            SurfaceMode::CenterInside,
            FillMode::Solid,
            MaterialMode::Flat,
            Some([255, 0, 0, 255]),
            2.0,
            None,
            "voxelized",
        )
        .unwrap();
        assert_eq!(state.validate(), Ok(()));
        assert_eq!(state.object_count(), 1);
        assert_eq!(state.palette_count(), 1);

        let (_, object) = state.iter_objects().next().unwrap();
        assert_eq!(object.bounds(), TyVector3U32::new(1, 1, 4));
        assert_eq!(object.live_count(), 4);

        // Every mode carries the glTF material properties.
        let (_, palette) = state.iter_palettes().next().unwrap();
        assert_eq!(palette.material_count(), 1);
        assert_eq!(
            palette
                .iter_properties()
                .map(|(_, property)| property.name.as_str())
                .collect::<Vec<_>>(),
            [
                BASE_COLOR_FACTOR,
                METALLIC_FACTOR,
                ROUGHNESS_FACTOR,
                EMISSIVE_FACTOR,
                EMISSIVE_STRENGTH,
                OCCLUSION_STRENGTH,
                IOR,
                TRANSMISSION_FACTOR,
            ]
        );
        assert_eq!(
            voxel_number(&state, TyVector3U32::new(0, 0, 0), EMISSIVE_STRENGTH),
            0.0
        );
        assert_eq!(voxel_hex(&state, TyVector3U32::new(0, 0, 0)), "#FF0000FF");

        // The root node records the meters-per-voxel scale.
        let root = state.root_hierarchy_nodes()[0];
        assert_eq!(state.hierarchy_node(root).unwrap().transform.scale.z, 2.0);
    }

    #[test]
    fn flat_defaults_fill_none_to_white() {
        let state = voxelize(
            &box_glb(1.0, 1.0, 1.0, None, None),
            TyVector3U32::new(1, 1, 1),
            SurfaceMode::CenterInside,
            FillMode::Solid,
            MaterialMode::Flat,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();
        assert_eq!(voxel_hex(&state, TyVector3U32::new(0, 0, 0)), "#FFFFFFFF");
    }

    #[test]
    fn per_primitive_reads_the_pbr_factors_and_srgb_encodes_the_base_color() {
        // Linear base color [1, 0.5, 0] sRGB-encodes to #FFBC00; a dropped gamma
        // would give #FF8000 instead.
        let state = voxelize(
            &box_glb(
                1.0,
                1.0,
                1.0,
                Some(([1.0, 0.5, 0.0, 1.0], 0.25, 0.75)),
                None,
            ),
            TyVector3U32::new(1, 1, 1),
            SurfaceMode::CenterInside,
            FillMode::Solid,
            MaterialMode::PerPrimitive,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();
        assert_eq!(state.validate(), Ok(()));

        let (_, palette) = state.iter_palettes().next().unwrap();
        assert_eq!(palette.material_count(), 1);
        let origin = TyVector3U32::new(0, 0, 0);
        assert_eq!(voxel_hex(&state, origin), "#FFBC00FF");
        assert_eq!(voxel_number(&state, origin, METALLIC_FACTOR), 0.25);
        assert_eq!(voxel_number(&state, origin, ROUGHNESS_FACTOR), 0.75);
        assert_eq!(voxel_number(&state, origin, OCCLUSION_STRENGTH), 1.0);
    }

    #[test]
    fn per_primitive_reads_the_khr_material_extensions() {
        // The three KHR extension factors import at their authored values.
        let state = voxelize(
            &extended_material_glb(1.4, 0.5, 3.0),
            TyVector3U32::new(1, 1, 1),
            SurfaceMode::CenterInside,
            FillMode::Solid,
            MaterialMode::PerPrimitive,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();
        assert_eq!(state.validate(), Ok(()));

        let origin = TyVector3U32::new(0, 0, 0);
        // glTF factors are f32, widened to f64 on store.
        assert_eq!(voxel_number(&state, origin, IOR), 1.4f32 as f64);
        assert_eq!(voxel_number(&state, origin, TRANSMISSION_FACTOR), 0.5);
        assert_eq!(voxel_number(&state, origin, EMISSIVE_STRENGTH), 3.0);
    }

    #[test]
    fn a_plain_gltf_imports_the_neutral_ior_and_transmission_defaults() {
        // No KHR extensions imports the defaults: ior 1.5, transmission 0.
        let state = voxelize(
            &box_glb(1.0, 1.0, 1.0, Some(([1.0, 0.0, 0.0, 1.0], 0.0, 1.0)), None),
            TyVector3U32::new(1, 1, 1),
            SurfaceMode::CenterInside,
            FillMode::Solid,
            MaterialMode::PerPrimitive,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();

        let origin = TyVector3U32::new(0, 0, 0);
        assert_eq!(voxel_number(&state, origin, IOR), 1.5);
        assert_eq!(voxel_number(&state, origin, TRANSMISSION_FACTOR), 0.0);
    }

    #[test]
    fn khr_extension_factors_round_trip_through_a_mesh_export() {
        use crate::{
            AtlasShape, MaterialMeshRequest, MeshMethod, ResourceStorage, object_to_material_glb,
        };
        use gltf::Gltf;

        // Import the three KHR factors, voxelize, then mesh back out: each factor
        // rides on the exported material as its flat KHR extension.
        let state = voxelize(
            &extended_material_glb(1.4, 0.5, 3.0),
            TyVector3U32::new(1, 1, 1),
            SurfaceMode::CenterInside,
            FillMode::Solid,
            MaterialMode::PerPrimitive,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();

        let (_, object) = state.iter_objects().next().unwrap();
        let layer = object.iter_layers().next().unwrap().0;
        let request = MaterialMeshRequest {
            method: MeshMethod::Greedy,
            layer,
            scale: 1.0,
            maps: Vec::new(),
            storage: ResourceStorage::Embedded,
            shape: AtlasShape::Fit,
        };

        let files = object_to_material_glb(&state, object, &request).unwrap();
        let gltf = Gltf::from_slice(&files.mesh).unwrap();
        let material = gltf.materials().next().unwrap();

        assert!((material.ior().unwrap() - 1.4).abs() < 1e-4, "ior");
        assert!(
            (material.transmission().unwrap().transmission_factor() - 0.5).abs() < 1e-6,
            "transmission"
        );
        assert!(
            (material.emissive_strength().unwrap() - 3.0).abs() < 1e-6,
            "emissive strength"
        );
    }

    #[test]
    fn per_primitive_fills_a_solid_interior_from_the_nearest_surface() {
        // A one-material solid box: with fill none, the invented interior adopts
        // the surface material, so the whole body is that one color and cell.
        let state = voxelize(
            &box_glb(1.0, 1.0, 1.0, Some(([1.0, 0.0, 0.0, 1.0], 0.0, 1.0)), None),
            TyVector3U32::new(4, 4, 4),
            SurfaceMode::CenterInside,
            FillMode::Solid,
            MaterialMode::PerPrimitive,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();

        let (_, object) = state.iter_objects().next().unwrap();
        assert_eq!(object.live_count(), 64);
        let (_, palette) = state.iter_palettes().next().unwrap();
        assert_eq!(palette.material_count(), 1);
        // A deep interior voxel resolved to the surface color.
        assert_eq!(voxel_hex(&state, TyVector3U32::new(2, 2, 2)), "#FF0000FF");
    }

    #[test]
    fn per_primitive_paints_a_solid_interior_with_a_given_fill_color() {
        // With an explicit fill color, the interior is that color, not the
        // sampled surface, so the palette carries both.
        let state = voxelize(
            &box_glb(1.0, 1.0, 1.0, Some(([1.0, 0.0, 0.0, 1.0], 0.0, 1.0)), None),
            TyVector3U32::new(4, 4, 4),
            SurfaceMode::CenterInside,
            FillMode::Solid,
            MaterialMode::PerPrimitive,
            Some([0, 0, 255, 255]),
            1.0,
            None,
            "voxelized",
        )
        .unwrap();

        let (_, palette) = state.iter_palettes().next().unwrap();
        assert_eq!(palette.material_count(), 2);
        assert_eq!(voxel_hex(&state, TyVector3U32::new(0, 0, 0)), "#FF0000FF");
        assert_eq!(voxel_hex(&state, TyVector3U32::new(2, 2, 2)), "#0000FFFF");
    }

    #[test]
    fn rejects_a_grid_past_the_dense_limit() {
        // 1024^3 = 2^30 cells, well past the 2^27 dense limit; this must error
        // before allocating the occupancy grid rather than exhaust memory.
        let result = voxelize(
            &box_glb(1.0, 1.0, 1.0, None, None),
            TyVector3U32::new(1024, 1024, 1024),
            SurfaceMode::CenterInside,
            FillMode::Solid,
            MaterialMode::Flat,
            None,
            1.0,
            None,
            "voxelized",
        );
        assert!(result.is_err());
    }

    /// Voxelizes a 1x1x1 box and returns the one object's name.
    fn object_name(bytes: &[u8], name: Option<&str>, fallback: &str) -> String {
        let state = voxelize(
            bytes,
            TyVector3U32::new(1, 1, 1),
            SurfaceMode::CenterInside,
            FillMode::Solid,
            MaterialMode::Flat,
            None,
            1.0,
            name,
            fallback,
        )
        .unwrap();
        state.iter_objects().next().unwrap().1.name().to_owned()
    }

    #[test]
    fn names_the_object_from_the_gltf_node_when_not_overridden() {
        let glb = box_glb(1.0, 1.0, 1.0, None, Some("Ship"));
        assert_eq!(object_name(&glb, None, "stem"), "Ship");
    }

    #[test]
    fn an_explicit_name_beats_the_gltf_node() {
        let glb = box_glb(1.0, 1.0, 1.0, None, Some("Ship"));
        assert_eq!(object_name(&glb, Some("Override"), "stem"), "Override");
    }

    #[test]
    fn falls_back_to_the_given_name_when_the_gltf_is_unnamed() {
        let glb = box_glb(1.0, 1.0, 1.0, None, None);
        assert_eq!(object_name(&glb, None, "stem"), "stem");
    }

    #[test]
    fn per_texel_reads_the_base_color_texture_a_white_factor_hides() {
        // The motivating case: a white base-color factor over a red texture. Per
        // primitive reads only the white factor; per texel samples the texture.
        let png = png_rgba(1, 1, &[[255, 0, 0, 255]]);
        let glb = textured_quad_glb(&png, [1.0, 1.0, 1.0, 1.0]);

        let per_texel = voxelize(
            &glb,
            TyVector3U32::new(2, 1, 2),
            SurfaceMode::TriangleCover,
            FillMode::Surface,
            MaterialMode::PerTexel,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();
        assert_eq!(per_texel.validate(), Ok(()));
        // Every surface voxel samples the one red texel, so one merged cell.
        let (_, palette) = per_texel.iter_palettes().next().unwrap();
        assert_eq!(palette.material_count(), 1);
        assert_eq!(
            voxel_hex(&per_texel, TyVector3U32::new(0, 0, 0)),
            "#FF0000FF"
        );

        let per_primitive = voxelize(
            &glb,
            TyVector3U32::new(2, 1, 2),
            SurfaceMode::TriangleCover,
            FillMode::Surface,
            MaterialMode::PerPrimitive,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();
        // Per primitive reads the white factor, missing the texture entirely.
        assert_eq!(
            voxel_hex(&per_primitive, TyVector3U32::new(0, 0, 0)),
            "#FFFFFFFF"
        );
    }

    #[test]
    fn per_texel_multiplies_the_base_color_factor_into_the_texel() {
        // A white texel under a linear blue factor resolves to blue.
        let png = png_rgba(1, 1, &[[255, 255, 255, 255]]);
        let glb = textured_quad_glb(&png, [0.0, 0.0, 1.0, 1.0]);

        let state = voxelize(
            &glb,
            TyVector3U32::new(1, 1, 1),
            SurfaceMode::TriangleCover,
            FillMode::Surface,
            MaterialMode::PerTexel,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();
        assert_eq!(voxel_hex(&state, TyVector3U32::new(0, 0, 0)), "#0000FFFF");
    }

    #[test]
    fn auto_samples_per_texel_when_textured_and_per_primitive_when_not() {
        // Textured: a white factor over a red texture. Auto samples the texture.
        let png = png_rgba(1, 1, &[[255, 0, 0, 255]]);
        let textured = voxelize(
            &textured_quad_glb(&png, [1.0, 1.0, 1.0, 1.0]),
            TyVector3U32::new(1, 1, 1),
            SurfaceMode::TriangleCover,
            FillMode::Surface,
            MaterialMode::Auto,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();
        assert_eq!(
            voxel_hex(&textured, TyVector3U32::new(0, 0, 0)),
            "#FF0000FF"
        );

        // Untextured: a green flat factor. Auto reads it per primitive.
        let untextured = voxelize(
            &box_glb(1.0, 1.0, 1.0, Some(([0.0, 1.0, 0.0, 1.0], 0.0, 1.0)), None),
            TyVector3U32::new(1, 1, 1),
            SurfaceMode::CenterInside,
            FillMode::Solid,
            MaterialMode::Auto,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();
        assert_eq!(
            voxel_hex(&untextured, TyVector3U32::new(0, 0, 0)),
            "#00FF00FF"
        );
    }

    #[test]
    fn per_texel_area_averages_the_voxel_footprint() {
        // A black/white texture under one voxel spanning both texels averages to
        // a gray, where point sampling would land on one extreme.
        let png = png_rgba(2, 1, &[[0, 0, 0, 255], [255, 255, 255, 255]]);
        let glb = textured_quad_glb(&png, [1.0, 1.0, 1.0, 1.0]);

        let state = voxelize(
            &glb,
            TyVector3U32::new(1, 1, 1),
            SurfaceMode::TriangleCover,
            FillMode::Surface,
            MaterialMode::PerTexel,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();
        let (r, g, b) = rgb(&voxel_hex(&state, TyVector3U32::new(0, 0, 0)));
        assert!(
            r == g && g == b,
            "the blend of grays stays neutral: {r} {g} {b}"
        );
        assert!(r > 0 && r < 255, "a blend, not either extreme: {r}");
    }

    #[test]
    fn per_texel_reads_metallic_roughness_as_linear_data() {
        // glTF packs metallic in blue and roughness in green, both linear data.
        // A green of 128 is roughness ~0.502 straight, not the ~0.216 an sRGB
        // decode would give, and the metallic factor scales the blue channel.
        let mr = png_rgba(1, 1, &[[0, 128, 64, 255]]);
        let glb = pbr_quad_glb(
            full_square(),
            full_square(),
            &[MapSpec::MetallicRoughness {
                png: &mr,
                tex_coord: 0,
                metallic: 0.5,
                roughness: 1.0,
            }],
        );

        let state = voxelize(
            &glb,
            TyVector3U32::new(1, 1, 1),
            SurfaceMode::TriangleCover,
            FillMode::Surface,
            MaterialMode::PerTexel,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();

        let metallic = voxel_number(&state, TyVector3U32::new(0, 0, 0), METALLIC_FACTOR);
        let roughness = voxel_number(&state, TyVector3U32::new(0, 0, 0), ROUGHNESS_FACTOR);
        assert!(
            (metallic - 0.5 * 64.0 / 255.0).abs() < 0.01,
            "metallic {metallic}"
        );
        assert!(
            (roughness - 128.0 / 255.0).abs() < 0.01,
            "roughness {roughness}"
        );
    }

    #[test]
    fn per_texel_reads_emissive_as_an_srgb_color() {
        // Emissive is sampled as a color, not collapsed to a scalar. A green of
        // 188 sRGB decodes to ~0.503 linear, is tinted by the unit factor, then
        // re-encodes to ~188 (~0.737 as a float), so the stored emissiveFactor is
        // green with no red or blue and the strength stays the material's unit
        // factor.
        let emissive = png_rgba(1, 1, &[[0, 188, 0, 255]]);
        let glb = pbr_quad_glb(
            full_square(),
            full_square(),
            &[MapSpec::Emissive {
                png: &emissive,
                tex_coord: 0,
                factor: [1.0, 1.0, 1.0],
            }],
        );

        let state = voxelize(
            &glb,
            TyVector3U32::new(1, 1, 1),
            SurfaceMode::TriangleCover,
            FillMode::Surface,
            MaterialMode::PerTexel,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();

        let origin = TyVector3U32::new(0, 0, 0);
        let (pool, index) = voxel_attribute(&state, origin, EMISSIVE_FACTOR);
        let [r, g, b] = match pool {
            VoxValuePool::Srgb { values } => values[index.to_usize_id()],
            other => panic!("expected an sRGB emissiveFactor pool, got {other:?}"),
        };
        assert!(r < 0.01 && b < 0.01, "emissiveFactor red {r} blue {b}");
        assert!((g - 188.0 / 255.0).abs() < 0.01, "emissiveFactor green {g}");
        assert_eq!(
            voxel_number(&state, origin, EMISSIVE_STRENGTH),
            1.0,
            "emissive strength stays the material's unit factor"
        );
    }

    #[test]
    fn per_texel_reads_occlusion_with_strength() {
        // Occlusion is linear data in red; a red of 128 is ~0.502 straight, and
        // strength 0.5 gives 1 + 0.5 * (0.502 - 1) = ~0.751, distinct from the
        // ~0.608 an sRGB decode would give.
        let occ = png_rgba(1, 1, &[[128, 0, 0, 255]]);
        let glb = pbr_quad_glb(
            full_square(),
            full_square(),
            &[MapSpec::Occlusion {
                png: &occ,
                tex_coord: 0,
                strength: 0.5,
            }],
        );

        let state = voxelize(
            &glb,
            TyVector3U32::new(1, 1, 1),
            SurfaceMode::TriangleCover,
            FillMode::Surface,
            MaterialMode::PerTexel,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();

        let occlusion = voxel_number(&state, TyVector3U32::new(0, 0, 0), OCCLUSION_STRENGTH);
        assert!((occlusion - 0.751).abs() < 0.01, "occlusion {occlusion}");
    }

    #[test]
    fn per_texel_samples_each_maps_own_texcoord_set() {
        // Base color reads TEXCOORD_0 and metallic-roughness reads TEXCOORD_1,
        // with the two sets pointing at different texels: set 0 at the left texel,
        // set 1 at the right. Base picks the red left half and metallic-roughness
        // the metal right half, so a shared set would cross the wires.
        let base = png_rgba(2, 1, &[[255, 0, 0, 255], [0, 0, 255, 255]]);
        let mr = png_rgba(2, 1, &[[0, 0, 0, 255], [0, 255, 255, 255]]);
        let left = [[0.25, 0.5]; 4];
        let right = [[0.75, 0.5]; 4];
        let glb = pbr_quad_glb(
            left,
            right,
            &[
                MapSpec::BaseColor {
                    png: &base,
                    tex_coord: 0,
                    factor: [1.0, 1.0, 1.0, 1.0],
                },
                MapSpec::MetallicRoughness {
                    png: &mr,
                    tex_coord: 1,
                    metallic: 1.0,
                    roughness: 1.0,
                },
            ],
        );

        let state = voxelize(
            &glb,
            TyVector3U32::new(1, 1, 1),
            SurfaceMode::TriangleCover,
            FillMode::Surface,
            MaterialMode::PerTexel,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();

        assert_eq!(voxel_hex(&state, TyVector3U32::new(0, 0, 0)), "#FF0000FF");
        let metallic = voxel_number(&state, TyVector3U32::new(0, 0, 0), METALLIC_FACTOR);
        assert!((metallic - 1.0).abs() < 1e-9, "metallic {metallic}");
    }
}
