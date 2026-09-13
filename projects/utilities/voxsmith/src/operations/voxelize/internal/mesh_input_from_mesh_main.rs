use crate::{
    Error, Result,
    dependencies::voxelize::DecodeImage,
    operations::voxelize::{
        MeshBaseColorMap, MeshEmissiveMap, MeshInput, MeshMaterial, MeshMaterialMaps,
        MeshMetallicRoughnessMap, MeshOcclusionMap, MeshSampler, MeshTexture, MeshTriangle,
        MeshTriangleUvs, MeshWrap,
    },
};
use branded_id::{IdSlice, U32Id};
use meshdoc::{
    BMeshHierarchyNode, BMeshImage, BMeshMaterial, BMeshVertex, MeshExt, MeshMain,
    MeshMaterial as DocumentMaterial, MeshPrimitive, MeshTextureRef, MeshWrap as DocumentWrap,
};
use std::collections::HashMap;
use ty_math::{TyLinSrgbaF64, TyTransformF64, TyVector2F64, TyVector3F64};

/// Flattens a mesh document into the world-space triangle mesh the
/// voxelizer rasterizes: every object the hierarchy places, its node
/// transforms applied, so two documents of one object at different scales
/// voxelize alike. Each distinct material becomes one entry, and each
/// texture it draws decodes once through `dependencies`. A primitive with
/// no material draws the default material. The mesh takes the name of the
/// first placed object, or of the node placing it when the object has none.
/// Errors when an image does not decode.
pub fn mesh_input_from_mesh_main<D: DecodeImage, T: MeshExt>(
    dependencies: &D,
    main: &MeshMain<T>,
) -> Result<MeshInput> {
    let mut input = MeshInput {
        triangles: Vec::new(),
        materials: Vec::new(),
        maps: Vec::new(),
        textures: Vec::new(),
        name: None,
    };

    let mut slots: HashMap<Option<U32Id<BMeshMaterial>>, u32> = HashMap::new();

    let mut image_slots: HashMap<U32Id<BMeshImage>, usize> = HashMap::new();

    let mut walk = Walk {
        dependencies,
        main,
        input: &mut input,
        slots: &mut slots,
        image_slots: &mut image_slots,
    };

    for &root_id in main.root_hierarchy_node_ids() {
        walk.node(root_id, &TyTransformF64::default())?;
    }

    Ok(input)
}

/// The state of one flattening walk.
struct Walk<'a, D, T> {
    dependencies: &'a D,
    main: &'a MeshMain<T>,
    input: &'a mut MeshInput,
    slots: &'a mut HashMap<Option<U32Id<BMeshMaterial>>, u32>,
    image_slots: &'a mut HashMap<U32Id<BMeshImage>, usize>,
}

