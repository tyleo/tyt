use crate::{
    Error, Result,
    dependencies::sdf_doc::MapCells,
    operations::sdf_doc::{
        EntryPaths, SdfSampleOptions, SdfSampling, SdfShapes, check_document, check_shape_boxes,
        collect_places, resolve_materials, sample_grid, sample_voxel_size,
    },
    utilities::VoxelFrame,
};
use sdfcore::SdfMain;
use std::collections::HashMap;
use ty_math::TyVector3F64;

/// Samples the model `main` into grids under `options` by model evaluation.
/// `dependencies` spreads each step's cells across threads. Errors on the
/// first failed check.
pub fn sample<D: MapCells>(
    dependencies: &D,
    main: &SdfMain,
    options: &SdfSampleOptions,
) -> Result<SdfSampling> {
    let state = main.state();
    let mut places = collect_places(state);
    let paths = EntryPaths::new(state, &places);

    check_document(state, &places, &paths)?;
    let materials = resolve_materials(state, &paths)?;

    let shapes = SdfShapes::new(main);
    check_shape_boxes(state, &shapes, &paths)?;

    let voxel_size = sample_voxel_size(state, &places, &shapes, options.resolution)?;
    let mut grids = Vec::new();
    let mut shared_grids = HashMap::new();

    for place in &mut places {
        for &object_id in &state.nodes[place.node_id.to_usize_id()].child_object_ids {
            let grid_index = match options.frame {
                VoxelFrame::World => {
                    grids.push(sample_grid(
                        dependencies,
                        state,
                        &shapes,
                        object_id,
                        &place.path,
                        voxel_size,
                        place.offset,
                        options.record_writes,
                    )?);
                    grids.len() - 1
                }

                VoxelFrame::Local => match shared_grids.get(&object_id) {
                    Some(grid_index) => *grid_index,

                    None => {
                        grids.push(sample_grid(
                            dependencies,
                            state,
                            &shapes,
                            object_id,
                            &place.path,
                            voxel_size,
                            TyVector3F64::ZERO,
                            options.record_writes,
                        )?);
                        shared_grids.insert(object_id, grids.len() - 1);
                        grids.len() - 1
                    }
                },
            };

            place.grid_indices.push(grid_index);
        }
    }

    let holds_live_cell = grids
        .iter()
        .any(|grid| grid.cells.iter().any(|cell| cell.material.is_some()));

    if !holds_live_cell {
        return Err(Error::invalid(
            "model must hold a live cell, not only empty grids",
        ));
    }

    Ok(SdfSampling {
        voxel_size,
        frame: options.frame,
        writes_recorded: options.record_writes,
        places,
        grids,
        materials,
    })
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        dependencies::DependenciesImpl,
        operations::sdf_doc::{
            SdfSampleOptions, SdfSampling, sample, shared_leaf_main, single_part_main,
        },
        utilities::{GridResolution, ResolutionReference, VoxelFrame},
    };
    use branded_id::{IdVec, U32Id};
    use sdfcore::{
        SdfMain, SdfMaterial, SdfNode, SdfObject, SdfPattern, SdfShape3d, SdfSide, SdfState,
        SdfStep, SdfStepMaterial,
    };
    use std::f64::consts::PI;
    use ty_math::{TyAxis3, TyVector3F64, TyVector3I32};

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

    fn at_size(voxel_size: f64) -> SdfSampleOptions {
        SdfSampleOptions {
            resolution: GridResolution::VoxelSize(voxel_size),
            frame: VoxelFrame::World,
            record_writes: true,
        }
    }

    fn live_cells(sampling: &SdfSampling) -> Vec<TyVector3I32> {
        sampling
            .grids
            .iter()
            .flat_map(|grid| {
                grid.cells
                    .iter()
                    .enumerate()
                    .filter_map(|(index, cell)| cell.material.map(|_| grid.lattice_index(index)))
            })
            .collect()
    }

    fn error(main: &SdfMain, options: SdfSampleOptions) -> String {
        sample(&DependenciesImpl, main, &options)
            .unwrap_err()
            .to_string()
    }

    #[test]
    fn a_box_with_corners_on_cell_corners_fills_exactly_its_cells() {
        for (min, max, first, last) in [
            ([0.0, 0.0, 0.0], [0.05, 0.25, 0.05], [0, 0, 0], [1, 9, 1]),
            (
                [-0.05, -0.025, 0.1],
                [0.025, 0.05, 0.175],
                [-2, -1, 4],
                [0, 1, 6],
            ),
        ] {
            let main = single_part_main(vec![cuboid(min, max)], vec![add("block", 0, 0)]);
            let cells = live_cells(&sample(&DependenciesImpl, &main, &at_size(0.025)).unwrap());

            let first = TyVector3I32::from_array(first);
            let last = TyVector3I32::from_array(last);
            let span = last - first + 1;

            assert_eq!(cells.len() as i32, span.x * span.y * span.z);
            assert!(
                cells
                    .iter()
                    .all(|cell| cell.cmpge(first).all() && cell.cmple(last).all())
            );
        }
    }

    #[test]
    fn each_step_writes_the_cells_model_evaluation_sets() {
        let main = single_part_main(
            vec![
                cuboid([0.0, 0.0, 0.0], [4.0, 4.0, 4.0]),
                cuboid([0.0, 0.0, 2.0], [4.0, 4.0, 4.0]),
                cuboid([0.0, 0.0, 0.0], [2.0, 4.0, 2.0]),
            ],
            vec![
                add("block", 0, 0),
                SdfStep::Carve {
                    name: "cut".to_string(),
                    shape_id: U32Id::from_u32(1),
                },
                SdfStep::Paint {
                    name: "stain".to_string(),
                    shape_id: U32Id::from_u32(2),
                    material: material(1),
                    onto_ids: None,
                },
                SdfStep::Coat {
                    name: "top".to_string(),
                    material: material(1),
                    sides: Some(vec![SdfSide::PositiveY]),
                    depth: 1.0,
                    within_id: None,
                    onto_ids: None,
                },
                SdfStep::Set {
                    name: "stud".to_string(),
                    points: vec![TyVector3F64::new(5.5, 0.5, 0.5)],
                    size: 1.0,
                    material: material(1),
                },
            ],
        );

        let sampling = sample(&DependenciesImpl, &main, &at_size(1.0)).unwrap();
        let grid = &sampling.grids[0];

        let written: Vec<usize> = grid
            .steps
            .iter()
            .map(|record| record.written.len())
            .collect();
        assert_eq!(written, [64, 32, 16, 8, 1]);

        let overwritten: Vec<&[u32]> = grid
            .steps
            .iter()
            .map(|record| record.overwritten_steps.as_slice())
            .collect();
        assert_eq!(overwritten, [&[][..], &[0], &[0], &[0, 2], &[]]);
        assert_eq!(grid.size.to_array(), [6, 4, 4]);

        let cell = |x, y, z| *grid.cell(TyVector3I32::new(x, y, z)).unwrap();
        assert_eq!(
            (cell(3, 0, 0).material, cell(3, 0, 0).step),
            (Some(U32Id::from_u32(0)), Some(0))
        );
        assert_eq!(
            (cell(0, 0, 3).material, cell(0, 0, 3).step),
            (None, Some(1))
        );
        assert_eq!(
            (cell(0, 0, 0).material, cell(0, 0, 0).step),
            (Some(U32Id::from_u32(1)), Some(2))
        );
        assert_eq!(
            (cell(3, 3, 1).material, cell(3, 3, 1).step),
            (Some(U32Id::from_u32(1)), Some(3))
        );
        assert_eq!(
            (cell(5, 0, 0).material, cell(5, 0, 0).step),
            (Some(U32Id::from_u32(1)), Some(4))
        );
        assert_eq!((cell(4, 0, 0).material, cell(4, 0, 0).step), (None, None));
    }

    #[test]
    fn a_set_fills_the_cells_its_cube_centers() {
        let set = |point, size| {
            single_part_main(
                Vec::new(),
                vec![SdfStep::Set {
                    name: "stud".to_string(),
                    points: vec![TyVector3F64::from_array(point)],
                    size,
                    material: material(0),
                }],
            )
        };

        for (point, size, count) in [
            ([0.5, 0.5, 0.5], 1.0, 1),
            ([0.5, 0.5, 0.5], 3.0, 27),
            ([1.0, 1.0, 1.0], 2.0, 8),
            ([1.0, 1.0, 1.0], 1.0, 1),
        ] {
            let cells =
                live_cells(&sample(&DependenciesImpl, &set(point, size), &at_size(1.0)).unwrap());
            assert_eq!(cells.len(), count, "{point:?} at {size}");
        }

        assert_eq!(
            error(&set([0.2, 0.5, 0.5], 0.2), at_size(1.0)),
            "model must hold a live cell, not only empty grids"
        );
    }

    #[test]
    fn a_coat_reaches_its_depth_toward_every_side() {
        for (depth, written) in [(1.0, 26), (2.0, 27)] {
            let main = single_part_main(
                vec![cuboid([0.0, 0.0, 0.0], [3.0, 3.0, 3.0])],
                vec![
                    add("block", 0, 0),
                    SdfStep::Coat {
                        name: "skin".to_string(),
                        material: material(1),
                        sides: None,
                        depth,
                        within_id: None,
                        onto_ids: None,
                    },
                ],
            );

            let sampling = sample(&DependenciesImpl, &main, &at_size(1.0)).unwrap();
            assert_eq!(sampling.grids[0].steps[1].written.len(), written);
        }
    }

    #[test]
    fn a_coat_reads_the_shapes_and_errors_under_one_voxel() {
        let coat = |depth| SdfStep::Coat {
            name: "skin".to_string(),
            material: material(1),
            sides: Some(vec![SdfSide::PositiveY]),
            depth,
            within_id: None,
            onto_ids: None,
        };

        let main = single_part_main(
            vec![
                cuboid([0.0, 0.0, 0.0], [1.0, 2.0, 1.0]),
                cuboid([0.0, 1.0, 0.0], [1.0, 2.0, 1.0]),
            ],
            vec![
                add("post", 0, 0),
                SdfStep::Carve {
                    name: "cut".to_string(),
                    shape_id: U32Id::from_u32(1),
                },
                coat(0.5),
            ],
        );
        let sampling = sample(&DependenciesImpl, &main, &at_size(0.5)).unwrap();
        let grid = &sampling.grids[0];
        let steps: Vec<Option<u32>> = (0..4)
            .map(|y| grid.cell(TyVector3I32::new(0, y, 0)).unwrap().step)
            .collect();
        assert_eq!(steps, [Some(0), Some(2), Some(1), Some(1)]);

        let main = single_part_main(
            vec![cuboid([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])],
            vec![add("post", 0, 0), coat(0.4)],
        );
        assert_eq!(
            error(&main, at_size(0.5)),
            "model/skin: coat depth must be at least the voxel size 0.5, not 0.4"
        );
    }

    #[test]
    fn onto_keeps_a_paint_and_a_coat_on_the_cells_their_adds_still_own() {
        let row = |from, to| cuboid([from, 0.0, 0.0], [to, 1.0, 1.0]);
        let onto = |step: u32| Some(vec![U32Id::from_u32(step)]);

        let main = single_part_main(
            vec![row(0.0, 4.0), row(2.0, 6.0), row(3.0, 4.0), row(0.0, 6.0)],
            vec![
                add("body", 0, 0),
                add("gem", 1, 0),
                SdfStep::Carve {
                    name: "cut".to_string(),
                    shape_id: U32Id::from_u32(2),
                },
                SdfStep::Set {
                    name: "stud".to_string(),
                    points: vec![TyVector3F64::new(3.5, 0.5, 0.5)],
                    size: 1.0,
                    material: material(0),
                },
                SdfStep::Paint {
                    name: "gilt".to_string(),
                    shape_id: U32Id::from_u32(3),
                    material: material(1),
                    onto_ids: onto(0),
                },
                SdfStep::Coat {
                    name: "rim".to_string(),
                    material: material(1),
                    sides: None,
                    depth: 1.0,
                    within_id: None,
                    onto_ids: onto(1),
                },
            ],
        );

        let sampling = sample(&DependenciesImpl, &main, &at_size(1.0)).unwrap();
        let grid = &sampling.grids[0];
        let steps: Vec<Option<u32>> = (0..6)
            .map(|x| grid.cell(TyVector3I32::new(x, 0, 0)).unwrap().step)
            .collect();
        assert_eq!(
            steps,
            [Some(4), Some(4), Some(5), Some(3), Some(5), Some(5)]
        );
    }

    #[test]
    fn onto_lists_only_earlier_adds() {
        let paint = |onto: u32| SdfStep::Paint {
            name: "gilt".to_string(),
            shape_id: U32Id::from_u32(0),
            material: material(1),
            onto_ids: Some(vec![U32Id::from_u32(onto)]),
        };
        let cut = SdfStep::Carve {
            name: "cut".to_string(),
            shape_id: U32Id::from_u32(0),
        };
        let block = vec![cuboid([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])];

        assert_eq!(
            error(
                &single_part_main(block.clone(), vec![paint(1), add("body", 0, 0)]),
                at_size(1.0)
            ),
            "model/gilt: paint onto must list add steps earlier in the list, not \"body\""
        );
        assert_eq!(
            error(
                &single_part_main(block, vec![add("body", 0, 0), cut, paint(1)]),
                at_size(1.0)
            ),
            "model/gilt: paint onto must list add steps earlier in the list, not \"cut\""
        );
    }

    /// Two-material `bands` across x, one cell thick.
    fn bands() -> SdfPattern {
        SdfPattern::Bands {
            material_ids: vec![U32Id::from_u32(0), U32Id::from_u32(1)],
            axis: TyAxis3::X,
            period: 1.0,
            warp: None,
            seed: None,
        }
    }

    #[test]
    fn a_pattern_reads_the_frame_its_shape_was_built_in() {
        let mut state = single_part_main(
            vec![
                cuboid([0.0, 0.0, 0.0], [4.0, 1.0, 1.0]),
                SdfShape3d::Translate {
                    shape_id: U32Id::from_u32(0),
                    offset: TyVector3F64::new(1.0, 2.0, 0.0),
                },
            ],
            Vec::new(),
        )
        .state()
        .clone();
        state.patterns = IdVec::from_vec(vec![bands()]);
        state.steps = IdVec::from_vec(vec![
            SdfStep::Add {
                name: "low".to_string(),
                shape_id: U32Id::from_u32(0),
                material: SdfStepMaterial::Pattern(U32Id::from_u32(0)),
            },
            SdfStep::Add {
                name: "high".to_string(),
                shape_id: U32Id::from_u32(1),
                material: SdfStepMaterial::Pattern(U32Id::from_u32(0)),
            },
        ]);
        state.objects.as_mut_vec()[0].step_ids = vec![U32Id::from_u32(0), U32Id::from_u32(1)];
        let main = SdfMain::new(state).unwrap();

        let sampling = sample(&DependenciesImpl, &main, &at_size(1.0)).unwrap();
        let material = |x, y| {
            sampling.grids[0]
                .cell(TyVector3I32::new(x, y, 0))
                .unwrap()
                .material
                .unwrap()
                .to_u32()
        };

        assert_eq!([material(0, 0), material(1, 0), material(2, 0)], [0, 1, 0]);
        assert_eq!([material(1, 2), material(2, 2), material(3, 2)], [0, 1, 0]);
    }

    #[test]
    fn a_set_pattern_reads_the_cell_center_before_the_offsets() {
        let main = SdfMain::new(SdfState {
            materials: IdVec::from_vec(vec![
                SdfMaterial::Material {
                    properties: Vec::new(),
                };
                2
            ]),
            patterns: IdVec::from_vec(vec![bands()]),
            steps: IdVec::from_vec(vec![SdfStep::Set {
                name: "stud".to_string(),
                points: vec![TyVector3F64::new(0.5, 0.5, 0.5)],
                size: 1.0,
                material: SdfStepMaterial::Pattern(U32Id::from_u32(0)),
            }]),
            objects: IdVec::from_vec(vec![SdfObject {
                name: "moved".to_string(),
                step_ids: vec![U32Id::from_u32(0)],
            }]),
            nodes: IdVec::from_vec(vec![
                SdfNode {
                    name: "moved".to_string(),
                    pivot: None,
                    offset: Some(TyVector3F64::new(1.0, 0.0, 0.0)),
                    rotation: Vec::new(),
                    child_object_ids: vec![U32Id::from_u32(0)],
                    child_node_ids: Vec::new(),
                },
                SdfNode {
                    name: "model".to_string(),
                    pivot: None,
                    offset: None,
                    rotation: Vec::new(),
                    child_object_ids: Vec::new(),
                    child_node_ids: vec![U32Id::from_u32(0)],
                },
            ]),
            root_node_ids: vec![U32Id::from_u32(1)],
            ..SdfState::default()
        })
        .unwrap();

        let sampling = sample(&DependenciesImpl, &main, &at_size(1.0)).unwrap();
        let cell = sampling.grids[0].cell(TyVector3I32::new(1, 0, 0)).unwrap();

        assert_eq!(cell.material, Some(U32Id::from_u32(0)));
    }

    #[test]
    fn each_place_samples_moved_by_its_offsets_under_the_world_frame() {
        let sampling = sample(&DependenciesImpl, &shared_leaf_main(), &at_size(1.0)).unwrap();

        let paths: Vec<String> = sampling
            .places
            .iter()
            .map(|place| place.path.join("/"))
            .collect();
        assert_eq!(
            paths,
            [
                "model",
                "model/a",
                "model/a/leaf",
                "model/b",
                "model/b/leaf"
            ]
        );

        let leaf_under_a = &sampling.places[2];
        assert_eq!(leaf_under_a.parent, Some(1));
        assert_eq!(leaf_under_a.offset, TyVector3F64::new(2.0, 0.0, 0.0));
        assert_eq!(leaf_under_a.pivot, TyVector3F64::new(2.5, 0.0, 0.0));

        assert_eq!(
            live_cells(&sampling),
            [TyVector3I32::new(2, 0, 0), TyVector3I32::ZERO]
        );
    }

    #[test]
    fn each_part_samples_once_for_all_its_places_under_the_local_frame() {
        let options = SdfSampleOptions {
            frame: VoxelFrame::Local,
            ..at_size(1.0)
        };
        let sampling = sample(&DependenciesImpl, &shared_leaf_main(), &options).unwrap();

        assert_eq!(sampling.grids.len(), 1);
        assert_eq!(sampling.places[2].grid_indices, [0]);
        assert_eq!(sampling.places[4].grid_indices, [0]);
        assert_eq!(sampling.places[4].pivot, TyVector3F64::new(0.5, 0.25, 0.0));
    }

    #[test]
    fn a_resolution_divides_the_side_its_reference_measures() {
        let main = single_part_main(
            vec![
                cuboid([0.0, 0.0, 0.0], [2.0, 1.0, 1.0]),
                cuboid([0.0, 0.0, 0.0], [1.0, 0.5, 0.5]),
            ],
            vec![add("big", 0, 0), add("small", 1, 0)],
        );

        let voxel_size = |reference, count| {
            let options = SdfSampleOptions {
                resolution: GridResolution::ReferenceCount { reference, count },
                frame: VoxelFrame::World,
                record_writes: false,
            };
            sample(&DependenciesImpl, &main, &options)
                .unwrap()
                .voxel_size
        };

        assert_eq!(voxel_size(ResolutionReference::LongestWorld, 4), 0.5);
        assert_eq!(voxel_size(ResolutionReference::ShortestObject, 4), 0.25);
        assert_eq!(voxel_size(ResolutionReference::WorldY, 2), 0.5);

        let world = sample(
            &DependenciesImpl,
            &shared_leaf_main(),
            &SdfSampleOptions {
                resolution: GridResolution::ReferenceCount {
                    reference: ResolutionReference::WorldX,
                    count: 3,
                },
                frame: VoxelFrame::World,
                record_writes: false,
            },
        )
        .unwrap();
        assert_eq!(world.voxel_size, 1.0);
    }

    #[test]
    fn a_failed_check_reports_the_step_that_reaches_it() {
        let ball = |radius| SdfShape3d::Sphere {
            center: TyVector3F64::ZERO,
            radius,
        };

        assert_eq!(
            error(
                &single_part_main(vec![ball(-1.0)], vec![add("ball", 0, 0)]),
                at_size(1.0)
            ),
            "model/ball: sphere radius must be above zero, not -1"
        );
        assert_eq!(
            error(
                &single_part_main(vec![ball(1.0), ball(0.0)], vec![add("ball", 0, 0)]),
                at_size(1.0)
            ),
            "shapes3d[1]: sphere radius must be above zero, not 0"
        );
        assert_eq!(
            error(
                &single_part_main(vec![ball(1.0)], vec![add("ball", 0, 0), add("ball", 0, 1)]),
                at_size(1.0)
            ),
            "model: step name must be unique in its list, not \"ball\" twice"
        );
        assert_eq!(
            error(
                &single_part_main(
                    vec![SdfShape3d::HalfSpace {
                        side: SdfSide::PositiveY,
                        at: 0.0,
                    }],
                    vec![add("floor", 0, 0)],
                ),
                at_size(1.0)
            ),
            "model/floor: add shape must be bounded, not a shape that reaches without end"
        );
        assert_eq!(
            error(
                &single_part_main(
                    vec![
                        cuboid([0.0, 0.0, 0.0], [4.0, 0.1, 0.1]),
                        SdfShape3d::Bend {
                            shape_id: U32Id::from_u32(0),
                            along: TyAxis3::X,
                            toward: SdfSide::PositiveY,
                            radius: 1.0,
                            pivot: None,
                        }
                    ],
                    vec![add("hoop", 1, 0)],
                ),
                at_size(1.0)
            ),
            format!(
                "model/hoop: bend shape must be at most {} long along x to bend within half a turn, not 4",
                PI
            )
        );
        assert_eq!(
            error(
                &single_part_main(
                    vec![
                        ball(1.0),
                        SdfShape3d::Elongate {
                            shape_id: U32Id::from_u32(0),
                            lengths: TyVector3F64::new(0.0, 2.0, 0.0),
                            center: Some(TyVector3F64::new(0.0, 1.5, 0.0)),
                        }
                    ],
                    vec![add("pill", 1, 0)],
                ),
                at_size(1.0)
            ),
            "model/pill: elongate center must be inside its shape's box from -1 to 1 along y, not 1.5"
        );
        assert_eq!(
            error(
                &single_part_main(
                    vec![cuboid([0.0, 0.0, 0.0], [1000.0, 1000.0, 1000.0])],
                    vec![add("slab", 0, 0)]
                ),
                at_size(1.0)
            ),
            "model: grid over [0, 0, 0] .. [1000, 1000, 1000] must hold at most 134217728 cells, \
             not 1000000000"
        );
        assert_eq!(
            error(
                &single_part_main(vec![ball(1.0)], vec![add("ball", 0, 0)]),
                at_size(0.0)
            ),
            "voxel size must be above zero, not 0"
        );
        assert_eq!(
            error(
                &single_part_main(
                    vec![ball(1.0)],
                    vec![SdfStep::Carve {
                        name: "dig".to_string(),
                        shape_id: U32Id::from_u32(0),
                    }],
                ),
                at_size(1.0)
            ),
            "model must hold a live cell, not only empty grids"
        );
    }
}
