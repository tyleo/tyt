use crate::{
    Error, Result,
    dependencies::voxelize::DecodeImage,
    operations::voxelize::{
        GridResolution, MeshInput, ResolutionReference, VoxelizeOptions, mesh_input_from_mesh_main,
        voxelize_mesh,
    },
    utilities::{order_palette_colors, reduce_palette},
};
use meshdoc::{MeshExt, MeshMain};
use ty_math::TyVector3F64;
use voxcore::VoxMain;

/// Voxelizes the mesh document `main` under `options` into a [`VoxMain`] of
/// one object per placed mesh object, all placed on one voxel lattice in
/// world space with their node transforms applied. The objects share one
/// palette, reduced when `options.reduction` is set and left canonical:
/// colors in material order, ids compacted. `dependencies` decodes the
/// document's images for per-texel sampling. Errors when the document places
/// no object, an object has no triangle geometry, or the resolution's
/// reference side has no extent.
pub fn voxelize<D: DecodeImage, T: MeshExt>(
    dependencies: &D,
    main: &MeshMain<T>,
    options: &VoxelizeOptions,
) -> Result<VoxMain> {
    let input = mesh_input_from_mesh_main(dependencies, main)?;

    if input.objects.is_empty() {
        return Err(Error::invalid("mesh places no objects"));
    }

    let voxel_size =
        resolve_voxel_size(&input, options.resolution, options.fallback_name.as_deref())?;

    let mut main = voxelize_mesh(&input, voxel_size, options)?;

    let palette_id = main
        .iter_palettes()
        .next()
        .map(|(palette_id, _)| palette_id)
        .expect("voxelize_mesh builds one palette");

    if let Some(reduction) = options.reduction {
        reduce_palette(&mut main, palette_id, reduction)?;
    }

    // Canonicalize the generated palette: its materials reference colors in
    // listing order, whatever order voxelize and the reduction left.
    order_palette_colors(&mut main, palette_id);

    // The reduction and the reorder both keep value ids stable, so compact
    // them to listing order: a writer serializes each material cell as an
    // index into the value pool it emits in listing order.
    main.gc()?;

    Ok(main)
}

/// The voxel size `resolution` sets over `input`. Errors on a reference side
/// with no extent, or an object with no triangles.
fn resolve_voxel_size(
    input: &MeshInput<'_>,
    resolution: GridResolution,
    fallback_name: Option<&str>,
) -> Result<f64> {
    let GridResolution::ReferenceCount { reference, count } = resolution else {
        let GridResolution::VoxelSize(size) = resolution else {
            unreachable!("a resolution is a size or a count");
        };
        return Ok(size);
    };

    let objects: Vec<TyVector3F64> = input
        .objects
        .iter()
        .map(|object| Ok(input.object_bounds(object, fallback_name)?.size()))
        .collect::<Result<_>>()?;

    let world = input
        .bounds()
        .expect("an object with bounds gives the mesh bounds")
        .size();

    let side = reference_side(world, &objects, reference);

    if side <= 0.0 {
        return Err(Error::invalid(format!(
            "resolution reference {reference:?} has no extent to divide"
        )));
    }

    Ok(side / f64::from(count.max(1)))
}

/// The side `reference` measures over the `world` extent and each object's,
/// or zero when it has none.
fn reference_side(
    world: TyVector3F64,
    objects: &[TyVector3F64],
    reference: ResolutionReference,
) -> f64 {
    let axis = |extent: TyVector3F64, axis: usize| extent.to_array()[axis];
    let longest_of = |extent: TyVector3F64| extent.to_array().into_iter().fold(0.0, f64::max);
    let shortest_of = |extent: TyVector3F64| shortest_positive(extent.to_array());
    let longest_object = |side: &dyn Fn(TyVector3F64) -> f64| {
        objects
            .iter()
            .map(|&extent| side(extent))
            .fold(0.0, f64::max)
    };
    let shortest_object = |side: &dyn Fn(TyVector3F64) -> f64| {
        shortest_positive(objects.iter().map(|&extent| side(extent)))
    };

    match reference {
        ResolutionReference::LongestWorld => longest_of(world),
        ResolutionReference::ShortestWorld => shortest_of(world),
        ResolutionReference::WorldX => axis(world, 0),
        ResolutionReference::WorldY => axis(world, 1),
        ResolutionReference::WorldZ => axis(world, 2),
        ResolutionReference::LongestObject => longest_object(&longest_of),
        ResolutionReference::ShortestObject => shortest_object(&shortest_of),
        ResolutionReference::LongestObjectX => longest_object(&|extent| axis(extent, 0)),
        ResolutionReference::LongestObjectY => longest_object(&|extent| axis(extent, 1)),
        ResolutionReference::LongestObjectZ => longest_object(&|extent| axis(extent, 2)),
        ResolutionReference::ShortestObjectX => shortest_object(&|extent| axis(extent, 0)),
        ResolutionReference::ShortestObjectY => shortest_object(&|extent| axis(extent, 1)),
        ResolutionReference::ShortestObjectZ => shortest_object(&|extent| axis(extent, 2)),
    }
}

