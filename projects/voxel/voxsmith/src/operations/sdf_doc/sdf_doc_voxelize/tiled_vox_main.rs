use crate::{
    Error, Result,
    dependencies::sdf_doc::MapCells,
    operations::sdf_doc::{
        CheckedModel, ObjectCells, ObjectSampler, PlaceFrame, PlaceNode, SdfGrid,
        SdfMaterialProperties, SdfTileOptions, assemble_vox_main, is_exposed, placed_nodes,
        see_through_materials,
    },
    utilities::{FillMode, VoxelFrame},
};
use branded_id::{IdSlice, IdVec, U32Id};
use sdfcore::{BSdfMaterial, SdfMain};
use ty_math::{TyPoseF64, TyVector3I32, TyVector3U32};
use voxcore::VoxMain;

/// The widest tile, whose 512^3 cells match the untiled grid cap.
const MOST_TILE: u32 = 512;

/// The voxj document of the model `main`, sampled one tile at a time under
/// `options`. Each tile that keeps a cell writes an object. The objects hold
/// the cells an untiled world-frame run writes. `dependencies` spreads each
/// step's cells across threads. Errors on a tile outside 1 to 512 cells and on
/// the first failed check.
pub fn tiled_vox_main<D: MapCells>(
    dependencies: &D,
    main: &SdfMain,
    options: &SdfTileOptions,
) -> Result<VoxMain> {
    let parts = tiled_parts(dependencies, main, options)?;

    assemble_vox_main(parts.objects, &parts.nodes, &parts.materials)
}

/// The objects and nodes of a tiled document.
struct TiledParts {
    objects: Vec<ObjectCells>,

    nodes: Vec<PlaceNode>,

    /// The properties of each material of the model.
    materials: IdVec<BSdfMaterial, SdfMaterialProperties>,
}

/// Samples the model `main` for [`tiled_vox_main`], which lists the errors.
fn tiled_parts<D: MapCells>(
    dependencies: &D,
    main: &SdfMain,
    options: &SdfTileOptions,
) -> Result<TiledParts> {
    if !(1..=MOST_TILE).contains(&options.tile) {
        return Err(Error::invalid(format!(
            "tile must span 1 to {MOST_TILE} cells, not {}",
            options.tile
        )));
    }

    let state = main.state();
    let CheckedModel {
        places,
        materials,
        shapes,
        voxel_size,
    } = CheckedModel::new(main, options.resolution)?;
    let see_through = see_through_materials(&materials);
    let frames = PlaceFrame::of_places(&places, false);
    let mut objects = Vec::new();

    let nodes = placed_nodes(
        &places,
        VoxelFrame::World,
        voxel_size,
        |index, lattice_offset| {
            let place = &places[index];
            let frame = frames[index].sampling();
            let pose = TyPoseF64::new(
                frame.position + frame.rotation * place.offset,
                frame.rotation,
            );
            let name = place.path.last().expect("a place has a part");
            let first = objects.len();

            for &object_id in &state.nodes[place.node_id.to_usize_id()].child_object_ids {
                let sampler = ObjectSampler::new(
                    state,
                    &shapes,
                    object_id,
                    &place.path,
                    voxel_size,
                    pose,
                    false,
                );

                for_each_tile(dependencies, &sampler, options, &see_through, |cells| {
                    objects.extend(ObjectCells::new(name.clone(), &cells, lattice_offset));
                })?;
            }

            Ok((first..objects.len()).collect())
        },
    )?;

    if objects.is_empty() {
        return Err(Error::invalid(
            "model must hold a live cell, not only empty grids",
        ));
    }

    Ok(TiledParts {
        objects,
        nodes,
        materials,
    })
}

