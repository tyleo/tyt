use crate::{
    GridResolution, Mesh, ResolutionAxis, Result, VoxelizeOptions, order_palette_colors,
    reduce_palette, voxelize_mesh,
};
use ty_math::{TyVector3F64, TyVector3U32};
use voxcore::VoxMain;

/// Voxelizes `mesh` under `options` into a [`VoxMain`] of one object placed by
/// one root node, its palette reduced when `options.reduction` is set and
/// left canonical: colors in material order, ids compacted. `fallback_name`
/// names the object when neither `options.name` nor the mesh does.
pub fn from_mesh(mesh: &Mesh, fallback_name: &str, options: &VoxelizeOptions) -> Result<VoxMain> {
    let (counts, node_scale) = resolve_grid(mesh.extent(), options.resolution);

    let mut state = voxelize_mesh(
        mesh,
        counts,
        options.surface_mode,
        options.fill_mode,
        options.material_mode,
        options.fill_color,
        node_scale,
        options.name.as_deref(),
        fallback_name,
        options.out_of_range_property,
    )?;

    let palette_id = state
        .iter_palettes()
        .next()
        .map(|(palette_id, _)| palette_id)
        .expect("voxelize_mesh builds one palette");

    if let Some(reduction) = options.reduction {
        reduce_palette(&mut state, palette_id, reduction)?;
    }

    // Canonicalize the generated palette: its materials reference colors in
    // listing order, whatever order voxelize and the reduction left.
    order_palette_colors(&mut state, palette_id);

    // The reduction and the reorder both keep value ids stable, so compact
    // them to listing order: a writer serializes each material cell as an
    // index into the value pool it emits in listing order.
    state.gc();

    Ok(state)
}

/// Resolves the grid counts and the placing node's scale from the mesh `extent`
/// and the chosen `resolution`. `MetersPerVoxel` sizes each axis to a fixed
/// real-world voxel size and records that size as the node scale;
/// `AxisVoxelCount` pins the chosen axis at the given count, sizing the others
/// to preserve aspect, and leaves the node scale at `1`. Every axis is at least
/// one voxel.
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
    use crate::{
        ColorSpace, Dither, FillMode, GridResolution, MaterialMode, Mesh, MeshMaterial,
        MeshTriangle, MeshTriangleUvs, OutOfRangeProperty, PaletteReduction, ReductionMethod,
        ResolutionAxis, SurfaceMode, VoxelizeOptions, from_mesh,
    };
    use ty_math::{TyLinSrgbaF64, TyVector3F64, TyVector3U32};
    use voxcore::material::BASE_COLOR;

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
        // The shortest axis (z, 1 wide) at 4 voxels scales x to 16 and y to 8.
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
        // The y axis (2 wide) at 4 voxels scales x to 8 and z to 2.
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
        // 3 / 2 = 1.5 -> 2 voxels per axis.
        let (counts, node_scale) = resolve_grid(
            TyVector3F64::new(3.0, 4.0, 3.0),
            GridResolution::MetersPerVoxel(2.0),
        );
        assert_eq!(counts, TyVector3U32::new(2, 2, 2));
        assert_eq!(node_scale, 2.0);
    }

    /// A mesh of `colors.len()` unit cells along x, one flat-colored triangle
    /// inside each, so a per-primitive voxelization samples one material per
    /// cell.
    fn cells_mesh(colors: &[TyLinSrgbaF64]) -> Mesh {
        let triangles = (0..colors.len())
            .map(|cell| MeshTriangle {
                points: [
                    TyVector3F64::new(cell as f64 + 0.2, 0.2, 0.5),
                    TyVector3F64::new(cell as f64 + 0.8, 0.2, 0.5),
                    TyVector3F64::new(cell as f64 + 0.5, 0.8, 0.5),
                ],
                uvs: MeshTriangleUvs::default(),
                material_index: cell as u32,
            })
            .collect();

        let materials: Vec<MeshMaterial> = colors
            .iter()
            .map(|&color| MeshMaterial::flat(color))
            .collect();

        Mesh {
            triangles,
            maps: vec![Default::default(); materials.len()],
            materials,
            textures: Vec::new(),
            name: Some("shape".to_owned()),
        }
    }

    /// Per-primitive surface voxelization at one voxel per meter, with
    /// `reduction`.
    fn options(reduction: Option<PaletteReduction>) -> VoxelizeOptions {
        VoxelizeOptions {
            resolution: GridResolution::MetersPerVoxel(1.0),
            surface_mode: SurfaceMode::TriangleCover,
            fill_mode: FillMode::Surface,
            material_mode: MaterialMode::PerPrimitive,
            fill_color: None,
            name: None,
            out_of_range_property: OutOfRangeProperty::Error,
            reduction,
        }
    }

    #[test]
    fn sizes_the_grid_names_the_object_and_keeps_every_material_without_a_reduction() {
        let mesh = cells_mesh(&[
            TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0),
            TyLinSrgbaF64::new(0.0, 1.0, 0.0, 1.0),
            TyLinSrgbaF64::new(0.0, 0.0, 1.0, 1.0),
        ]);

        let state = from_mesh(&mesh, "fallback", &options(None)).unwrap();
        assert_eq!(state.validate(), Ok(()));

        let (_, object) = state.iter_objects().next().unwrap();
        assert_eq!(object.name(), "shape");
        assert_eq!(object.bounds(), TyVector3U32::new(3, 1, 1));
        assert_eq!(object.live_count(), 3);

        let (_, palette) = state.iter_palettes().next().unwrap();
        assert_eq!(palette.material_count(), 3);
    }

    #[test]
    fn a_reduction_caps_the_generated_palette_and_the_state_stays_valid() {
        let mesh = cells_mesh(&[
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

        let state = from_mesh(&mesh, "fallback", &options(Some(reduction))).unwrap();
        assert_eq!(state.validate(), Ok(()));

        let (palette_id, palette) = state.iter_palettes().next().unwrap();
        assert_eq!(palette.material_count(), 2);

        // The merged-away color is pruned and the survivors are compacted to
        // ids `0..2`, in material order.
        let color_property_id = palette.property_id_by_name(BASE_COLOR).unwrap();
        let value_pool_id = palette.property(color_property_id).unwrap().value_pool_id;
        assert_eq!(state.value_pool(value_pool_id).unwrap().len(), 2);
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
        assert_eq!(state.palette(palette_id).unwrap().material_count(), 2);
    }

    #[test]
    fn the_name_override_beats_the_mesh_name_and_the_fallback_fills_in() {
        let mesh = cells_mesh(&[TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0)]);

        let named = VoxelizeOptions {
            name: Some("override".to_owned()),
            ..options(None)
        };
        let state = from_mesh(&mesh, "fallback", &named).unwrap();
        assert_eq!(state.iter_objects().next().unwrap().1.name(), "override");

        let mut nameless = cells_mesh(&[TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0)]);
        nameless.name = None;
        let state = from_mesh(&nameless, "fallback", &options(None)).unwrap();
        assert_eq!(state.iter_objects().next().unwrap().1.name(), "fallback");
    }
}