/// The smallest positive side, or zero when none is positive.
fn shortest_positive(sides: impl IntoIterator<Item = f64>) -> f64 {
    sides
        .into_iter()
        .filter(|&side| side > 0.0)
        .fold(0.0, |shortest, side| {
            if shortest > 0.0 {
                shortest.min(side)
            } else {
                side
            }
        })
}

#[cfg(test)]
mod tests {
    use super::reference_side;
    use crate::operations::voxelize::ResolutionReference;
    use ty_math::TyVector3F64;

    /// The extent `(x, y, z)`.
    fn extent(x: f64, y: f64, z: f64) -> TyVector3F64 {
        TyVector3F64::new(x, y, z)
    }

    /// Every reference's side over a world of `[3, 2, 0]` holding objects of
    /// `[3, 1, 0]` and `[2, 2, 0]`.
    fn side(reference: ResolutionReference) -> f64 {
        reference_side(
            extent(3.0, 2.0, 0.0),
            &[extent(3.0, 1.0, 0.0), extent(2.0, 2.0, 0.0)],
            reference,
        )
    }

    #[test]
    fn world_references_measure_the_whole_and_skip_flat_sides_for_shortest() {
        assert_eq!(side(ResolutionReference::LongestWorld), 3.0);
        assert_eq!(side(ResolutionReference::ShortestWorld), 2.0);
        assert_eq!(side(ResolutionReference::WorldX), 3.0);
        assert_eq!(side(ResolutionReference::WorldY), 2.0);
        assert_eq!(side(ResolutionReference::WorldZ), 0.0);
    }

    #[test]
    fn object_references_take_the_extreme_across_objects() {
        assert_eq!(side(ResolutionReference::LongestObject), 3.0);
        assert_eq!(side(ResolutionReference::ShortestObject), 1.0);
        assert_eq!(side(ResolutionReference::LongestObjectX), 3.0);
        assert_eq!(side(ResolutionReference::ShortestObjectX), 2.0);
        assert_eq!(side(ResolutionReference::LongestObjectY), 2.0);
        assert_eq!(side(ResolutionReference::ShortestObjectY), 1.0);
        assert_eq!(side(ResolutionReference::LongestObjectZ), 0.0);
        assert_eq!(side(ResolutionReference::ShortestObjectZ), 0.0);
    }
}

#[cfg(all(test, feature = "impl"))]
mod document_tests {
    use crate::{
        dependencies::DependenciesImpl,
        operations::voxelize::{
            FillMode, GridResolution, MapSpec, MaterialMode, OutOfRangeProperty,
            ResolutionReference, SurfaceMode, VoxelizeOptions, box_main, box_primitive,
            document_of, full_square, pbr_quad_main, png_rgba, textured_quad_main, triangle_of,
            voxel_attribute, voxel_hex, voxel_number, voxelize,
        },
        utilities::{ColorSpace, Dither, PaletteReduction, ReductionMethod},
    };
    use meshdoc::{MeshHierarchyNode, MeshMain, MeshMaterial, MeshObject, MeshPrimitive};
    use ty_math::{
        TyLinSrgbF64, TyLinSrgbaF64, TyTransformF64, TyVector3F64, TyVector3I32, TyVector3U32,
    };
    use voxcore::{
        VoxMain, VoxValuePoolValueRef,
        material::{
            BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC, OCCLUSION_STRENGTH,
            ROUGHNESS, TRANSMISSION,
        },
    };

    /// Options at `meters` per voxel under the given modes, everything else
    /// off.
    fn options(
        meters: f64,
        surface_mode: SurfaceMode,
        fill_mode: FillMode,
        material_mode: MaterialMode,
    ) -> VoxelizeOptions {
        VoxelizeOptions {
            resolution: GridResolution::VoxelSize(meters),
            surface_mode,
            fill_mode,
            material_mode,
            fill_color: None,
            fallback_name: Some("voxelized".to_owned()),
            out_of_range_property: OutOfRangeProperty::Error,
            reduction: None,
        }
    }