impl<D: DecodeImage, T: MeshExt> Walk<'_, D, T> {
    /// Appends the triangles of the objects `node_id` places, in world
    /// space under `parent`, then recurses into its children.
    fn node(&mut self, node_id: U32Id<BMeshHierarchyNode>, parent: &TyTransformF64) -> Result<()> {
        let node = self
            .main
            .hierarchy_node(node_id)
            .expect("a root or child is one of the document's nodes");

        let world = parent.compose(&node.transform);

        for &object_id in &node.child_object_ids {
            let object = self
                .main
                .object(object_id)
                .expect("a placed object is one of the document's");

            if self.input.name.is_none() {
                let name = [object.name(), node.name.as_str()]
                    .into_iter()
                    .find(|name| !name.is_empty());
                self.input.name = name.map(str::to_owned);
            }

            for (_, primitive) in object.iter_primitives() {
                self.primitive(primitive, &world)?;
            }
        }

        for &child_id in &node.child_node_ids {
            self.node(child_id, &world)?;
        }

        Ok(())
    }

    /// Appends a primitive's triangles in world space under `world`, tagged
    /// with its material's slot.
    fn primitive(&mut self, primitive: &MeshPrimitive, world: &TyTransformF64) -> Result<()> {
        let material_index = self.material_slot(primitive.material_id())?;

        let material = primitive
            .material_id()
            .map(|material_id| {
                self.main
                    .material(material_id)
                    .expect("a primitive draws one of the document's materials")
            })
            .cloned()
            .unwrap_or_default();

        let positions: Vec<TyVector3F64> = primitive
            .positions()
            .iter()
            .map(|&position| world.transform_point(position))
            .collect();

        // The UV stream each map slot samples, in the fixed slot order base
        // color, metallic-roughness, emissive, occlusion.
        let slot_streams = [
            material.base_color_texture,
            material.metallic_roughness_texture,
            material.emissive_texture,
            material.occlusion_texture,
        ]
        .map(|texture_ref| {
            texture_ref.and_then(|texture_ref| primitive.uv_stream(texture_ref.uv_stream_id))
        });

        for triangle in primitive.triangles() {
            let corners = triangle.vertex_ids.map(|vertex_id| vertex_id.to_usize_id());

            let uvs = |stream: Option<&IdSlice<BMeshVertex, TyVector2F64>>| {
                stream.map(|stream| corners.map(|corner| stream[corner]))
            };

            self.input.triangles.push(MeshTriangle {
                points: corners.map(|corner| positions[corner.to_usize()]),
                uvs: MeshTriangleUvs {
                    base_color: uvs(slot_streams[0]),
                    metallic_roughness: uvs(slot_streams[1]),
                    emissive: uvs(slot_streams[2]),
                    occlusion: uvs(slot_streams[3]),
                },
                material_index,
            });
        }

        Ok(())
    }

    /// The material-table slot for `material_id`, interning its flat factors
    /// and texture maps on first sight so primitives sharing a material share
    /// one entry.
    fn material_slot(&mut self, material_id: Option<U32Id<BMeshMaterial>>) -> Result<u32> {
        if let Some(&slot) = self.slots.get(&material_id) {
            return Ok(slot);
        }

        let material = material_id
            .map(|material_id| {
                self.main
                    .material(material_id)
                    .expect("a primitive draws one of the document's materials")
            })
            .cloned()
            .unwrap_or_default();

        let slot = self.input.materials.len() as u32;

        self.input.materials.push(mesh_material_of(&material));

        let maps = self.material_maps(&material)?;

        self.input.maps.push(maps);

        self.slots.insert(material_id, slot);

        Ok(slot)
    }

    /// A material's texture bindings, decoding each referenced image on first
    /// sight and interning it. Absent maps are `None`. Each binding carries
    /// the factor its sampled texel combines with.
    fn material_maps(&mut self, material: &DocumentMaterial) -> Result<MeshMaterialMaps> {
        let base_color = match material.base_color_texture {
            Some(texture_ref) => Some(MeshBaseColorMap {
                sampler: self.sampler_of(texture_ref)?,
                factor: material.base_color_factor,
            }),
            None => None,
        };

        let metallic_roughness = match material.metallic_roughness_texture {
            Some(texture_ref) => Some(MeshMetallicRoughnessMap {
                sampler: self.sampler_of(texture_ref)?,
                metallic: material.metallic_factor,
                roughness: material.roughness_factor,
            }),
            None => None,
        };

        let emissive = match material.emissive_texture {
            Some(texture_ref) => Some(MeshEmissiveMap {
                sampler: self.sampler_of(texture_ref)?,
                factor: <[f64; 3]>::from(material.emissive_factor),
            }),
            None => None,
        };

        let occlusion = match material.occlusion_texture {
            Some(texture_ref) => Some(MeshOcclusionMap {
                sampler: self.sampler_of(texture_ref)?,
                strength: material.occlusion_strength,
            }),
            None => None,
        };

        Ok(MeshMaterialMaps {
            base_color,
            metallic_roughness,
            emissive,
            occlusion,
        })
    }

    /// The sampler for a texture reference: its image, decoded on first sight
    /// and interned by image id, and its wrap modes.
    fn sampler_of(&mut self, texture_ref: MeshTextureRef) -> Result<MeshSampler> {
        let texture = self
            .main
            .texture(texture_ref.texture_id)
            .expect("a material draws one of the document's textures");

        let image = if let Some(&image) = self.image_slots.get(&texture.image_id) {
            image
        } else {
            let source = self
                .main
                .image(texture.image_id)
                .expect("a texture samples one of the document's images");
            let bytes = self
                .main
                .image_bytes(texture.image_id)
                .expect("an image reads its own bytes or a live file's");

            let decoded = self
                .dependencies
                .decode_image(source.media_type, bytes)
                .map_err(|reason| {
                    Error::DecodeImage(format!(
                        "image {} ({}): {reason}",
                        texture.image_id.to_u32(),
                        source.media_type
                    ))
                })?;

            self.input.textures.push(MeshTexture::new(
                decoded.width,
                decoded.height,
                decoded.pixels,
            ));

            let image = self.input.textures.len() - 1;
            self.image_slots.insert(texture.image_id, image);
            image
        };

        Ok(MeshSampler {
            image,
            wrap_s: wrap_of(texture.wrap_s),
            wrap_t: wrap_of(texture.wrap_t),
        })
    }
}

