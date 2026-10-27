use crate::{
    Error, Result,
    dependencies::sdf_doc::MapCells,
    operations::sdf_doc::{
        EntryPaths, PlaceFrame, SdfCellBounds, SdfGrid, SdfSampleOptions, SdfSampling, SdfShapes,
        SdfStepRecord, check_document, check_shape_boxes, collect_places, resolve_materials,
        sample_grid, sample_voxel_size, text_outlines, whole_voxels,
    },
    utilities::VoxelFrame,
};
use sdfcore::SdfMain;
use std::collections::HashMap;
use ty_math::{TyPoseF64, TyVector3I32};

/// Samples the model `main` into grids under `options` by model evaluation.
/// `dependencies` spreads each step's cells across threads. Errors on the
/// first failed check.
pub fn sample<D: MapCells>(
    dependencies: &D,
    main: &SdfMain,
    options: &SdfSampleOptions,
) -> Result<SdfSampling> {
    if options.posed && options.frame == VoxelFrame::Local {
        return Err(Error::invalid(
            "posed sampling must sample under the world frame, not the local frame",
        ));
    }

    let state = main.state();
    let mut places = collect_places(state);
    let paths = EntryPaths::new(state, &places);

    check_document(state, &places, &paths)?;
    let materials = resolve_materials(state, &paths)?;

    let shapes = SdfShapes::new(main, text_outlines(main, &paths)?);
    check_shape_boxes(state, &shapes, &paths)?;

    let voxel_size = sample_voxel_size(state, &places, &shapes, options.resolution)?;
    let mut grids = Vec::new();
    let mut shared_grids = HashMap::new();

    let frames = PlaceFrame::of_places(&places, options.posed);

    // The index of the first place of each part.
    let mut first_places = HashMap::new();

    for (index, frame) in frames.into_iter().enumerate() {
        let first = *first_places.entry(places[index].node_id).or_insert(index);

        // Unposed under the world frame, a later place a whole number of
        // voxels from its part's first place takes the first place's grids
        // moved by those voxels.
        if options.frame == VoxelFrame::World
            && !options.posed
            && first != index
            && let Some(shift) =
                whole_voxels((places[index].offset - places[first].offset) / voxel_size)
        {
            for grid_index in places[first].grid_indices.clone() {
                let grid = shifted_grid(&grids[grid_index], shift);
                grids.push(grid);
                places[index].grid_indices.push(grids.len() - 1);
            }

            continue;
        }

        let place = &mut places[index];
        let frame = frame.sampling();
        let pose = TyPoseF64::new(
            frame.position + frame.rotation * place.offset,
            frame.rotation,
        );

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
                        pose,
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
                            TyPoseF64::IDENTITY,
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
        posed: options.posed,
        writes_recorded: options.record_writes,
        places,
        grids,
        materials,
    })
}