/// Samples each tile of `sampler`'s object and passes its kept cells to
/// `keep`. Tiles run x outermost from the least corner of the object's `add`
/// and `set` boxes. Tiles outside every box are skipped. Under surface fill a
/// tile also samples a one-cell border, so cells on a seam see their
/// neighbors. Errors on the first failed step.
fn for_each_tile<D: MapCells>(
    dependencies: &D,
    sampler: &ObjectSampler,
    options: &SdfTileOptions,
    see_through: &IdSlice<BSdfMaterial, bool>,
    mut keep: impl FnMut(Vec<(TyVector3I32, U32Id<BSdfMaterial>)>),
) -> Result<()> {
    let mut ran = false;

    if let Some(reach) = sampler.reach() {
        let (min, max) = (
            sampler.lattice_cell(reach.min)?,
            sampler.lattice_cell(reach.max)?,
        );
        let halo = TyVector3I32::splat(match options.fill_mode {
            FillMode::Solid => 0,
            FillMode::Surface => 1,
        });
        let step = options.tile as usize;

        for x in (min.x..max.x).step_by(step) {
            for y in (min.y..max.y).step_by(step) {
                for z in (min.z..max.z).step_by(step) {
                    let tile_min = TyVector3I32::new(x, y, z);
                    let tile_max = tile_min
                        .saturating_add(TyVector3I32::splat(options.tile as i32))
                        .min(max);

                    if !sampler.reaches(tile_min, tile_max) {
                        continue;
                    }

                    let grid_min = tile_min.saturating_sub(halo).max(min);
                    let grid_max = tile_max.saturating_add(halo).min(max);
                    let grid =
                        sampler.sample(dependencies, grid_min, (grid_max - grid_min).as_uvec3())?;
                    ran = true;

                    keep(kept_cells(
                        &grid,
                        tile_min,
                        tile_max,
                        options.fill_mode,
                        see_through,
                    ));
                }
            }
        }
    }

    // An object with no tiles still runs its steps on an empty grid, so their
    // argument checks fail as they do untiled.
    if !ran {
        sampler.sample(dependencies, TyVector3I32::ZERO, TyVector3U32::ZERO)?;
    }

    Ok(())
}