    /// Solid, center-inside options at one voxel per meter under
    /// `material_mode`.
    fn solid(material_mode: MaterialMode) -> VoxelizeOptions {
        options(
            1.0,
            SurfaceMode::CenterInside,
            FillMode::Solid,
            material_mode,
        )
    }

    /// Hollow, triangle-cover options at `meters` per voxel under
    /// `material_mode`.
    fn shell(meters: f64, material_mode: MaterialMode) -> VoxelizeOptions {
        options(
            meters,
            SurfaceMode::TriangleCover,
            FillMode::Surface,
            material_mode,
        )
    }

    /// Voxelizes `document` under `options`.
    fn run(document: &MeshMain<()>, options: &VoxelizeOptions) -> VoxMain {
        let main = voxelize(&DependenciesImpl, document, options).unwrap();
        assert_eq!(main.validate(), Ok(()));
        main
    }

    /// The `(r, g, b)` bytes of a `#RRGGBBAA` hex string.
    fn rgb(hex: &str) -> (u8, u8, u8) {
        let byte = |at: usize| u8::from_str_radix(&hex[at..at + 2], 16).unwrap();
        (byte(1), byte(3), byte(5))
    }

    #[test]
    fn flat_paints_the_whole_body_one_color_over_the_material_properties() {
        let main = run(
            &box_main(2.0, 2.0, 8.0, None, None),
            &VoxelizeOptions {
                resolution: GridResolution::VoxelSize(2.0),
                fill_color: Some([255, 0, 0, 255]),
                ..solid(MaterialMode::Flat)
            },
        );
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
        let main = run(
            &box_main(
                1.0,
                1.0,
                1.0,
                Some(([1.0, 0.5, 0.0, 1.0], 0.25, 0.75)),
                None,
            ),
            &solid(MaterialMode::PerPrimitive),
        );

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
        // A unit box whose material carries the three factors the voxelizer
        // reads beyond the metallic-roughness set, over a green emissive.
        let mut extended = MeshMain::default();
        let material_id = extended
            .retain_material(MeshMaterial {
                base_color_factor: TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0),
                emissive_factor: TyLinSrgbF64::new(0.0, 1.0, 0.0),
                emissive_strength: 3.0,
                ior: 1.4,
                transmission_factor: 0.5,
                ..Default::default()
            })
            .unwrap();
        let mut primitive = box_primitive(1.0, 1.0, 1.0);
        primitive.set_material_id(Some(material_id));
        let extended = document_of(extended, primitive, None, TyTransformF64::default());

        let main = run(&extended, &solid(MaterialMode::PerPrimitive));
        let origin = TyVector3U32::new(0, 0, 0);
        assert_eq!(voxel_number(&main, origin, IOR), 1.4);
        assert_eq!(voxel_number(&main, origin, TRANSMISSION), 0.5);
        assert_eq!(voxel_number(&main, origin, EMISSIVE_STRENGTH), 3.0);

