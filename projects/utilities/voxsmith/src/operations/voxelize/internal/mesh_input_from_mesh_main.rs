use crate::{
    Error, Result,
    dependencies::voxelize::DecodeImage,
    operations::voxelize::{MeshInput, MeshTriangle, PlacedPrimitive},
};
use branded_id::U32Id;
use meshdoc::{
    BMeshHierarchyNode, BMeshTexture, MeshExt, MeshMain, MeshMaterial, MeshPrimitive, MeshState,
};
use std::{collections::HashMap, sync::LazyLock};
use ty_math::{TyTransformF64, TyVector3F64};

/// The material a primitive without one draws.
static DEFAULT_MATERIAL: LazyLock<MeshMaterial> = LazyLock::new(MeshMaterial::default);

/// Flattens a mesh document into the world-space triangle mesh the
/// voxelizer rasterizes: every object the hierarchy places, its node
/// transforms applied, so two documents of one object at different scales
/// voxelize alike. Each image a sampled slot draws decodes once through
/// `dependencies`. A normal map or other unsampled image stays encoded. A
/// primitive with no material draws the default material. The mesh takes the
/// name of the first placed object, or of its placing node when the object
/// has none. Errors when an image does not decode.
pub fn mesh_input_from_mesh_main<'a, D: DecodeImage, T: MeshExt>(
    dependencies: &D,
    main: &'a MeshMain<T>,
) -> Result<MeshInput<'a>> {
    let state = main.state();

    let mut walk = Walk {
        dependencies,
        state,
        input: MeshInput {
            primitives: Vec::new(),
            triangles: Vec::new(),
            images: HashMap::new(),
            state,
            name: None,
        },
    };

    for &root_id in state.root_hierarchy_node_ids() {
        walk.node(root_id, &TyTransformF64::default())?;
    }

    Ok(walk.input)
}

/// The state of one flattening walk.
struct Walk<'a, 'd, D> {
    dependencies: &'d D,
    state: &'a MeshState,
    input: MeshInput<'a>,
}