/// The live cells of `grid` from `tile_min` up to `tile_max` that `fill_mode`
/// keeps, in a raster with x outermost.
fn kept_cells(
    grid: &SdfGrid,
    tile_min: TyVector3I32,
    tile_max: TyVector3I32,
    fill_mode: FillMode,
    see_through: &IdSlice<BSdfMaterial, bool>,
) -> Vec<(TyVector3I32, U32Id<BSdfMaterial>)> {
    grid.cells
        .iter()
        .enumerate()
        .filter_map(|(index, cell)| {
            let material_id = cell.material?;
            let lattice_index = grid.lattice_index(index);

            let keeps = lattice_index.cmpge(tile_min).all()
                && lattice_index.cmplt(tile_max).all()
                && match fill_mode {
                    FillMode::Solid => true,
                    FillMode::Surface => is_exposed(grid, lattice_index, see_through),
                };

            keeps.then_some((lattice_index, material_id))
        })
        .collect()
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        dependencies::DependenciesImpl,
        operations::sdf_doc::{
            SdfSampleOptions, SdfTileOptions, SdfVoxMainOptions, is_exposed, parts_main, sample,
            sdf_doc_voxelize::tiled_vox_main::tiled_parts, see_through_materials, shared_leaf_main,
            single_part_main, tiled_vox_main, to_vox_main,
        },
        utilities::{FillMode, FlattenMode, GridResolution, VoxelFrame},
    };
    use branded_id::U32Id;
    use sdfcore::{
        BSdfMaterial, SdfMain, SdfMaterial, SdfProperty, SdfPropertyValue, SdfShape3d, SdfSide,
        SdfStep, SdfStepMaterial,
    };
    use std::collections::{HashMap, HashSet};
    use ty_math::TyVector3F64;
    use voxcore::VoxMain;

    /// A cell by its part's name and lattice index.
    type CellKey = (String, [i32; 3]);

    fn material(index: u32) -> SdfStepMaterial {
        SdfStepMaterial::Material(U32Id::from_u32(index))
    }

    fn cuboid(min: [f64; 3], max: [f64; 3]) -> SdfShape3d {
        SdfShape3d::Box {
            min: TyVector3F64::from_array(min),
            max: TyVector3F64::from_array(max),
            round: None,
        }
    }

    fn add(name: &str, shape: u32, material_index: u32) -> SdfStep {
        SdfStep::Add {
            name: name.to_string(),
            shape_id: U32Id::from_u32(shape),
            material: material(material_index),
        }
    }

    fn untiled_options(voxel_size: f64) -> SdfSampleOptions {
        SdfSampleOptions {
            resolution: GridResolution::VoxelSize(voxel_size),
            frame: VoxelFrame::World,
            posed: false,
            record_writes: false,
        }
    }

    fn tile_options(voxel_size: f64, fill_mode: FillMode, tile: u32) -> SdfTileOptions {
        SdfTileOptions {
            resolution: GridResolution::VoxelSize(voxel_size),
            fill_mode,
            tile,
        }
    }

    /// A block with every step kind: a carved pit, a see-through pool, sloped
    /// and flat coats, a paint onto the block, and a `set` stud past it.
    fn block_main() -> SdfMain {
        let mut state = single_part_main(
            vec![
                cuboid([0.0, 0.0, 0.0], [7.0, 4.0, 6.0]),
                SdfShape3d::Sphere {
                    center: TyVector3F64::new(3.5, 4.0, 3.0),
                    radius: 1.6,
                },
                cuboid([5.0, 1.0, 1.0], [6.0, 3.0, 5.0]),
                cuboid([0.0, 0.0, 0.0], [3.0, 2.0, 6.0]),
            ],
            vec![
                add("block", 0, 0),
                SdfStep::Carve {
                    name: "pit".to_string(),
                    shape_id: U32Id::from_u32(1),
                },
                add("pool", 2, 1),
                SdfStep::Coat {
                    name: "skin".to_string(),
                    material: material(0),
                    sides: None,
                    depth: 1.0,
                    slope: Some(45.0),
                    within_id: None,
                    onto_ids: None,
                },
                SdfStep::Paint {
                    name: "stain".to_string(),
                    shape_id: U32Id::from_u32(3),
                    material: material(1),
                    onto_ids: Some(vec![U32Id::from_u32(0)]),
                },
                SdfStep::Set {
                    name: "stud".to_string(),
                    points: vec![TyVector3F64::new(8.25, 0.25, 0.25)],
                    size: 0.5,
                    material: material(0),
                },
                SdfStep::Coat {
                    name: "moss".to_string(),
                    material: material(0),
                    sides: Some(vec![SdfSide::PositiveY]),
                    depth: 1.0,
                    slope: None,
                    within_id: None,
                    onto_ids: None,
                },
            ],
        )
        .into_state();

        state.materials[U32Id::from_u32(1).to_usize_id()] = SdfMaterial::Material {
            properties: vec![SdfProperty {
                name: "transmission".to_string(),
                value: SdfPropertyValue::Number(1.0),
            }],
        };

        SdfMain::new(state).unwrap()
    }

    /// The cells an untiled run of `main` keeps under `fill_mode`, with their
    /// materials.
    fn untiled_cells(
        main: &SdfMain,
        voxel_size: f64,
        fill_mode: FillMode,
    ) -> HashMap<CellKey, U32Id<BSdfMaterial>> {
        let sampling = sample(&DependenciesImpl, main, &untiled_options(voxel_size)).unwrap();
        let see_through = see_through_materials(&sampling.materials);
        let mut parts = HashSet::new();
        let mut cells = HashMap::new();

        for place in &sampling.places {
            if !parts.insert(place.node_id) {
                continue;
            }

            for &grid_index in &place.grid_indices {
                let grid = &sampling.grids[grid_index];

                for (index, cell) in grid.cells.iter().enumerate() {
                    let lattice_index = grid.lattice_index(index);

                    if let Some(material_id) = cell.material
                        && (fill_mode == FillMode::Solid
                            || is_exposed(grid, lattice_index, &see_through))
                    {
                        let key = (place.path.last().unwrap().clone(), lattice_index.to_array());
                        assert_eq!(cells.insert(key, material_id), None);
                    }
                }
            }
        }

        cells
    }

    /// The cells the tiled objects of `main` hold, with their materials.
    fn tiled_cells(
        main: &SdfMain,
        voxel_size: f64,
        fill_mode: FillMode,
        tile: u32,
    ) -> HashMap<CellKey, U32Id<BSdfMaterial>> {
        let parts = tiled_parts(
            &DependenciesImpl,
            main,
            &tile_options(voxel_size, fill_mode, tile),
        )
        .unwrap();
        let mut cells = HashMap::new();

        for node in &parts.nodes {
            for &object_index in &node.object_indices {
                let object = &parts.objects[object_index];

                for (position, material_id) in &object.cells {
                    let lattice_index = object.min + position.as_ivec3();
                    let key = (node.name.clone(), lattice_index.to_array());
                    assert_eq!(
                        cells.insert(key, *material_id),
                        None,
                        "one tile holds a cell"
                    );
                }
            }
        }

        cells
    }

    /// The nodes, objects, and live voxels of `document`.
    fn described(document: &VoxMain) -> Vec<String> {
        let nodes = document.iter_hierarchy_nodes().map(|(_, node)| {
            format!(
                "{} {:?}: nodes {:?}, objects {:?}",
                node.name, node.transform, node.child_node_ids, node.child_object_ids
            )
        });

        let objects = document.iter_objects().map(|(_, object)| {
            let samples: Vec<_> = object
                .iter_layers()
                .flat_map(|(layer_id, _)| object.iter_live_samples(layer_id).unwrap())
                .collect();

            format!(
                "{} from {:?} across {:?}: {samples:?}",
                object.name(),
                object.origin(),
                object.bounds()
            )
        });

        nodes
            .chain(objects)
            .chain([format!("{} palettes", document.palette_count())])
            .collect()
    }

    #[test]
    fn tiles_hold_the_cells_an_untiled_grid_keeps() {
        for (main, voxel_size) in [
            (block_main(), 0.5),
            (parts_main(), 1.0),
            (shared_leaf_main(2.0), 1.0),
        ] {
            for fill_mode in [FillMode::Solid, FillMode::Surface] {
                let untiled = untiled_cells(&main, voxel_size, fill_mode);
                assert!(!untiled.is_empty());

                for tile in [1, 2, 3, 5, 32] {
                    assert_eq!(
                        tiled_cells(&main, voxel_size, fill_mode, tile),
                        untiled,
                        "{fill_mode:?} at {tile} cells"
                    );
                }
            }
        }
    }

    #[test]
    fn parts_that_each_fit_one_tile_write_the_untiled_document() {
        for fill_mode in [FillMode::Solid, FillMode::Surface] {
            let sampling = sample(&DependenciesImpl, &parts_main(), &untiled_options(1.0)).unwrap();
            let untiled = to_vox_main(
                &sampling,
                &SdfVoxMainOptions {
                    fill_mode,
                    flatten: FlattenMode::None,
                },
            )
            .unwrap();

            let tiled = tiled_vox_main(
                &DependenciesImpl,
                &parts_main(),
                &tile_options(1.0, fill_mode, 32),
            )
            .unwrap();

            assert_eq!(described(&tiled), described(&untiled));
        }
    }

    #[test]
    fn each_tile_writes_an_object_under_its_part_node() {
        let document = tiled_vox_main(
            &DependenciesImpl,
            &block_main(),
            &tile_options(0.5, FillMode::Surface, 4),
        )
        .unwrap();
        assert_eq!(document.validate(), Ok(()));

        let nodes: Vec<_> = document.iter_hierarchy_nodes().collect();
        assert_eq!(nodes.len(), 1);
        assert_eq!(
            nodes[0].1.child_object_ids.len(),
            document.iter_objects().count()
        );
        assert!(document.iter_objects().count() > 1);

        let voxels: usize = document
            .iter_objects()
            .map(|(_, object)| {
                assert_eq!(object.name(), "model");
                object.live_count()
            })
            .sum();
        assert_eq!(
            voxels,
            untiled_cells(&block_main(), 0.5, FillMode::Surface).len()
        );
    }

    #[test]
    fn a_tiled_part_may_outgrow_the_untiled_grid_cap() {
        // Two cells 600 cells apart along each axis span over 2^27 cells.
        let main = single_part_main(
            vec![
                cuboid([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
                cuboid([600.0, 600.0, 600.0], [601.0, 601.0, 601.0]),
            ],
            vec![add("near", 0, 0), add("far", 1, 0)],
        );

        let error = sample(&DependenciesImpl, &main, &untiled_options(1.0))
            .unwrap_err()
            .to_string();
        assert!(error.contains("must hold at most"), "{error}");

        let parts = tiled_parts(
            &DependenciesImpl,
            &main,
            &tile_options(1.0, FillMode::Solid, 64),
        )
        .unwrap();
        assert_eq!(parts.objects.len(), 2);
    }

    #[test]
    fn a_part_without_tiles_checks_its_steps_as_untiled() {
        // A part of no `add` or `set` steps reaches no tile.
        let main = single_part_main(
            Vec::new(),
            vec![SdfStep::Coat {
                name: "skin".to_string(),
                material: material(1),
                sides: None,
                depth: 0.25,
                slope: None,
                within_id: None,
                onto_ids: None,
            }],
        );

        let untiled = sample(&DependenciesImpl, &main, &untiled_options(1.0))
            .unwrap_err()
            .to_string();
        let error = tiled_vox_main(
            &DependenciesImpl,
            &main,
            &tile_options(1.0, FillMode::Solid, 8),
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("model/skin"), "{error}");
        assert_eq!(error, untiled);
    }

    #[test]
    fn a_tile_outside_one_to_512_cells_errors() {
        for tile in [0, 513] {
            let error = tiled_vox_main(
                &DependenciesImpl,
                &parts_main(),
                &tile_options(1.0, FillMode::Solid, tile),
            )
            .unwrap_err()
            .to_string();

            assert!(
                error.contains(&format!("tile must span 1 to 512 cells, not {tile}")),
                "{error}"
            );
        }
    }
}