        // A default material imports the neutral defaults.
        let main = run(
            &box_main(1.0, 1.0, 1.0, None, None),
            &solid(MaterialMode::PerPrimitive),
        );
        assert_eq!(voxel_number(&main, origin, IOR), 1.5);
        assert_eq!(voxel_number(&main, origin, TRANSMISSION), 0.0);
        assert_eq!(voxel_hex(&main, origin), "#FFFFFFFF");
    }

    /// The one object's name after voxelizing `document`.
    fn object_name(document: &MeshMain<()>) -> String {
        let main = run(document, &solid(MaterialMode::Flat));
        main.iter_objects().next().unwrap().1.name().to_owned()
    }

    #[test]
    fn the_object_name_falls_back_to_the_node_then_the_option() {
        let mut main = MeshMain::default();
        let mut object = MeshObject::new("Hull".to_owned());
        object.retain_primitive(box_primitive(1.0, 1.0, 1.0));
        let object_id = main.retain_object(object).unwrap();
        let node_id = main
            .retain_hierarchy_node(MeshHierarchyNode {
                name: "Ship".to_owned(),
                child_object_ids: vec![object_id],
                ..Default::default()
            })
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![node_id]).unwrap();
        assert_eq!(object_name(&main), "Hull");

        let node_named = box_main(1.0, 1.0, 1.0, None, Some("Ship"));
        assert_eq!(object_name(&node_named), "Ship");

        let unnamed = box_main(1.0, 1.0, 1.0, None, None);
        assert_eq!(object_name(&unnamed), "voxelized");

        let main = run(
            &unnamed,
            &VoxelizeOptions {
                fallback_name: None,
                ..solid(MaterialMode::Flat)
            },
        );
        assert_eq!(main.iter_objects().next().unwrap().1.name(), "");
    }

    /// A document of one `Crate` box of `side` meters placed by two unnamed
    /// nodes, at the origin and translated by `offset` along x.
    fn two_crates(side: f64, offset: f64) -> MeshMain<()> {
        let mut main = MeshMain::default();
        let mut object = MeshObject::new("Crate".to_owned());
        object.retain_primitive(box_primitive(side, side, side));
        let object_id = main.retain_object(object).unwrap();

        let node_of = |main: &mut MeshMain<()>, x: f64| {
            main.retain_hierarchy_node(MeshHierarchyNode {
                transform: TyTransformF64 {
                    position: TyVector3F64::new(x, 0.0, 0.0),
                    ..Default::default()
                },
                child_object_ids: vec![object_id],
                ..Default::default()
            })
            .unwrap()
        };
        let left = node_of(&mut main, 0.0);
        let right = node_of(&mut main, offset);
        main.set_root_hierarchy_node_ids(vec![left, right]).unwrap();
        main.validate().unwrap();
        main
    }

    #[test]
    fn each_placement_becomes_one_object_on_a_shared_lattice_and_palette() {
        let main = run(
            &two_crates(1.0, 4.5),
            &VoxelizeOptions {
                resolution: GridResolution::ReferenceCount {
                    reference: ResolutionReference::LongestObject,
                    count: 2,
                },
                ..solid(MaterialMode::Flat)
            },
        );

        assert_eq!(main.object_count(), 2);
        assert_eq!(main.palette_count(), 1);
        assert_eq!(main.root_hierarchy_node_ids().len(), 2);

        // Half-meter cubes: the second crate spans `[4.5, 5.5]`, lattice
        // cells 9 and 10.
        let objects: Vec<_> = main.iter_objects().map(|(_, object)| object).collect();
        assert_eq!(objects[0].origin(), TyVector3I32::new(0, 0, 0));
        assert_eq!(objects[0].bounds(), TyVector3U32::new(2, 2, 2));
        assert_eq!(objects[0].live_count(), 8);
        assert_eq!(objects[1].origin(), TyVector3I32::new(9, 0, 0));
        assert_eq!(objects[1].bounds(), TyVector3U32::new(2, 2, 2));
        assert_eq!(objects[1].name(), "Crate");

        let (palette_id, _) = main.iter_palettes().next().unwrap();
        for object in &objects {
            let (_, layer_palette_id) = object.iter_layers().next().unwrap();
            assert_eq!(layer_palette_id, palette_id);
        }

        for &root_id in main.root_hierarchy_node_ids() {
            let node = main.hierarchy_node(root_id).unwrap();
            assert_eq!(node.transform.scale, TyVector3F64::splat(0.5));
            assert_eq!(node.transform.position, TyVector3F64::ZERO);
        }
    }

    #[test]
    fn a_world_reference_spans_every_object() {
        // Two unit crates 4 meters apart span 5 meters, so 10 voxels across
        // are half a meter each.
        let main = run(
            &two_crates(1.0, 4.0),
            &VoxelizeOptions {
                resolution: GridResolution::ReferenceCount {
                    reference: ResolutionReference::WorldX,
                    count: 10,
                },
                ..solid(MaterialMode::Flat)
            },
        );

        let (_, object) = main.iter_objects().next().unwrap();
        assert_eq!(object.bounds(), TyVector3U32::new(2, 2, 2));
    }

    #[test]
    fn an_object_without_triangles_errors_with_its_name() {
        let mut main: MeshMain<()> = MeshMain::default();
        let object_id = main
            .retain_object(MeshObject::new("Empty".to_owned()))
            .unwrap();
        let node_id = main
            .retain_hierarchy_node(MeshHierarchyNode {
                child_object_ids: vec![object_id],
                ..Default::default()
            })
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![node_id]).unwrap();

        let error = voxelize(&DependenciesImpl, &main, &solid(MaterialMode::Flat)).unwrap_err();
        assert!(error.to_string().contains("Empty"), "{error}");

        let empty: MeshMain<()> = MeshMain::default();
        let error = voxelize(&DependenciesImpl, &empty, &solid(MaterialMode::Flat)).unwrap_err();
        assert!(error.to_string().contains("no objects"), "{error}");
    }

    #[test]
    fn per_texel_reads_the_base_color_texture_a_white_factor_hides() {
        // A white base-color factor over a red texture. Per primitive reads
        // only the white factor; per texel samples the texture.
        let png = png_rgba(1, 1, &[[255, 0, 0, 255]]);
        let document = textured_quad_main(&png, [1.0, 1.0, 1.0, 1.0]);

        let per_texel = run(&document, &shell(0.5, MaterialMode::PerTexel));
        let (_, object) = per_texel.iter_objects().next().unwrap();
        assert_eq!(object.bounds(), TyVector3U32::new(2, 1, 2));
        let (_, palette) = per_texel.iter_palettes().next().unwrap();
        assert_eq!(palette.material_count(), 1);
        assert_eq!(
            voxel_hex(&per_texel, TyVector3U32::new(0, 0, 0)),
            "#FF0000FF"
        );

        let per_primitive = run(&document, &shell(0.5, MaterialMode::PerPrimitive));
        assert_eq!(
            voxel_hex(&per_primitive, TyVector3U32::new(0, 0, 0)),
            "#FFFFFFFF"
        );

        // Auto samples the texture when there is one.
        let auto = run(&document, &shell(1.0, MaterialMode::Auto));
        assert_eq!(voxel_hex(&auto, TyVector3U32::new(0, 0, 0)), "#FF0000FF");
    }

    #[test]
    fn per_texel_multiplies_the_factor_and_averages_the_footprint() {
        // A white texel under a linear blue factor resolves to blue.
        let png = png_rgba(1, 1, &[[255, 255, 255, 255]]);
        let main = run(
            &textured_quad_main(&png, [0.0, 0.0, 1.0, 1.0]),
            &shell(1.0, MaterialMode::PerTexel),
        );
        assert_eq!(voxel_hex(&main, TyVector3U32::new(0, 0, 0)), "#0000FFFF");

        // A black/white texture under one voxel spanning both texels
        // averages to a gray.
        let png = png_rgba(2, 1, &[[0, 0, 0, 255], [255, 255, 255, 255]]);
        let main = run(
            &textured_quad_main(&png, [1.0, 1.0, 1.0, 1.0]),
            &shell(1.0, MaterialMode::PerTexel),
        );
        let (r, g, b) = rgb(&voxel_hex(&main, TyVector3U32::new(0, 0, 0)));
        assert!(r == g && g == b, "a neutral blend: {r} {g} {b}");
        assert!(r > 0 && r < 255, "a blend, not either extreme: {r}");
    }

    #[test]
    fn per_texel_reads_the_data_maps_with_their_factors() {
        let origin = TyVector3U32::new(0, 0, 0);

        // Metallic in blue and roughness in green, both linear data; the
        // metallic factor scales the blue channel.
        let mr = png_rgba(1, 1, &[[0, 128, 64, 255]]);
        let main = run(
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
            &shell(1.0, MaterialMode::PerTexel),
        );
        let metallic = voxel_number(&main, origin, METALLIC);
        let roughness = voxel_number(&main, origin, ROUGHNESS);
        assert!((metallic - 0.5 * 64.0 / 255.0).abs() < 0.01, "{metallic}");
        assert!((roughness - 128.0 / 255.0).abs() < 0.01, "{roughness}");

        // Occlusion is linear data in red at `1 + strength * (red - 1)`.
        let occlusion = png_rgba(1, 1, &[[128, 0, 0, 255]]);
        let main = run(
            &pbr_quad_main(
                full_square(),
                full_square(),
                &[MapSpec::Occlusion {
                    png: &occlusion,
                    stream: 0,
                    strength: 0.5,
                }],
            ),
            &shell(1.0, MaterialMode::PerTexel),
        );
        let strength = voxel_number(&main, origin, OCCLUSION_STRENGTH);
        assert!((strength - 0.751).abs() < 0.01, "{strength}");

        // Emissive decodes sRGB to linear and keeps the flat strength.
        let emissive = png_rgba(1, 1, &[[0, 188, 0, 255]]);
        let main = run(
            &pbr_quad_main(
                full_square(),
                full_square(),
                &[MapSpec::Emissive {
                    png: &emissive,
                    stream: 0,
                    factor: [1.0, 1.0, 1.0],
                }],
            ),
            &shell(1.0, MaterialMode::PerTexel),
        );
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
        let main = run(
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
            &shell(1.0, MaterialMode::PerTexel),
        );
        assert_eq!(voxel_hex(&main, TyVector3U32::new(0, 0, 0)), "#FF0000FF");
        let metallic = voxel_number(&main, TyVector3U32::new(0, 0, 0), METALLIC);
        assert!((metallic - 1.0).abs() < 1e-9, "{metallic}");
    }

    /// A document of one flat-colored triangle primitive per entry of
    /// `colors`, each inside its own unit cell along x, under a node named
    /// `shape`.
    fn cells_main(colors: &[TyLinSrgbaF64]) -> MeshMain<()> {
        let mut main = MeshMain::default();
        let mut object = MeshObject::new(String::new());

        for (cell, &color) in colors.iter().enumerate() {
            let material_id = main
                .retain_material(MeshMaterial {
                    base_color_factor: color,
                    metallic_factor: 0.0,
                    ..Default::default()
                })
                .unwrap();

            let x = cell as f64;
            let mut primitive = MeshPrimitive::new(
                vec![
                    TyVector3F64::new(x + 0.2, 0.2, 0.5),
                    TyVector3F64::new(x + 0.8, 0.2, 0.5),
                    TyVector3F64::new(x + 0.5, 0.8, 0.5),
                ],
                vec![triangle_of(0, 1, 2)],
            )
            .unwrap();
            primitive.set_material_id(Some(material_id));
            object.retain_primitive(primitive);
        }

        let object_id = main.retain_object(object).unwrap();
        let node_id = main
            .retain_hierarchy_node(MeshHierarchyNode {
                name: "shape".to_owned(),
                child_object_ids: vec![object_id],
                ..Default::default()
            })
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![node_id]).unwrap();
        main.validate().unwrap();
        main
    }

    #[test]
    fn sizes_the_grid_names_the_object_and_keeps_every_material_without_a_reduction() {
        let document = cells_main(&[
            TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0),
            TyLinSrgbaF64::new(0.0, 1.0, 0.0, 1.0),
            TyLinSrgbaF64::new(0.0, 0.0, 1.0, 1.0),
        ]);

        let main = run(&document, &shell(1.0, MaterialMode::PerPrimitive));

        let (_, object) = main.iter_objects().next().unwrap();
        assert_eq!(object.name(), "shape");
        assert_eq!(object.bounds(), TyVector3U32::new(3, 1, 1));
        assert_eq!(object.live_count(), 3);

        let (_, palette) = main.iter_palettes().next().unwrap();
        assert_eq!(palette.material_count(), 3);
    }

    #[test]
    fn a_reduction_caps_the_generated_palette_and_the_state_stays_valid() {
        let document = cells_main(&[
            TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0),
            TyLinSrgbaF64::new(0.99, 0.0, 0.0, 1.0),
            TyLinSrgbaF64::new(0.0, 0.0, 1.0, 1.0),
        ]);

        let reduction = PaletteReduction {
            max_materials: 2,
            method: ReductionMethod::MedianCut,
            space: ColorSpace::Oklab,
            dither: Dither::None,
            keep_unused_values: false,
        };

        let main = run(
            &document,
            &VoxelizeOptions {
                reduction: Some(reduction),
                ..shell(1.0, MaterialMode::PerPrimitive)
            },
        );

        let (palette_id, palette) = main.iter_palettes().next().unwrap();
        assert_eq!(palette.material_count(), 2);

        // The merged-away color is pruned and the survivors are compacted to
        // ids `0..2`, in material order.
        let color_property_id = palette.property_id_by_name(BASE_COLOR).unwrap();
        let value_pool_id = palette.property(color_property_id).unwrap().value_pool_id;
        assert_eq!(main.value_pool(value_pool_id).unwrap().len(), 2);
        let value_ids: Vec<u32> = palette
            .iter_materials()
            .map(|material_id| {
                palette
                    .value_id(material_id, color_property_id)
                    .unwrap()
                    .to_u32()
            })
            .collect();
        assert_eq!(value_ids, [0, 1]);
        assert_eq!(main.palette(palette_id).unwrap().material_count(), 2);
    }
}