/// `grid` moved by `shift` whole cells.
fn shifted_grid(grid: &SdfGrid, shift: TyVector3I32) -> SdfGrid {
    SdfGrid {
        min: grid.min + shift,
        steps: grid
            .steps
            .iter()
            .map(|step| SdfStepRecord {
                written_bounds: step.written_bounds.map(|bounds| SdfCellBounds {
                    min: bounds.min + shift,
                    max: bounds.max + shift,
                }),
                ..step.clone()
            })
            .collect(),
        ..grid.clone()
    }
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
        SdfAxisRotation, SdfFont, SdfFontData, SdfGlyph, SdfGlyphContour, SdfGlyphSegment, SdfMain,
        SdfMaterial, SdfNode, SdfObject, SdfPattern, SdfShape2d, SdfShape3d, SdfSide, SdfState,
        SdfStep, SdfStepMaterial,
    };
    use std::{collections::BTreeMap, f64::consts::PI};
    use ty_math::{TyAxis3, TyVector2F64, TyVector3F64, TyVector3I32};

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
            posed: false,
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
                    slope: None,
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
                        slope: None,
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
    fn a_coat_slope_keeps_a_flat_top_and_skips_a_steep_cone() {
        // A flat slab beside a cone whose sides face 76 degrees off up.
        let cone = SdfShape3d::Cone {
            a: TyVector3F64::new(8.0, 0.0, 2.0),
            b: TyVector3F64::new(8.0, 6.0, 2.0),
            radius_a: 1.5,
            radius_b: 0.0,
        };

        for (slope, slab, crown) in [(None, 256, 112), (Some(40.0), 256, 0)] {
            let main = single_part_main(
                vec![cuboid([0.0, 0.0, 0.0], [4.0, 1.0, 4.0]), cone.clone()],
                vec![
                    add("slab", 0, 0),
                    add("crown", 1, 0),
                    SdfStep::Coat {
                        name: "snow".to_string(),
                        material: material(1),
                        sides: Some(vec![SdfSide::PositiveY]),
                        depth: 0.25,
                        slope,
                        within_id: None,
                        onto_ids: None,
                    },
                ],
            );

            let sampling = sample(&DependenciesImpl, &main, &at_size(0.25)).unwrap();
            let grid = &sampling.grids[0];
            let written = &grid.steps[2].written;
            let on_slab = written
                .iter()
                .filter(|&&index| grid.lattice_index(index as usize).x < 16)
                .count();
            assert_eq!(
                (on_slab, written.len() - on_slab),
                (slab, crown),
                "{slope:?}"
            );
        }
    }

    #[test]
    fn a_coat_reads_the_shapes_and_errors_under_one_voxel() {
        let coat = |depth| SdfStep::Coat {
            name: "skin".to_string(),
            material: material(1),
            sides: Some(vec![SdfSide::PositiveY]),
            depth,
            slope: None,
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
                    slope: None,
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
                    joint: false,
                    child_object_ids: vec![U32Id::from_u32(0)],
                    child_node_ids: Vec::new(),
                },
                SdfNode {
                    name: "model".to_string(),
                    pivot: None,
                    offset: None,
                    rotation: Vec::new(),
                    joint: false,
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
        let sampling = sample(&DependenciesImpl, &shared_leaf_main(2.0), &at_size(1.0)).unwrap();

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
            [TyVector3I32::new(2, 0, 0), TyVector3I32::new(0, 2, 0)]
        );
    }

    /// A part's name, pivot, offset, and rotation.
    type Link<'a> = (&'a str, [f64; 3], [f64; 3], Vec<SdfAxisRotation>);

    /// A model whose root part `model` holds the chain of parts `chain`, each
    /// holding the next. The last part runs `steps` over `shapes`.
    fn chain_main(shapes: Vec<SdfShape3d>, steps: Vec<SdfStep>, chain: &[Link]) -> SdfMain {
        // A node lists only earlier nodes, so the chain lands leaf first.
        let mut nodes: Vec<SdfNode> = chain
            .iter()
            .rev()
            .enumerate()
            .map(|(index, (name, pivot, offset, rotation))| SdfNode {
                name: name.to_string(),
                pivot: Some(TyVector3F64::from_array(*pivot)),
                offset: Some(TyVector3F64::from_array(*offset)),
                rotation: rotation.clone(),
                joint: false,
                child_object_ids: if index == 0 {
                    vec![U32Id::from_u32(0)]
                } else {
                    Vec::new()
                },
                child_node_ids: if index == 0 {
                    Vec::new()
                } else {
                    vec![U32Id::from_u32(index as u32 - 1)]
                },
            })
            .collect();
        nodes.push(SdfNode {
            name: "model".to_string(),
            pivot: None,
            offset: None,
            rotation: Vec::new(),
            joint: false,
            child_object_ids: Vec::new(),
            child_node_ids: vec![U32Id::from_u32(chain.len() as u32 - 1)],
        });

        SdfMain::new(SdfState {
            shapes3d: IdVec::from_vec(shapes),
            materials: IdVec::from_vec(vec![SdfMaterial::Material {
                properties: Vec::new(),
            }]),
            objects: IdVec::from_vec(vec![SdfObject {
                name: "leaf".to_string(),
                step_ids: (0..steps.len() as u32).map(U32Id::from_u32).collect(),
            }]),
            steps: IdVec::from_vec(steps),
            root_node_ids: vec![U32Id::from_u32(nodes.len() as u32 - 1)],
            nodes: IdVec::from_vec(nodes),
            ..SdfState::default()
        })
        .unwrap()
    }

    fn turn(axis: TyAxis3, degrees: f64) -> Vec<SdfAxisRotation> {
        vec![SdfAxisRotation { axis, degrees }]
    }

    fn posed() -> SdfSampleOptions {
        SdfSampleOptions {
            posed: true,
            ..at_size(1.0)
        }
    }

    #[test]
    fn a_posed_place_samples_through_every_turn_below_the_root_part() {
        let main = chain_main(
            vec![cuboid([1.0, 0.0, 0.0], [2.0, 1.0, 2.0])],
            vec![add("hand", 0, 0)],
            &[
                ("arm", [0.0; 3], [0.0; 3], turn(TyAxis3::Z, 90.0)),
                (
                    "hand",
                    [1.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    turn(TyAxis3::Y, 90.0),
                ),
            ],
        );

        // The hand's quarter turn about y at [2, 0, 0] carries its box to x 2
        // to 4 and z -1 to 0, and the arm's quarter turn about z then carries
        // that onto x -1 to 0 and y 2 to 4.
        assert_eq!(
            live_cells(&sample(&DependenciesImpl, &main, &posed()).unwrap()),
            [TyVector3I32::new(-1, 2, -1), TyVector3I32::new(-1, 3, -1)]
        );
    }

    #[test]
    fn a_turned_set_cube_fills_the_cells_whose_centers_it_covers() {
        let main = chain_main(
            Vec::new(),
            vec![SdfStep::Set {
                name: "stud".to_string(),
                points: vec![TyVector3F64::ZERO],
                size: 2.0,
                material: material(0),
            }],
            &[("turned", [0.0; 3], [0.0; 3], turn(TyAxis3::Y, 45.0))],
        );

        let cells = live_cells(&sample(&DependenciesImpl, &main, &posed()).unwrap());
        assert_eq!(cells.len(), 8);
        assert!(
            cells
                .iter()
                .all(|cell| (-1..=0).contains(&cell.x) && (-1..=0).contains(&cell.z))
        );
    }

    #[test]
    fn posed_sampling_needs_the_world_frame() {
        let options = SdfSampleOptions {
            frame: VoxelFrame::Local,
            ..posed()
        };

        assert_eq!(
            error(&shared_leaf_main(0.25), options),
            "posed sampling must sample under the world frame, not the local frame"
        );
    }

    #[test]
    fn each_part_samples_once_for_all_its_places_under_the_local_frame() {
        let options = SdfSampleOptions {
            frame: VoxelFrame::Local,
            ..at_size(1.0)
        };
        let sampling = sample(&DependenciesImpl, &shared_leaf_main(0.25), &options).unwrap();

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
                posed: false,
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
            &shared_leaf_main(0.25),
            &SdfSampleOptions {
                resolution: GridResolution::ReferenceCount {
                    reference: ResolutionReference::WorldX,
                    count: 3,
                },
                frame: VoxelFrame::World,
                posed: false,
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

    #[test]
    fn a_text_fills_its_letters_and_an_error_points_to_its_font() {
        let text = |string: &str| {
            let mut state =
                single_part_main(vec![cuboid([0.0; 3], [1.0; 3])], vec![add("letters", 0, 0)])
                    .state()
                    .clone();
            state.shapes3d = IdVec::from_vec(vec![SdfShape3d::Extrude {
                profile_id: U32Id::from_u32(0),
                axis: None,
                from: 0.0,
                to: 0.01,
            }]);
            state.shapes2d = IdVec::from_vec(vec![SdfShape2d::Text {
                font_id: U32Id::from_u32(0),
                string: string.to_owned(),
                size: 0.07,
                align: None,
            }]);
            state.fonts = IdVec::from_vec(vec![SdfFont::Installed {
                name: "sign".to_owned(),
                font: "Test Sans".to_owned(),
            }]);
            SdfMain::new(state).unwrap()
        };

        // `H` covers 500 by 700 units of a 700 unit cap height.
        let corner = |u: f64, v: f64| SdfGlyphSegment::Line {
            to: TyVector2F64::new(u, v),
        };
        let h = SdfGlyph {
            advance: 600.0,
            contours: vec![SdfGlyphContour {
                start: TyVector2F64::ZERO,
                segments: vec![corner(0.0, 700.0), corner(500.0, 700.0), corner(500.0, 0.0)],
            }],
        };
        let fonts = IdVec::from_vec(vec![SdfFontData {
            units_per_em: 1000.0,
            cap_height: Some(700.0),
            glyphs: [('H', h)].into(),
            kerning: BTreeMap::new(),
        }]);

        let loaded = |string: &str| text(string).with_font_data(fonts.clone());

        let sampling = sample(&DependenciesImpl, &loaded("H"), &at_size(0.01)).unwrap();
        assert_eq!(live_cells(&sampling).len(), 5 * 7);

        for (main, message) in [
            (loaded("HÉ"), ": the font `sign` has no `É`"),
            (text("H"), ": the font `sign` is not loaded"),
        ] {
            let error = sample(&DependenciesImpl, &main, &at_size(0.01))
                .unwrap_err()
                .to_string();
            assert!(error.ends_with(message), "{error}");
        }
    }
}
