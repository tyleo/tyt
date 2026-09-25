use crate::{
    Result,
    dependencies::voxelize::DecodeImage,
    operations::voxelize::{
        GridResolution, ResolutionAxis, VoxelizeOptions, mesh_input_from_mesh_main, voxelize_mesh,
    },
    utilities::{order_palette_colors, reduce_palette},
};
use meshdoc::{MeshExt, MeshMain};
use ty_math::{TyVector3F64, TyVector3U32};
use voxcore::VoxMain;

/// Voxelizes the mesh document `main` under `options` into a [`VoxMain`] of
/// one object placed by one root node, its palette reduced when
/// `options.reduction` is set and left canonical: colors in material order,
/// ids compacted. Every object the hierarchy places rasterizes in world
/// space, its node transforms applied. `dependencies` decodes the
/// document's images for per-texel sampling. `fallback_name` names the
/// object when neither `options.name` nor the document does.
pub fn voxelize<D: DecodeImage, T: MeshExt>(
    dependencies: &D,
    main: &MeshMain<T>,
    fallback_name: &str,
    options: &VoxelizeOptions,
) -> Result<VoxMain> {
    let input = mesh_input_from_mesh_main(dependencies, main)?;

    let (counts, node_scale) = resolve_grid(input.extent(), options.resolution);

    let mut main = voxelize_mesh(&input, counts, node_scale, fallback_name, options)?;

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

/// The grid counts and the placing node's scale for the mesh `extent` at
/// `resolution`.
fn resolve_grid(extent: TyVector3F64, resolution: GridResolution) -> (TyVector3U32, f64) {
    match resolution {
        GridResolution::MetersPerVoxel(meters) => {
            let count = |edge: f64| (edge / meters).ceil().max(1.0) as u32;

            let counts = TyVector3U32::new(count(extent.x), count(extent.y), count(extent.z));

            (counts, meters)
        }

        GridResolution::AxisVoxelCount { axis, count } => {
            let n = count.max(1) as f64;

            let reference = match axis {
                ResolutionAxis::Long => extent.x.max(extent.y).max(extent.z),
                ResolutionAxis::Short => extent.x.min(extent.y).min(extent.z),
                ResolutionAxis::X => extent.x,
                ResolutionAxis::Y => extent.y,
                ResolutionAxis::Z => extent.z,
            };

            let count = |edge: f64| {
                if reference > 0.0 {
                    (edge / reference * n).round().max(1.0) as u32
                } else {
                    1
                }
            };

            let counts = TyVector3U32::new(count(extent.x), count(extent.y), count(extent.z));

            (counts, 1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_grid;
    use crate::operations::voxelize::{GridResolution, ResolutionAxis};
    use ty_math::{TyVector3F64, TyVector3U32};

    #[test]
    fn axis_voxel_count_sizes_the_longest_axis_and_preserves_aspect() {
        let (counts, node_scale) = resolve_grid(
            TyVector3F64::new(4.0, 2.0, 1.0),
            GridResolution::AxisVoxelCount {
                axis: ResolutionAxis::Long,
                count: 4,
            },
        );
        assert_eq!(counts, TyVector3U32::new(4, 2, 1));
        assert_eq!(node_scale, 1.0);
    }

    #[test]
    fn axis_voxel_count_sizes_the_shortest_axis() {
        let (counts, _) = resolve_grid(
            TyVector3F64::new(4.0, 2.0, 1.0),
            GridResolution::AxisVoxelCount {
                axis: ResolutionAxis::Short,
                count: 4,
            },
        );
        assert_eq!(counts, TyVector3U32::new(16, 8, 4));
    }

    #[test]
    fn axis_voxel_count_sizes_a_named_axis() {
        let (counts, _) = resolve_grid(
            TyVector3F64::new(4.0, 2.0, 1.0),
            GridResolution::AxisVoxelCount {
                axis: ResolutionAxis::Y,
                count: 4,
            },
        );
        assert_eq!(counts, TyVector3U32::new(8, 4, 2));
    }

    #[test]
    fn axis_voxel_count_keeps_a_zero_axis_at_one_voxel() {
        let (counts, _) = resolve_grid(
            TyVector3F64::new(4.0, 0.0, 2.0),
            GridResolution::AxisVoxelCount {
                axis: ResolutionAxis::Long,
                count: 4,
            },
        );
        assert_eq!(counts, TyVector3U32::new(4, 1, 2));
    }

    #[test]
    fn meters_per_voxel_rounds_each_axis_up_and_records_the_size() {
        let (counts, node_scale) = resolve_grid(
            TyVector3F64::new(3.0, 4.0, 3.0),
            GridResolution::MetersPerVoxel(2.0),
        );
        assert_eq!(counts, TyVector3U32::new(2, 2, 2));
        assert_eq!(node_scale, 2.0);
    }
}

#[cfg(all(test, feature = "impl"))]
mod document_tests {
    use crate::{
        dependencies::DependenciesImpl,
        operations::voxelize::{
            FillMode, GridResolution, MapSpec, MaterialMode, OutOfRangeProperty, SurfaceMode,
            VoxelizeOptions, box_main, box_primitive, document_of, full_square, pbr_quad_main,
            png_rgba, textured_quad_main, triangle_of, voxel_attribute, voxel_hex, voxel_number,
            voxelize,
        },
        utilities::{ColorSpace, Dither, PaletteReduction, ReductionMethod},
    };
    use meshdoc::{MeshHierarchyNode, MeshMain, MeshMaterial, MeshObject, MeshPrimitive};
    use ty_math::{TyLinSrgbF64, TyLinSrgbaF64, TyTransformF64, TyVector3F64, TyVector3U32};
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
            resolution: GridResolution::MetersPerVoxel(meters),
            surface_mode,
            fill_mode,
            material_mode,
            fill_color: None,
            name: None,
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

    /// Voxelizes `document` under `options` with the fallback name
    /// `voxelized`.
    fn run(document: &MeshMain<()>, options: &VoxelizeOptions) -> VoxMain {
        let main = voxelize(&DependenciesImpl, document, "voxelized", options).unwrap();
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
                resolution: GridResolution::MetersPerVoxel(2.0),
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

    /// The one object's name after voxelizing `document` under a `name`
    /// override.
    fn object_name(document: &MeshMain<()>, name: Option<&str>) -> String {
        let main = run(
            document,
            &VoxelizeOptions {
                name: name.map(str::to_owned),
                ..solid(MaterialMode::Flat)
            },
        );
        main.iter_objects().next().unwrap().1.name().to_owned()
    }

    #[test]
    fn the_name_override_beats_the_mesh_name_and_the_fallback_fills_in() {
        let named = box_main(1.0, 1.0, 1.0, None, Some("Ship"));
        assert_eq!(object_name(&named, None), "Ship");
        assert_eq!(object_name(&named, Some("Override")), "Override");

        let unnamed = box_main(1.0, 1.0, 1.0, None, None);
        assert_eq!(object_name(&unnamed, None), "voxelized");
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