impl<'a, D: DecodeImage> Walk<'a, '_, D> {
    /// Appends the triangles of the objects `node_id` places, in world
    /// space under `parent`, then recurses into its children.
    fn node(&mut self, node_id: U32Id<BMeshHierarchyNode>, parent: &TyTransformF64) -> Result<()> {
        let state = self.state;

        let node = state
            .hierarchy_node(node_id)
            .expect("a root or child is one of the document's nodes");

        let world = parent.compose(&node.transform);

        for &object_id in &node.child_object_ids {
            let object = state
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

    /// Appends a primitive's triangles in world space under `world` and
    /// decodes the images its sampled slots draw.
    fn primitive(&mut self, primitive: &'a MeshPrimitive, world: &TyTransformF64) -> Result<()> {
        let material = match primitive.material_id() {
            Some(material_id) => self
                .state
                .material(material_id)
                .expect("a primitive draws one of the document's materials"),
            None => &DEFAULT_MATERIAL,
        };

        let sampled = [
            material.base_color_texture,
            material.metallic_roughness_texture,
            material.emissive_texture,
            material.occlusion_texture,
        ];

        for texture_ref in sampled.into_iter().flatten() {
            self.decode(texture_ref.texture_id)?;
        }

        let positions: Vec<TyVector3F64> = primitive
            .positions()
            .iter()
            .map(|&position| world.transform_point(position))
            .collect();

        let index = self.input.primitives.len() as u32;

        for triangle in primitive.triangles() {
            self.input.triangles.push(MeshTriangle {
                points: triangle
                    .vertex_ids
                    .map(|vertex_id| positions[vertex_id.to_usize_id().to_usize()]),
                vertex_ids: triangle.vertex_ids,
                primitive: index,
            });
        }

        self.input.primitives.push(PlacedPrimitive {
            primitive,
            material,
        });

        Ok(())
    }

    /// Decodes the image `texture_id` samples, unless an earlier texture
    /// already did.
    fn decode(&mut self, texture_id: U32Id<BMeshTexture>) -> Result<()> {
        let texture = self
            .state
            .texture(texture_id)
            .expect("a material draws one of the document's textures");

        if self.input.images.contains_key(&texture.image_id) {
            return Ok(());
        }

        let source = self
            .state
            .image(texture.image_id)
            .expect("a texture samples one of the document's images");
        let bytes = self
            .state
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

        self.input.images.insert(texture.image_id, decoded);

        Ok(())
    }
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        dependencies::DependenciesImpl,
        operations::voxelize::{
            box_main, box_primitive, document_of, mesh_input_from_mesh_main, png_rgba,
        },
    };
    use branded_id::U32Id;
    use meshdoc::{
        MeshImage, MeshImageMediaType, MeshImageSource, MeshMain, MeshMaterial, MeshTexture,
        MeshTextureRef,
    };
    use ty_math::{TyTransformF64, TyVector2F64, TyVector3F64};

    /// A PNG signature with nothing behind it: the document accepts it and a
    /// decoder rejects it.
    const TRUNCATED_PNG: [u8; 10] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0];

    #[test]
    fn node_transforms_apply_to_the_geometry() {
        // A unit box under a node scaled by two on z spans two meters tall.
        let document = document_of(
            MeshMain::default(),
            box_primitive(1.0, 1.0, 1.0),
            None,
            TyTransformF64 {
                scale: TyVector3F64::new(1.0, 1.0, 2.0),
                ..Default::default()
            },
        );

        let input = mesh_input_from_mesh_main(&DependenciesImpl, &document).unwrap();
        let extent = input.bounds().unwrap().size();
        assert!((extent.z - 2.0).abs() < 1e-9, "z extent {}", extent.z);
        assert!((extent.x - 1.0).abs() < 1e-9, "x extent {}", extent.x);
        assert_eq!(input.triangles.len(), 12);
        assert_eq!(input.primitives.len(), 1);
    }

    #[test]
    fn names_the_mesh_from_the_node_when_the_object_is_unnamed() {
        let named = box_main(1.0, 1.0, 1.0, None, Some("Ship"));
        let input = mesh_input_from_mesh_main(&DependenciesImpl, &named).unwrap();
        assert_eq!(input.name.as_deref(), Some("Ship"));

        let unnamed = box_main(1.0, 1.0, 1.0, None, None);
        let input = mesh_input_from_mesh_main(&DependenciesImpl, &unnamed).unwrap();
        assert_eq!(input.name, None);
    }

    #[test]
    fn a_primitive_without_a_material_draws_the_default() {
        let document = box_main(1.0, 1.0, 1.0, None, None);
        let input = mesh_input_from_mesh_main(&DependenciesImpl, &document).unwrap();
        assert_eq!(*input.primitives[0].material, MeshMaterial::default());
        assert!(!input.is_textured());
    }

    /// A unit box drawing `material_id` over a UV stream of zeros.
    fn textured_box(
        main: MeshMain<()>,
        material_id: U32Id<meshdoc::BMeshMaterial>,
    ) -> MeshMain<()> {
        let mut primitive = box_primitive(1.0, 1.0, 1.0);
        primitive
            .push_uv_stream(vec![TyVector2F64::ZERO; 8])
            .unwrap();
        primitive.set_material_id(Some(material_id));
        document_of(main, primitive, None, TyTransformF64::default())
    }

    #[test]
    fn an_image_two_textures_draw_decodes_once_and_a_normal_map_stays_encoded() {
        let mut main = MeshMain::default();
        let image_of = |main: &mut MeshMain<()>, bytes: Vec<u8>| {
            main.retain_image(MeshImage {
                name: String::new(),
                media_type: MeshImageMediaType::Png,
                source: MeshImageSource::Bytes(bytes),
            })
            .unwrap()
        };
        let shared_id = image_of(&mut main, png_rgba(1, 1, &[[255, 0, 0, 255]]));
        let normal_id = image_of(&mut main, TRUNCATED_PNG.to_vec());

        let texture_of = |main: &mut MeshMain<()>, image_id| MeshTextureRef {
            texture_id: main.retain_texture(MeshTexture::new(image_id)).unwrap(),
            uv_stream_id: U32Id::from_u32(0),
        };
        let material = MeshMaterial {
            base_color_texture: Some(texture_of(&mut main, shared_id)),
            emissive_texture: Some(texture_of(&mut main, shared_id)),
            normal_texture: Some(texture_of(&mut main, normal_id)),
            ..Default::default()
        };
        let material_id = main.retain_material(material).unwrap();

        let document = textured_box(main, material_id);
        let input = mesh_input_from_mesh_main(&DependenciesImpl, &document).unwrap();
        assert_eq!(input.images.len(), 1);
        assert!(input.images.contains_key(&shared_id));
        assert!(input.is_textured());
    }

    #[test]
    fn an_undecodable_image_errors() {
        let mut main = MeshMain::default();
        let image_id = main
            .retain_image(MeshImage {
                name: String::new(),
                media_type: MeshImageMediaType::Png,
                source: MeshImageSource::Bytes(TRUNCATED_PNG.to_vec()),
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

        let document = textured_box(main, material_id);
        assert!(mesh_input_from_mesh_main(&DependenciesImpl, &document).is_err());
    }
}