/// A document material's flat factors as the voxelizer's material. Occlusion
/// has no flat factor and is full; the texture's strength rides on its map.
fn mesh_material_of(material: &DocumentMaterial) -> MeshMaterial {
    let emissive = <[f64; 3]>::from(material.emissive_factor);

    MeshMaterial {
        base_color: material.base_color_factor,
        metallic: material.metallic_factor,
        roughness: material.roughness_factor,
        emissive_color: TyLinSrgbaF64::new(emissive[0], emissive[1], emissive[2], 1.0),
        emissive_strength: material.emissive_strength,
        occlusion: 1.0,
        ior: material.ior,
        transmission: material.transmission_factor,
    }
}

/// The voxelizer's wrap for a document wrap.
fn wrap_of(wrap: DocumentWrap) -> MeshWrap {
    match wrap {
        DocumentWrap::Repeat => MeshWrap::Repeat,
        DocumentWrap::ClampToEdge => MeshWrap::Clamp,
        DocumentWrap::MirroredRepeat => MeshWrap::Mirror,
    }
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        Result,
        dependencies::DependenciesImpl,
        operations::voxelize::{
            FillMode, MaterialMode, OutOfRangeProperty, SurfaceMode, mesh_input_from_mesh_main,
            voxelize_mesh,
        },
    };
    use branded_id::U32Id;
    use meshdoc::{
        BMeshMaterial, BMeshTexture, MeshHierarchyNode, MeshImage, MeshImageMediaType,
        MeshImageSource, MeshMain, MeshMaterial, MeshObject, MeshPrimitive, MeshTexture,
        MeshTextureRef, MeshTriangle,
    };
    use png::{BitDepth, ColorType, Encoder};
    use ty_math::{
        TyLinSrgbF64, TyLinSrgbaF64, TyTransformF64, TyVector2F64, TyVector3F64, TyVector3U32,
    };
    use voxcore::{
        BVoxValuePoolValue, VoxMain, VoxValuePool, VoxValuePoolValueRef,
        color::srgba_u8_from_lin_srgba_f64,
        material::{
            BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC, OCCLUSION_STRENGTH,
            ROUGHNESS, TRANSMISSION,
        },
    };

    /// A triangle over three vertex indices.
    fn triangle(a: u32, b: u32, c: u32) -> MeshTriangle {
        MeshTriangle {
            vertex_ids: [U32Id::from_u32(a), U32Id::from_u32(b), U32Id::from_u32(c)],
        }
    }

    /// A document of one object of one primitive under one root node,
    /// named `node_name` and placed at `transform`.
    fn document_of(
        mut main: MeshMain<()>,
        primitive: MeshPrimitive,
        node_name: Option<&str>,
        transform: TyTransformF64,
    ) -> MeshMain<()> {
        let mut object = MeshObject::new(String::new());
        object.retain_primitive(primitive);
        let object_id = main.retain_object(object).unwrap();

        let node_id = main
            .retain_hierarchy_node(MeshHierarchyNode {
                name: node_name.unwrap_or_default().to_owned(),
                transform,
                child_object_ids: vec![object_id],
                ..Default::default()
            })
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![node_id]).unwrap();
        main.validate().unwrap();
        main
    }

    /// An axis-aligned box spanning `[0, sx]`, `[0, sy]`, `[0, sz]` on the
    /// Z-up axes, indexed triangles. When `material` is
    /// `Some((base_color, metallic, roughness))` the primitive draws that
    /// material; otherwise it draws the default. A `node_name` names the
    /// placing node.
    fn box_main(
        sx: f64,
        sy: f64,
        sz: f64,
        material: Option<([f64; 4], f64, f64)>,
        node_name: Option<&str>,
    ) -> MeshMain<()> {
        let mut main = MeshMain::default();

        let material_id = material.map(|(color, metallic, roughness)| {
            main.retain_material(MeshMaterial {
                base_color_factor: TyLinSrgbaF64::from(color),
                metallic_factor: metallic,
                roughness_factor: roughness,
                ..Default::default()
            })
            .unwrap()
        });

        let mut primitive = box_primitive(sx, sy, sz);
        primitive.set_material_id(material_id);

        document_of(main, primitive, node_name, TyTransformF64::default())
    }

    /// The box primitive of [`box_main`].
    fn box_primitive(sx: f64, sy: f64, sz: f64) -> MeshPrimitive {
        let positions = vec![
            TyVector3F64::new(0.0, 0.0, 0.0),
            TyVector3F64::new(sx, 0.0, 0.0),
            TyVector3F64::new(sx, sy, 0.0),
            TyVector3F64::new(0.0, sy, 0.0),
            TyVector3F64::new(0.0, 0.0, sz),
            TyVector3F64::new(sx, 0.0, sz),
            TyVector3F64::new(sx, sy, sz),
            TyVector3F64::new(0.0, sy, sz),
        ];
        let faces = [
            [0, 1, 2],
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

        MeshPrimitive::new(
            positions,
            faces
                .iter()
                .map(|face| triangle(face[0], face[1], face[2]))
                .collect(),
        )
        .unwrap()
    }

    /// A unit box whose material carries the three factors the voxelizer
    /// reads beyond the metallic-roughness set, at the given `ior`,
    /// `transmission`, and `emissive_strength`, over a green emissive.
    fn extended_material_main(ior: f64, transmission: f64, emissive_strength: f64) -> MeshMain<()> {
        let mut main = MeshMain::default();

        let material_id = main
            .retain_material(MeshMaterial {
                base_color_factor: TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0),
                emissive_factor: TyLinSrgbF64::new(0.0, 1.0, 0.0),
                emissive_strength,
                ior,
                transmission_factor: transmission,
                ..Default::default()
            })
            .unwrap();

        let mut primitive = box_primitive(1.0, 1.0, 1.0);
        primitive.set_material_id(Some(material_id));

        document_of(main, primitive, None, TyTransformF64::default())
    }

    /// Encodes `texels` (row-major RGBA8) into a PNG of the given size.
    fn png_rgba(width: u32, height: u32, texels: &[[u8; 4]]) -> Vec<u8> {
        let samples: Vec<u8> = texels.iter().flatten().copied().collect();
        let mut bytes = Vec::new();
        let mut encoder = Encoder::new(&mut bytes, width, height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&samples).unwrap();
        writer.finish().unwrap();
        bytes
    }

    /// One PBR texture map to attach to the test quad: its PNG, the UV
    /// stream it samples, and its factors.
    enum MapSpec<'a> {
        BaseColor {
            png: &'a [u8],
            stream: u32,
            factor: [f64; 4],
        },
        MetallicRoughness {
            png: &'a [u8],
            stream: u32,
            metallic: f64,
            roughness: f64,
        },
        Emissive {
            png: &'a [u8],
            stream: u32,
            factor: [f64; 3],
        },
        Occlusion {
            png: &'a [u8],
            stream: u32,
            strength: f64,
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

    /// A unit quad in the Z-up XZ plane at `y = 0` carrying two UV streams,
    /// `uv0` and `uv1`, and the given PBR maps. Each map is its own image and
    /// texture, so a map samples the stream it declares; a voxel spanning
    /// the quad averages the texels under it.
    fn pbr_quad_main(uv0: [[f64; 2]; 4], uv1: [[f64; 2]; 4], maps: &[MapSpec]) -> MeshMain<()> {
        let mut main = MeshMain::default();

        let mut material = MeshMaterial::default();

        for map in maps {
            let image_id = main
                .retain_image(MeshImage {
                    name: String::new(),
                    media_type: MeshImageMediaType::Png,
                    source: MeshImageSource::Bytes(map.png().to_vec()),
                })
                .unwrap();
            let texture_id: U32Id<BMeshTexture> =
                main.retain_texture(MeshTexture::new(image_id)).unwrap();
            let texture_ref = |stream: u32| MeshTextureRef {
                texture_id,
                uv_stream_id: U32Id::from_u32(stream),
            };

            match *map {
                MapSpec::BaseColor { stream, factor, .. } => {
                    material.base_color_factor = TyLinSrgbaF64::from(factor);
                    material.base_color_texture = Some(texture_ref(stream));
                }
                MapSpec::MetallicRoughness {
                    stream,
                    metallic,
                    roughness,
                    ..
                } => {
                    material.metallic_factor = metallic;
                    material.roughness_factor = roughness;
                    material.metallic_roughness_texture = Some(texture_ref(stream));
                }
                MapSpec::Emissive { stream, factor, .. } => {
                    material.emissive_factor = TyLinSrgbF64::from(factor);
                    material.emissive_texture = Some(texture_ref(stream));
                }
                MapSpec::Occlusion {
                    stream, strength, ..
                } => {
                    material.occlusion_strength = strength;
                    material.occlusion_texture = Some(texture_ref(stream));
                }
            }
        }

        let material_id: U32Id<BMeshMaterial> = main.retain_material(material).unwrap();

        let mut primitive = MeshPrimitive::new(
            vec![
                TyVector3F64::new(0.0, 0.0, 0.0),
                TyVector3F64::new(1.0, 0.0, 0.0),
                TyVector3F64::new(1.0, 0.0, 1.0),
                TyVector3F64::new(0.0, 0.0, 1.0),
            ],
            vec![triangle(0, 1, 2), triangle(0, 2, 3)],
        )
        .unwrap();

        for uvs in [uv0, uv1] {
            primitive
                .push_uv_stream(uvs.iter().map(|uv| TyVector2F64::from_array(*uv)).collect())
                .unwrap();
        }

        primitive.set_material_id(Some(material_id));

        document_of(main, primitive, None, TyTransformF64::default())
    }

    /// A unit quad with a base-color texture of the given PNG over the full
    /// square and the given linear `factor`.
    fn textured_quad_main(png: &[u8], factor: [f64; 4]) -> MeshMain<()> {
        pbr_quad_main(
            full_square(),
            full_square(),
            &[MapSpec::BaseColor {
                png,
                stream: 0,
                factor,
            }],
        )
    }

    /// A full square UV layout over the quad, mapping each corner to a
    /// texture corner.
    fn full_square() -> [[f64; 2]; 4] {
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
    }

    /// Flattens `main` and voxelizes it, the whole import in one call.
    #[allow(clippy::too_many_arguments)]
    fn voxelize(
        main: &MeshMain<()>,
        counts: TyVector3U32,
        surface_mode: SurfaceMode,
        fill_mode: FillMode,
        material_mode: MaterialMode,
        fill_color: Option<[u8; 4]>,
        node_scale: f64,
        name: Option<&str>,
        fallback_name: &str,
    ) -> Result<VoxMain> {
        let input = mesh_input_from_mesh_main(&DependenciesImpl, main)?;

        voxelize_mesh(
            &input,
            counts,
            surface_mode,
            fill_mode,
            material_mode,
            fill_color,
            node_scale,
            name,
            fallback_name,
            OutOfRangeProperty::Error,
        )
    }

    /// The value pool and value id one attribute resolves to on the material
    /// a given voxel samples, through the object's first layer.
    fn voxel_attribute<'a>(
        main: &'a VoxMain,
        position: TyVector3U32,
        attribute: &str,
    ) -> (&'a VoxValuePool, U32Id<BVoxValuePoolValue>) {
        let (_, object) = main.iter_objects().next().unwrap();
        let voxel_id = object.voxel_id(position).unwrap();
        let (layer_id, palette_id) = object.iter_layers().next().unwrap();
        let material_id = object.voxel_material(voxel_id, layer_id).unwrap();
        let palette = main.palette(palette_id).unwrap();
        let property_id = palette.property_id_by_name(attribute).unwrap();
        main.material_value(palette_id, material_id, property_id)
            .unwrap()
    }

    /// The `#RRGGBBAA` hex of the `baseColor` a given voxel samples, encoded
    /// to sRGB from the stored linear color.
    fn voxel_hex(main: &VoxMain, position: TyVector3U32) -> String {
        let (value_pool, value_id) = voxel_attribute(main, position, BASE_COLOR);
        let VoxValuePoolValueRef::Vec4Float(components) = value_pool.value(value_id).unwrap()
        else {
            panic!("baseColor is a four-float color");
        };
        let bytes = <[u8; 4]>::from(srgba_u8_from_lin_srgba_f64(TyLinSrgbaF64::new(
            components[0],
            components[1],
            components[2],
            components[3],
        )));
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            bytes[0], bytes[1], bytes[2], bytes[3]
        )
    }

    /// The numeric value of one float attribute a given voxel samples.
    fn voxel_number(main: &VoxMain, position: TyVector3U32, attribute: &str) -> f64 {
        let (value_pool, value_id) = voxel_attribute(main, position, attribute);
        match value_pool.value(value_id).unwrap() {
            VoxValuePoolValueRef::Float(number) => number,
            other => panic!("{attribute} is a float, not {other:?}"),
        }
    }

    /// The `(r, g, b)` bytes of a `#RRGGBBAA` hex string.
    fn rgb(hex: &str) -> (u8, u8, u8) {
        let byte = |at: usize| u8::from_str_radix(&hex[at..at + 2], 16).unwrap();
        (byte(1), byte(3), byte(5))
    }

    #[test]
    fn flat_paints_the_whole_body_one_color_over_the_material_properties() {
        let main = voxelize(
            &box_main(1.0, 1.0, 4.0, None, None),
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
        assert_eq!(main.validate(), Ok(()));
        assert_eq!(main.object_count(), 1);

        let (_, object) = main.iter_objects().next().unwrap();
        assert_eq!(object.bounds(), TyVector3U32::new(1, 1, 4));
        assert_eq!(object.live_count(), 4);

        // Every mode carries the material properties.
        let (_, palette) = main.iter_palettes().next().unwrap();
        assert_eq!(palette.material_count(), 1);
        assert_eq!(
            palette
                .iter_properties()
                .map(|(_, property)| property.name.as_str())
                .collect::<Vec<_>>(),
            [
                BASE_COLOR,
                METALLIC,
                ROUGHNESS,
                EMISSIVE_COLOR,
                EMISSIVE_STRENGTH,
                OCCLUSION_STRENGTH,
                IOR,
                TRANSMISSION,
            ]
        );
        assert_eq!(voxel_hex(&main, TyVector3U32::new(0, 0, 0)), "#FF0000FF");

        // The root node records the meters-per-voxel scale.
        let root_id = main.root_hierarchy_node_ids()[0];
        assert_eq!(main.hierarchy_node(root_id).unwrap().transform.scale.z, 2.0);
    }

    #[test]
    fn per_primitive_passes_the_factors_through_linear() {
        // The base color factor is linear light, the form the value pool
        // stores, so it imports bit-exact. Its sRGB display encoding is
        // #FFBC00, where a curve applied on import would shift it to #FF8000.
        let main = voxelize(
            &box_main(
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
        assert_eq!(main.validate(), Ok(()));

        let origin = TyVector3U32::new(0, 0, 0);
        let (value_pool, value_id) = voxel_attribute(&main, origin, BASE_COLOR);
        assert_eq!(
            value_pool.value(value_id),
            Some(VoxValuePoolValueRef::Vec4Float(&[1.0, 0.5, 0.0, 1.0]))
        );
        assert_eq!(voxel_hex(&main, origin), "#FFBC00FF");
        assert_eq!(voxel_number(&main, origin, METALLIC), 0.25);
        assert_eq!(voxel_number(&main, origin, ROUGHNESS), 0.75);
        assert_eq!(voxel_number(&main, origin, OCCLUSION_STRENGTH), 1.0);
    }

    #[test]
    fn per_primitive_reads_the_extended_factors_and_the_defaults() {
        let main = voxelize(
            &extended_material_main(1.4, 0.5, 3.0),
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
        assert_eq!(voxel_number(&main, origin, IOR), 1.4);
        assert_eq!(voxel_number(&main, origin, TRANSMISSION), 0.5);
        assert_eq!(voxel_number(&main, origin, EMISSIVE_STRENGTH), 3.0);

        // A default material imports the neutral defaults.
        let main = voxelize(
            &box_main(1.0, 1.0, 1.0, None, None),
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
        assert_eq!(voxel_number(&main, origin, IOR), 1.5);
        assert_eq!(voxel_number(&main, origin, TRANSMISSION), 0.0);
        assert_eq!(voxel_hex(&main, origin), "#FFFFFFFF");
    }

    #[test]
    fn node_transforms_apply_to_the_geometry() {
        // A unit box under a node scaled by two on z rasterizes two voxels
        // tall at one voxel per meter.
        let mut main = MeshMain::default();
        let primitive = box_primitive(1.0, 1.0, 1.0);
        let document = document_of(
            std::mem::take(&mut main),
            primitive,
            None,
            TyTransformF64 {
                scale: TyVector3F64::new(1.0, 1.0, 2.0),
                ..Default::default()
            },
        );

        let input = mesh_input_from_mesh_main(&DependenciesImpl, &document).unwrap();
        let extent = input.extent();
        assert!((extent.z - 2.0).abs() < 1e-9, "z extent {}", extent.z);
        assert!((extent.x - 1.0).abs() < 1e-9, "x extent {}", extent.x);
    }

    /// Voxelizes a unit box and returns the one object's name.
    fn object_name(document: &MeshMain<()>, name: Option<&str>, fallback: &str) -> String {
        let main = voxelize(
            document,
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
        main.iter_objects().next().unwrap().1.name().to_owned()
    }

    #[test]
    fn names_the_object_from_the_document_when_not_overridden() {
        let named = box_main(1.0, 1.0, 1.0, None, Some("Ship"));
        assert_eq!(object_name(&named, None, "stem"), "Ship");
        assert_eq!(object_name(&named, Some("Override"), "stem"), "Override");

        let unnamed = box_main(1.0, 1.0, 1.0, None, None);
        assert_eq!(object_name(&unnamed, None, "stem"), "stem");
    }

    #[test]
    fn per_texel_reads_the_base_color_texture_a_white_factor_hides() {
        // A white base-color factor over a red texture. Per primitive reads
        // only the white factor; per texel samples the texture.
        let png = png_rgba(1, 1, &[[255, 0, 0, 255]]);
        let document = textured_quad_main(&png, [1.0, 1.0, 1.0, 1.0]);

        let per_texel = voxelize(
            &document,
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
        let (_, palette) = per_texel.iter_palettes().next().unwrap();
        assert_eq!(palette.material_count(), 1);
        assert_eq!(
            voxel_hex(&per_texel, TyVector3U32::new(0, 0, 0)),
            "#FF0000FF"
        );

        let per_primitive = voxelize(
            &document,
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
        assert_eq!(
            voxel_hex(&per_primitive, TyVector3U32::new(0, 0, 0)),
            "#FFFFFFFF"
        );

        // Auto samples the texture when there is one.
        let auto = voxelize(
            &document,
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
        assert_eq!(voxel_hex(&auto, TyVector3U32::new(0, 0, 0)), "#FF0000FF");
    }

    #[test]
    fn per_texel_multiplies_the_factor_and_averages_the_footprint() {
        // A white texel under a linear blue factor resolves to blue.
        let png = png_rgba(1, 1, &[[255, 255, 255, 255]]);
        let main = voxelize(
            &textured_quad_main(&png, [0.0, 0.0, 1.0, 1.0]),
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
        assert_eq!(voxel_hex(&main, TyVector3U32::new(0, 0, 0)), "#0000FFFF");

        // A black/white texture under one voxel spanning both texels
        // averages to a gray.
        let png = png_rgba(2, 1, &[[0, 0, 0, 255], [255, 255, 255, 255]]);
        let main = voxelize(
            &textured_quad_main(&png, [1.0, 1.0, 1.0, 1.0]),
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
        let (r, g, b) = rgb(&voxel_hex(&main, TyVector3U32::new(0, 0, 0)));
        assert!(r == g && g == b, "a neutral blend: {r} {g} {b}");
        assert!(r > 0 && r < 255, "a blend, not either extreme: {r}");
    }

    #[test]
    fn per_texel_reads_the_data_maps_with_their_factors() {
        // Metallic in blue and roughness in green, both linear data; the
        // metallic factor scales the blue channel.
        let mr = png_rgba(1, 1, &[[0, 128, 64, 255]]);
        let main = voxelize(
            &pbr_quad_main(
                full_square(),
                full_square(),
                &[MapSpec::MetallicRoughness {
                    png: &mr,
                    stream: 0,
                    metallic: 0.5,
                    roughness: 1.0,
                }],
            ),
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
        let metallic = voxel_number(&main, origin, METALLIC);
        let roughness = voxel_number(&main, origin, ROUGHNESS);
        assert!((metallic - 0.5 * 64.0 / 255.0).abs() < 0.01, "{metallic}");
        assert!((roughness - 128.0 / 255.0).abs() < 0.01, "{roughness}");

        // Occlusion is linear data in red at `1 + strength * (red - 1)`.
        let occlusion = png_rgba(1, 1, &[[128, 0, 0, 255]]);
        let main = voxelize(
            &pbr_quad_main(
                full_square(),
                full_square(),
                &[MapSpec::Occlusion {
                    png: &occlusion,
                    stream: 0,
                    strength: 0.5,
                }],
            ),
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
        let strength = voxel_number(&main, origin, OCCLUSION_STRENGTH);
        assert!((strength - 0.751).abs() < 0.01, "{strength}");

        // Emissive decodes sRGB to linear and keeps the flat strength.
        let emissive = png_rgba(1, 1, &[[0, 188, 0, 255]]);
        let main = voxelize(
            &pbr_quad_main(
                full_square(),
                full_square(),
                &[MapSpec::Emissive {
                    png: &emissive,
                    stream: 0,
                    factor: [1.0, 1.0, 1.0],
                }],
            ),
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
        let (value_pool, value_id) = voxel_attribute(&main, origin, EMISSIVE_COLOR);
        let VoxValuePoolValueRef::Vec3Float(color) = value_pool.value(value_id).unwrap() else {
            panic!("emissiveColor is a three-float color");
        };
        assert!((color[1] - 0.503).abs() < 0.01, "{color:?}");
        assert!(color[0] < 0.001 && color[2] < 0.001, "{color:?}");
        assert_eq!(voxel_number(&main, origin, EMISSIVE_STRENGTH), 1.0);
    }

    #[test]
    fn per_texel_samples_each_maps_own_uv_stream() {
        // Base color reads stream 0 and metallic-roughness reads stream 1,
        // the two streams pointing at different texels.
        let base = png_rgba(2, 1, &[[255, 0, 0, 255], [0, 0, 255, 255]]);
        let mr = png_rgba(2, 1, &[[0, 0, 0, 255], [0, 255, 255, 255]]);
        let main = voxelize(
            &pbr_quad_main(
                [[0.25, 0.5]; 4],
                [[0.75, 0.5]; 4],
                &[
                    MapSpec::BaseColor {
                        png: &base,
                        stream: 0,
                        factor: [1.0, 1.0, 1.0, 1.0],
                    },
                    MapSpec::MetallicRoughness {
                        png: &mr,
                        stream: 1,
                        metallic: 1.0,
                        roughness: 1.0,
                    },
                ],
            ),
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
        assert_eq!(voxel_hex(&main, TyVector3U32::new(0, 0, 0)), "#FF0000FF");
        let metallic = voxel_number(&main, TyVector3U32::new(0, 0, 0), METALLIC);
        assert!((metallic - 1.0).abs() < 1e-9, "{metallic}");
    }

    #[test]
    fn an_undecodable_image_errors() {
        let mut main = MeshMain::default();
        let image_id = main
            .retain_image(MeshImage {
                name: String::new(),
                media_type: MeshImageMediaType::Png,
                source: MeshImageSource::Bytes(vec![
                    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0,
                ]),
            })
            .unwrap();
        let texture_id = main.retain_texture(MeshTexture::new(image_id)).unwrap();
        let material_id = main
            .retain_material(MeshMaterial {
                base_color_texture: Some(MeshTextureRef {
                    texture_id,
                    uv_stream_id: U32Id::from_u32(0),
                }),
                ..Default::default()
            })
            .unwrap();

        let mut primitive = box_primitive(1.0, 1.0, 1.0);
        primitive
            .push_uv_stream(vec![TyVector2F64::ZERO; 8])
            .unwrap();
        primitive.set_material_id(Some(material_id));
        let document = document_of(main, primitive, None, TyTransformF64::default());

        assert!(mesh_input_from_mesh_main(&DependenciesImpl, &document).is_err());
    }
}
