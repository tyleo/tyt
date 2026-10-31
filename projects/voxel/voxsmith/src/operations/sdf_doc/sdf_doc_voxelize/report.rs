use crate::{
    Error, Result,
    operations::sdf_doc::{
        Bounds3d, FACE_OFFSETS, PlaceFrame, SdfCell, SdfGrid, SdfPlace, SdfSampling, SdfStepRecord,
        is_exposed, meters, see_through_materials,
    },
    utilities::{FillMode, VoxelFrame},
};
use branded_id::{IdSlice, U32Id};
use sdfcore::{
    BSdfMaterial, BSdfNode, BSdfShape3d, SdfAxes3d, SdfMain, SdfShape3d, SdfState, SdfStep,
};
use std::{
    borrow::Cow,
    cmp::Reverse,
    collections::{HashMap, HashSet, VecDeque},
};
use ty_math::{
    TyPoseF64, TyQuaternionExt, TyQuaternionF64, TyVector3F64, TyVector3I32, ZERO_LENGTH_TOLERANCE,
};

/// The report on the grids of `sampling` for the voxj document `name` written
/// under `fill_mode` by model evaluation. Errors on grids sampled under the
/// local frame or without their writes recorded.
pub fn report(
    main: &SdfMain,
    sampling: &SdfSampling,
    fill_mode: FillMode,
    name: &str,
) -> Result<String> {
    if sampling.frame == VoxelFrame::Local {
        return Err(Error::invalid(
            "report must read grids sampled under the world frame, not the local frame",
        ));
    }

    if !sampling.writes_recorded {
        return Err(Error::invalid(
            "report must read grids sampled with their writes recorded",
        ));
    }

    let state = main.state();
    let voxel_size = sampling.voxel_size;

    let place_cells: Vec<HashSet<TyVector3I32>> = sampling
        .places
        .iter()
        .map(|place| {
            place
                .grid_indices
                .iter()
                .flat_map(|&grid_index| live_cells(&sampling.grids[grid_index]))
                .collect()
        })
        .collect();

    let poses = world_poses(&sampling.places);
    let node_of = node_places(sampling);

    // A place's cells sit in the frame of the place that keeps its node.
    let cell_poses: Vec<TyPoseF64> = node_of.iter().map(|&node| poses[node]).collect();
    let (lattice_of, lattice_poses) = lattices(&cell_poses);

    let mut covered: Vec<HashSet<TyVector3I32>> = vec![HashSet::new(); lattice_poses.len()];

    for (place_index, cells) in place_cells.iter().enumerate() {
        covered[lattice_of[place_index]].extend(cells);
    }

    let pieces = model_pieces(&covered, &lattice_poses, voxel_size);

    // A model in one piece leaves no part detached and lists no pieces.
    let piece_of: HashMap<(usize, TyVector3I32), usize> = match pieces.len() {
        0 | 1 => HashMap::new(),
        _ => pieces
            .iter()
            .enumerate()
            .flat_map(|(index, piece)| piece.iter().map(move |cell| (*cell, index)))
            .collect(),
    };
    let detached = detached_places(sampling, &place_cells, &lattice_of, &piece_of, pieces.len());

    let see_through = see_through_materials(&sampling.materials);

    // The voxel counts read the cells the document holds.
    let place_voxels: Cow<[HashSet<TyVector3I32>]> = match fill_mode {
        FillMode::Solid => Cow::Borrowed(&place_cells[..]),

        FillMode::Surface => Cow::Owned(
            sampling
                .places
                .iter()
                .map(|place| {
                    place
                        .grid_indices
                        .iter()
                        .flat_map(|&grid_index| {
                            let grid = &sampling.grids[grid_index];

                            live_cells(grid).filter(|cell| is_exposed(grid, *cell, &see_through))
                        })
                        .collect()
                })
                .collect(),
        ),
    };

    // Each node holds its voxels apart, so voxels where nodes overlap count once
    // for each node.
    let mut node_voxels: Vec<HashSet<TyVector3I32>> = vec![HashSet::new(); place_voxels.len()];

    for (place_index, cells) in place_voxels.iter().enumerate() {
        node_voxels[node_of[place_index]].extend(cells);
    }

    let voxel_count: u64 = node_voxels.iter().map(|cells| cells.len() as u64).sum();

    let roots: Vec<usize> = (0..sampling.places.len())
        .filter(|&index| sampling.places[index].parent.is_none())
        .collect();
    let lone_root = roots.len() == 1;

    let mut lists_per_name: HashMap<&str, usize> = HashMap::new();

    for place in &sampling.places {
        for &grid_index in &place.grid_indices {
            let object = &state.objects[sampling.grids[grid_index].object_id.to_usize_id()];

            for step_id in &object.step_ids {
                *lists_per_name
                    .entry(state.steps[step_id.to_usize_id()].name())
                    .or_default() += 1;
            }
        }
    }

    let place_label = |place_index: usize| {
        let path = &sampling.places[place_index].path;
        let parts = if lone_root { &path[1..] } else { &path[..] };

        parts.join("/")
    };

    let step_label = |place_index: usize, step: &SdfStep| {
        if lists_per_name[step.name()] < 2 {
            return step.name().to_owned();
        }

        [place_label(place_index).as_str(), step.name()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("/")
    };

    let extent = world_extent_fields(
        covered
            .iter()
            .enumerate()
            .flat_map(|(lattice, cells)| cells.iter().map(move |cell| (lattice, *cell))),
        &lattice_poses,
        voxel_size,
    );

    let pieces_label = match pieces.len() {
        1 => "1 piece".to_owned(),
        count => format!("{count} pieces"),
    };

    let mut lines = vec![ReportLine {
        kind: LineKind::Model,
        depth: 0,
        fields: [
            vec![
                Some(Field::Text(name.to_owned())),
                Some(Field::Text(format!(
                    "{} of {} m",
                    count_text(voxel_count, "voxel", "voxels"),
                    meters(voxel_size)
                ))),
                Some(Field::Text(pieces_label.clone())),
            ],
            extent,
        ]
        .concat(),
    }];

    let mut writer = ReportWriter {
        state,
        sampling,
        place_cells: &place_cells,
        place_voxels: &place_voxels,
        detached: &detached,
        see_through: &see_through,
        poses: &poses,
        cell_poses: &cell_poses,
        place_label: &place_label,
        step_label: &step_label,
        first_places: HashMap::new(),
        lines: &mut lines,
        checks: Vec::new(),
    };

    for &root in &roots {
        if lone_root {
            writer
                .first_places
                .insert(sampling.places[root].node_id, root);
            writer.contents(root, 1);
        } else {
            writer.place(root, 1);
        }
    }

    let mut checks = writer.checks;

    if pieces.len() > 1 {
        let mut sources: Vec<Vec<String>> = vec![Vec::new(); pieces.len()];

        for (place_index, place) in sampling.places.iter().enumerate() {
            for &grid_index in &place.grid_indices {
                let grid = &sampling.grids[grid_index];
                let object = &state.objects[grid.object_id.to_usize_id()];

                for (step_id, record) in object.step_ids.iter().zip(&grid.steps) {
                    let step = &state.steps[step_id.to_usize_id()];
                    let mut reached: Vec<usize> = live_written(grid, record)
                        .map(|cell| piece_of[&(lattice_of[place_index], cell)])
                        .collect();
                    reached.sort_unstable();
                    reached.dedup();

                    for piece_index in reached {
                        sources[piece_index].push(step_label(place_index, step));
                    }
                }
            }
        }

        let width = pieces.len().to_string().len();
        let mut piece_voxels = vec![0; pieces.len()];

        for (node, cells) in node_voxels.iter().enumerate() {
            for cell in cells {
                piece_voxels[piece_of[&(lattice_of[node], *cell)]] += 1;
            }
        }
        let mut folds: Vec<(String, Vec<usize>)> = Vec::new();
        let mut piece_lines = Vec::with_capacity(pieces.len());

        for (index, (piece, sources)) in pieces.iter().zip(sources).enumerate() {
            let from = format!("from {}", sources.join(", "));

            if index > 0 {
                match folds.iter_mut().find(|(fold, _)| *fold == from) {
                    Some((_, indices)) => indices.push(index),
                    None => folds.push((from.clone(), vec![index])),
                }
            }

            let line = ReportLine {
                kind: LineKind::Piece,
                depth: 1,
                fields: [
                    vec![
                        Some(Field::Text(format!("piece {:>width$}", index + 1))),
                        Some(count_field(piece_voxels[index], "voxel", "voxels")),
                    ],
                    world_extent_fields(piece.iter().copied(), &lattice_poses, voxel_size),
                    vec![Some(Field::Text(from))],
                ]
                .concat(),
            };

            lines.push(line.clone());
            piece_lines.push(line);
        }

        // Pieces from the same steps fold into one line, so a run of links or
        // bubbles reads once and a stray from another step stands apart.
        for (from, indices) in folds {
            if let [index] = indices[..] {
                checks.push(piece_lines[index].clone());
                continue;
            }

            let counts: Vec<u64> = indices.iter().map(|&index| piece_voxels[index]).collect();
            let least = counts.iter().min().expect("a fold holds pieces");
            let most = counts.iter().max().expect("a fold holds pieces");
            let size = if least == most {
                format!("{} each", count_text(*least, "voxel", "voxels"))
            } else {
                format!("{least} to {most} voxels")
            };

            checks.push(ReportLine {
                kind: LineKind::Piece,
                depth: 1,
                fields: [
                    vec![
                        Some(Field::Text(format!("{} pieces", indices.len()))),
                        Some(Field::Text(size)),
                    ],
                    world_extent_fields(
                        indices
                            .iter()
                            .flat_map(|&index| pieces[index].iter().copied()),
                        &lattice_poses,
                        voxel_size,
                    ),
                    vec![Some(Field::Text(from))],
                ]
                .concat(),
            });
        }
    }

    lines.push(ReportLine {
        kind: LineKind::Check,
        depth: 0,
        fields: vec![
            Some(Field::Text("check".to_owned())),
            Some(count_field(checks.len() as u64, "line", "lines")),
            Some(Field::Text(format!(
                "model {}, {pieces_label}",
                count_text(voxel_count, "voxel", "voxels")
            ))),
        ],
    });
    lines.extend(checks);

    Ok(layout(&lines))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LineKind {
    Check,

    Model,

    Part,

    Piece,

    Step,
}

#[derive(Clone, PartialEq)]
enum Field {
    /// A count beside its label, with the counts of one column aligned right.
    Count(u64, &'static str),

    /// Text aligned left.
    Text(String),
}

/// One line of the report before its columns line up.
#[derive(Clone)]
struct ReportLine {
    kind: LineKind,

    depth: usize,

    /// The fields in column order, with `None` for a column the line leaves
    /// blank.
    fields: Vec<Option<Field>>,
}

/// Adds the part and step lines of the places to a report.
struct ReportWriter<'a> {
    state: &'a SdfState,

    sampling: &'a SdfSampling,

    place_cells: &'a [HashSet<TyVector3I32>],

    /// Each place's cells the document holds.
    place_voxels: &'a [HashSet<TyVector3I32>],

    /// Whether each place reads detached.
    detached: &'a [bool],

    see_through: &'a IdSlice<BSdfMaterial, bool>,

    /// Each place's pose in the world.
    poses: &'a [TyPoseF64],

    /// The world pose of the frame that holds each place's cells.
    cell_poses: &'a [TyPoseF64],

    /// The path a line gives the place at an index.
    place_label: &'a dyn Fn(usize) -> String,

    /// The name a check line gives a step of the place at an index.
    step_label: &'a dyn Fn(usize, &SdfStep) -> String,

    /// The first place written of each part, by the part's node.
    first_places: HashMap<U32Id<BSdfNode>, usize>,

    lines: &'a mut Vec<ReportLine>,

    /// The step lines the check block repeats.
    checks: Vec<ReportLine>,
}

impl ReportWriter<'_> {
    /// Adds the line of the place at `place_index` at `depth`, then the
    /// place's steps and child parts one level deeper. A part's steps show
    /// under its first place only.
    fn place(&mut self, place_index: usize, depth: usize) {
        let place = &self.sampling.places[place_index];
        let cells = &self.place_cells[place_index];

        let repeats = match self.first_places.get(&place.node_id) {
            Some(&first) => Some(Field::Text(format!(
                "repeats {}",
                (self.place_label)(first)
            ))),

            None => {
                self.first_places.insert(place.node_id, place_index);
                None
            }
        };

        let detached = self.detached[place_index].then(|| Field::Text("detached".to_owned()));

        let voxel_size = self.sampling.voxel_size;
        let pose = self.poses[place_index];
        let extent = extent_fields(cells.iter().copied(), voxel_size);

        // A part under a turn adds its posed box.
        let posed = posed_bounds(
            cells.iter().copied(),
            self.cell_poses[place_index],
            voxel_size,
        )
        .map(|bounds| bounds_text(&bounds))
        .filter(|posed| extent[1] != Some(Field::Text(posed.clone())))
        .map(|posed| Field::Text(format!("posed {posed}")));

        self.lines.push(ReportLine {
            kind: LineKind::Part,
            depth,
            fields: [
                vec![
                    Some(Field::Text("part".to_owned())),
                    Some(Field::Text(
                        place.path.last().expect("a place has a part").clone(),
                    )),
                    Some(count_field(
                        self.place_voxels[place_index].len() as u64,
                        "voxel",
                        "voxels",
                    )),
                ],
                extent,
                vec![
                    Some(Field::Text(format!(
                        "pivot {}",
                        point_text(pose.position + pose.rotation * place.pivot)
                    ))),
                    posed,
                    detached,
                    repeats,
                ],
            ]
            .concat(),
        });

        self.contents(place_index, depth + 1);
    }

    /// Adds the step lines of the place at `place_index` at `depth` when the
    /// place comes first for its part, then the lines of its child parts.
    fn contents(&mut self, place_index: usize, depth: usize) {
        let place = &self.sampling.places[place_index];

        if self.first_places[&place.node_id] == place_index {
            let sampling = self.sampling;

            for &grid_index in &place.grid_indices {
                self.steps(&sampling.grids[grid_index], place_index, depth);
            }
        }

        for (child_index, child) in self.sampling.places.iter().enumerate() {
            if child.parent == Some(place_index) {
                self.place(child_index, depth);
            }
        }
    }

    /// Adds the line of each step that ran over `grid` at `depth`, and the
    /// check line of each step a review has to clear.
    fn steps(&mut self, grid: &SdfGrid, place_index: usize, depth: usize) {
        let object = &self.state.objects[grid.object_id.to_usize_id()];
        let step_name = |step_index: u32| {
            self.state.steps[object.step_ids[step_index as usize].to_usize_id()].name()
        };

        for (step_index, step_id) in object.step_ids.iter().enumerate() {
            let step = &self.state.steps[step_id.to_usize_id()];
            let record = &grid.steps[step_index];
            let kept: Vec<(TyVector3I32, bool)> = kept_cells(grid, step_index).collect();

            let (exposed, piece_count) = match step {
                SdfStep::Carve { .. } => (None, 0),

                _ => {
                    let exposed = kept
                        .iter()
                        .filter(|(cell, live)| *live && is_exposed(grid, *cell, self.see_through))
                        .count();

                    (
                        Some(exposed),
                        face_pieces(&live_written(grid, record).collect()).len(),
                    )
                }
            };

            let over = match step {
                SdfStep::Coat { .. } | SdfStep::Paint { .. }
                    if !record.overwritten_steps.is_empty() =>
                {
                    Some(Field::Text(format!(
                        "over {}",
                        names(record.overwritten_steps.iter().copied(), step_name)
                    )))
                }

                _ => None,
            };

            let lost_to: Vec<u32> = (step_index + 1..grid.steps.len())
                .filter(|&later| {
                    grid.steps[later]
                        .overwritten_steps
                        .contains(&(step_index as u32))
                })
                .map(|later| later as u32)
                .collect();
            let lost = (!lost_to.is_empty())
                .then(|| Field::Text(format!("lost to {}", names(lost_to.into_iter(), step_name))));

            let written = record.written_bounds.map(|bounds| [bounds.min, bounds.max]);

            let line = |name: String, depth: usize| ReportLine {
                kind: LineKind::Step,
                depth,
                fields: [
                    vec![
                        Some(Field::Text(step_kind(step).to_owned())),
                        Some(Field::Text(name)),
                        Some(count_field(record.written.len() as u64, "cell", "cells")),
                        Some(count_field(kept.len() as u64, "kept", "kept")),
                        exposed.map(|exposed| count_field(exposed as u64, "exposed", "exposed")),
                        (piece_count > 1).then_some(Field::Count(piece_count as u64, "pieces")),
                    ],
                    extent_fields(written.into_iter().flatten(), self.sampling.voxel_size),
                    vec![over.clone(), lost.clone()],
                ]
                .concat(),
            };

            // A step in more pieces than the forms it draws has broken apart.
            let forms = match step {
                SdfStep::Add { shape_id, .. } | SdfStep::Paint { shape_id, .. } => {
                    forms(self.state, *shape_id)
                }

                SdfStep::Set { points, .. } => points.len() as u64,

                _ => 1,
            };

            if kept.is_empty() || exposed == Some(0) || piece_count as u64 > forms {
                self.checks
                    .push(line((self.step_label)(place_index, step), 1));
            }

            self.lines.push(line(step.name().to_owned(), depth));
        }
    }
}

/// How many forms the shape at `shape_id` draws.
fn forms(state: &SdfState, shape_id: U32Id<BSdfShape3d>) -> u64 {
    match &state.shapes3d[shape_id.to_usize_id()] {
        SdfShape3d::Mirror { shape_id, axes, .. } => {
            let reflections = match axes {
                SdfAxes3d::X | SdfAxes3d::Y | SdfAxes3d::Z => 2,
                SdfAxes3d::Xy | SdfAxes3d::Xz | SdfAxes3d::Yz => 4,
                SdfAxes3d::Xyz => 8,
            };

            reflections * forms(state, *shape_id)
        }

        SdfShape3d::Repeat {
            shape_id, count, ..
        } => (count.x * count.y * count.z) as u64 * forms(state, *shape_id),

        SdfShape3d::RepeatPolar {
            shape_id, count, ..
        } => *count as u64 * forms(state, *shape_id),

        SdfShape3d::Bend { shape_id, .. }
        | SdfShape3d::Displace { shape_id, .. }
        | SdfShape3d::Elongate { shape_id, .. }
        | SdfShape3d::Offset { shape_id, .. }
        | SdfShape3d::Orient { shape_id, .. }
        | SdfShape3d::Rotate { shape_id, .. }
        | SdfShape3d::Scale { shape_id, .. }
        | SdfShape3d::Shell { shape_id, .. }
        | SdfShape3d::SmoothSubtract {
            base_id: shape_id, ..
        }
        | SdfShape3d::Subtract {
            base_id: shape_id, ..
        }
        | SdfShape3d::Translate { shape_id, .. }
        | SdfShape3d::Twist { shape_id, .. } => forms(state, *shape_id),

        // An intersect clips one operand's copies to the others.
        SdfShape3d::Intersect { shape_ids } | SdfShape3d::SmoothIntersect { shape_ids, .. } => {
            shape_ids
                .iter()
                .map(|&shape_id| forms(state, shape_id))
                .max()
                .expect("an intersect has a shape")
        }

        SdfShape3d::SmoothUnion { shape_ids, .. } | SdfShape3d::Union { shape_ids } => shape_ids
            .iter()
            .map(|&shape_id| forms(state, shape_id))
            .sum(),

        _ => 1,
    }
}

/// The names `step_name` gives `step_indices`, joined by commas.
fn names<'a>(
    step_indices: impl Iterator<Item = u32>,
    step_name: impl Fn(u32) -> &'a str,
) -> String {
    step_indices.map(step_name).collect::<Vec<_>>().join(", ")
}

/// A count field labeled `singular` or `plural`.
fn count_field(count: u64, singular: &'static str, plural: &'static str) -> Field {
    Field::Count(count, if count == 1 { singular } else { plural })
}

/// A count beside its label, `singular` or `plural`.
fn count_text(count: u64, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count == 1 { singular } else { plural })
}

/// The size and bounds fields of the box around `cells`, or two blank fields
/// for no cells.
fn extent_fields(cells: impl Iterator<Item = TyVector3I32>, voxel_size: f64) -> Vec<Option<Field>> {
    let Some((min, max)) = cells.fold(None, |bounds, cell| {
        Some(
            bounds.map_or((cell, cell), |(min, max): (TyVector3I32, TyVector3I32)| {
                (min.min(cell), max.max(cell))
            }),
        )
    }) else {
        return vec![None, None];
    };

    let size = max - min + 1;

    vec![
        Some(Field::Text(format!("{}x{}x{}", size.x, size.y, size.z))),
        Some(Field::Text(bounds_text(&Bounds3d {
            min: min.as_dvec3() * voxel_size,
            max: (max + 1).as_dvec3() * voxel_size,
        }))),
    ]
}

/// The size and bounds fields of the box in the world around `cells`, each
/// beside its lattice in `lattice_poses`. Under a turn, the box reads posed and
/// has no size in cells.
fn world_extent_fields(
    cells: impl Iterator<Item = (usize, TyVector3I32)> + Clone,
    lattice_poses: &[TyPoseF64],
    voxel_size: f64,
) -> Vec<Option<Field>> {
    let extent = extent_fields(cells.clone().map(|(_, cell)| cell), voxel_size);
    let posed = lattice_poses
        .iter()
        .enumerate()
        .filter_map(|(lattice, pose)| {
            posed_bounds(
                cells
                    .clone()
                    .filter(|(cell_lattice, _)| *cell_lattice == lattice)
                    .map(|(_, cell)| cell),
                *pose,
                voxel_size,
            )
        })
        .reduce(|bounds, lattice| bounds.union(&lattice))
        .map(|bounds| Field::Text(bounds_text(&bounds)));

    if extent[1] == posed {
        extent
    } else {
        vec![None, posed]
    }
}

/// The box in meters around `cells` posed by `pose`, or `None` for no cells.
fn posed_bounds(
    cells: impl Iterator<Item = TyVector3I32>,
    pose: TyPoseF64,
    voxel_size: f64,
) -> Option<Bounds3d> {
    let half_extents = pose
        .rotation
        .rotate_extents_abs(TyVector3F64::splat(voxel_size / 2.0));

    cells
        .map(|cell| {
            let center = (cell.as_dvec3() + 0.5) * voxel_size;

            Bounds3d::around(pose.position + pose.rotation * center, half_extents)
        })
        .reduce(|bounds, cell| bounds.union(&cell))
}

/// The world pose of each of `places`, which list each place before its
/// children. A pose composes the turns about the pivots on the place's path.
fn world_poses(places: &[SdfPlace]) -> Vec<TyPoseF64> {
    let mut poses: Vec<TyPoseF64> = Vec::with_capacity(places.len());

    for place in places {
        let base = place
            .parent
            .map_or(TyPoseF64::IDENTITY, |parent| poses[parent]);

        // The place turns `x` about its pivot to
        // `place.rotation * (x - pivot) + pivot`, and the base pose carries
        // the result.
        poses.push(TyPoseF64::new(
            base.position + base.rotation * (place.pivot - place.rotation * place.pivot),
            base.rotation * place.rotation,
        ));
    }

    poses
}

/// The place whose node holds the voxels of each place of `sampling`: the place
/// itself when it keeps its node, else the nearest kept place above it.
fn node_places(sampling: &SdfSampling) -> Vec<usize> {
    let frames = PlaceFrame::of_places(&sampling.places, sampling.posed);
    let mut node_of: Vec<usize> = Vec::with_capacity(frames.len());

    for (index, place) in sampling.places.iter().enumerate() {
        node_of.push(match place.parent {
            Some(parent) if !frames[index].kept => node_of[parent],
            _ => index,
        });
    }

    node_of
}

/// The lattice of each place and the world pose of each lattice, from
/// `cell_poses`. Places whose cells sit in frames of one pose share a lattice.
fn lattices(cell_poses: &[TyPoseF64]) -> (Vec<usize>, Vec<TyPoseF64>) {
    let mut lattice_poses: Vec<TyPoseF64> = Vec::new();

    let lattice_of = cell_poses
        .iter()
        .map(
            |pose| match lattice_poses.iter().position(|lattice| lattice == pose) {
                Some(lattice) => lattice,

                None => {
                    lattice_poses.push(*pose);
                    lattice_poses.len() - 1
                }
            },
        )
        .collect();

    (lattice_of, lattice_poses)
}

/// `bounds` as a report line shows it.
fn bounds_text(bounds: &Bounds3d) -> String {
    format!("{} .. {}", point_text(bounds.min), point_text(bounds.max))
}

/// `point` as a report line shows it.
fn point_text(point: TyVector3F64) -> String {
    format!(
        "[{}, {}, {}]",
        meters(point.x),
        meters(point.y),
        meters(point.z)
    )
}

/// The live cells of `grid` by lattice index.
fn live_cells(grid: &SdfGrid) -> impl Iterator<Item = TyVector3I32> + '_ {
    grid_cells(grid)
        .filter(|(_, cell)| cell.material.is_some())
        .map(|(position, _)| position)
}

/// The cells of `grid` the step of `record` wrote that hold a material.
fn live_written<'a>(
    grid: &'a SdfGrid,
    record: &'a SdfStepRecord,
) -> impl Iterator<Item = TyVector3I32> + 'a {
    record
        .written
        .iter()
        .map(|&index| index as usize)
        .filter(|&index| grid.cells[index].material.is_some())
        .map(|index| grid.lattice_index(index))
}

/// The cells of `grid` whose last change came from the step at `step_index` of
/// its list, each with whether it holds a material.
fn kept_cells(
    grid: &SdfGrid,
    step_index: usize,
) -> impl Iterator<Item = (TyVector3I32, bool)> + '_ {
    grid_cells(grid)
        .filter(move |(_, cell)| cell.step == Some(step_index as u32))
        .map(|(position, cell)| (position, cell.material.is_some()))
}

/// Each cell of `grid` beside its lattice index, in a raster with x outermost.
fn grid_cells(grid: &SdfGrid) -> impl Iterator<Item = (TyVector3I32, &SdfCell)> + '_ {
    grid.cells
        .iter()
        .enumerate()
        .map(|(index, cell)| (grid.lattice_index(index), cell))
}

/// Whether each place of `sampling` reads detached: the place and its parent
/// have live cells, yet no piece holding the cells of the place or a part below
/// it holds a cell of the parent. `piece_of` holds the piece of every cell in
/// `place_cells`, beside the lattice `lattice_of` gives its place, once the
/// model splits into `piece_count` pieces.
fn detached_places(
    sampling: &SdfSampling,
    place_cells: &[HashSet<TyVector3I32>],
    lattice_of: &[usize],
    piece_of: &HashMap<(usize, TyVector3I32), usize>,
    piece_count: usize,
) -> Vec<bool> {
    let place_count = sampling.places.len();

    if piece_count < 2 {
        return vec![false; place_count];
    }

    let mut children = vec![Vec::new(); place_count];

    for (index, place) in sampling.places.iter().enumerate() {
        if let Some(parent) = place.parent {
            children[parent].push(index);
        }
    }

    let place_pieces: Vec<HashSet<usize>> = place_cells
        .iter()
        .zip(lattice_of)
        .map(|(cells, &lattice)| {
            cells
                .iter()
                .map(|cell| piece_of[&(lattice, *cell)])
                .collect()
        })
        .collect();

    (0..place_count)
        .map(|index| {
            let Some(parent) = sampling.places[index].parent else {
                return false;
            };

            if place_cells[index].is_empty() || place_cells[parent].is_empty() {
                return false;
            }

            let mut below = vec![index];
            let mut stack = vec![index];

            while let Some(place) = stack.pop() {
                below.extend(&children[place]);
                stack.extend(&children[place]);
            }

            below
                .iter()
                .all(|&place| place_pieces[place].is_disjoint(&place_pieces[parent]))
        })
        .collect()
}

/// The face-connected groups of `cells`, from the largest. Among groups of one
/// size, the group whose first cell comes first in a raster with x outermost
/// comes first.
fn face_pieces(cells: &HashSet<TyVector3I32>) -> Vec<Vec<TyVector3I32>> {
    let mut raster: Vec<TyVector3I32> = cells.iter().copied().collect();
    raster.sort_by_key(|cell| cell.to_array());

    let mut seen = HashSet::with_capacity(cells.len());
    let mut pieces = Vec::new();

    for start in raster {
        if !seen.insert(start) {
            continue;
        }

        let mut piece = vec![start];
        let mut queue = VecDeque::from([start]);

        while let Some(cell) = queue.pop_front() {
            for face in FACE_OFFSETS {
                let neighbor = cell + face;

                if cells.contains(&neighbor) && seen.insert(neighbor) {
                    piece.push(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }

        pieces.push(piece);
    }

    pieces.sort_by_key(|piece| Reverse(piece.len()));
    pieces
}

/// The pieces of the cells `covered` holds per lattice, from the largest, with
/// each cell beside its lattice. Cells of one lattice join across a face, and
/// cells of two lattices join where their cubes meet in the world.
fn model_pieces(
    covered: &[HashSet<TyVector3I32>],
    lattice_poses: &[TyPoseF64],
    voxel_size: f64,
) -> Vec<Vec<(usize, TyVector3I32)>> {
    let lattice_pieces: Vec<(usize, Vec<TyVector3I32>)> = covered
        .iter()
        .enumerate()
        .flat_map(|(lattice, cells)| {
            face_pieces(cells)
                .into_iter()
                .map(move |piece| (lattice, piece))
        })
        .collect();
    let mut roots: Vec<usize> = (0..lattice_pieces.len()).collect();

    if lattice_poses.len() > 1 {
        let center = |lattice: usize, cell: TyVector3I32| {
            let pose = lattice_poses[lattice];

            pose.position + pose.rotation * ((cell.as_dvec3() + 0.5) * voxel_size)
        };

        // Cubes that meet have centers less than a bucket apart, so a cell's
        // bucket and the buckets around it hold every cube it can meet.
        let bucket_size = 2.0 * voxel_size;
        let bucket_of = |center: TyVector3F64| (center / bucket_size).floor().as_ivec3();
        let neighbors: Vec<TyVector3I32> = (-1..=1)
            .flat_map(|x| {
                (-1..=1).flat_map(move |y| (-1..=1).map(move |z| TyVector3I32::new(x, y, z)))
            })
            .collect();
        let mut buckets: HashMap<TyVector3I32, Vec<(usize, TyVector3F64)>> = HashMap::new();

        for (piece_index, (lattice, piece)) in lattice_pieces.iter().enumerate() {
            for &cell in piece {
                let center = center(*lattice, cell);
                buckets
                    .entry(bucket_of(center))
                    .or_default()
                    .push((piece_index, center));
            }
        }

        // Only surface cells look for contact, because where two pieces meet, a
        // surface cell of one meets a cell of the other.
        for (piece_index, (lattice, piece)) in lattice_pieces.iter().enumerate() {
            let cells = &covered[*lattice];
            let rotation = lattice_poses[*lattice].rotation;

            for &cell in piece {
                if FACE_OFFSETS
                    .iter()
                    .all(|&face| cells.contains(&(cell + face)))
                {
                    continue;
                }

                let center = center(*lattice, cell);
                let bucket = bucket_of(center);

                for &(other_index, other_center) in neighbors
                    .iter()
                    .filter_map(|&neighbor| buckets.get(&(bucket + neighbor)))
                    .flatten()
                {
                    let other_lattice = lattice_pieces[other_index].0;
                    let root = root_of(&mut roots, piece_index);
                    let other_root = root_of(&mut roots, other_index);

                    if other_lattice != *lattice
                        && root != other_root
                        && cubes_meet(
                            center,
                            rotation,
                            other_center,
                            lattice_poses[other_lattice].rotation,
                            voxel_size / 2.0,
                        )
                    {
                        roots[root] = other_root;
                    }
                }
            }
        }
    }

    let mut pieces: Vec<Vec<(usize, TyVector3I32)>> = Vec::new();
    let mut piece_of_root: HashMap<usize, usize> = HashMap::new();

    for (piece_index, (lattice, piece)) in lattice_pieces.iter().enumerate() {
        let root = root_of(&mut roots, piece_index);
        let index = *piece_of_root.entry(root).or_insert_with(|| {
            pieces.push(Vec::new());
            pieces.len() - 1
        });
        pieces[index].extend(piece.iter().map(|&cell| (*lattice, cell)));
    }

    pieces.sort_by_key(|piece| Reverse(piece.len()));
    pieces
}

/// The root of the set holding `index` in the forest `roots`. Each step halves
/// the path to it.
fn root_of(roots: &mut [usize], mut index: usize) -> usize {
    while roots[index] != index {
        roots[index] = roots[roots[index]];
        index = roots[index];
    }

    index
}

/// Whether the cubes of half side `half` centered at `center` and
/// `other_center`, turned by `rotation` and `other_rotation`, overlap or touch.
fn cubes_meet(
    center: TyVector3F64,
    rotation: TyQuaternionF64,
    other_center: TyVector3F64,
    other_rotation: TyQuaternionF64,
    half: f64,
) -> bool {
    let axes = [TyVector3F64::X, TyVector3F64::Y, TyVector3F64::Z];
    let edges = axes.map(|axis| rotation * axis);
    let other_edges = axes.map(|axis| other_rotation * axis);
    let offset = other_center - center;

    // Cubes that stay apart have a separating axis among the face normals and
    // the cross products of the edges.
    let normals = edges.iter().chain(&other_edges).copied().chain(
        edges
            .iter()
            .flat_map(|&edge| other_edges.iter().map(move |&other| edge.cross(other))),
    );

    !normals
        .filter(|normal| normal.length() >= ZERO_LENGTH_TOLERANCE)
        .any(|normal| {
            let normal = normal.normalize();
            let reach: f64 = edges
                .iter()
                .chain(&other_edges)
                .map(|edge| edge.dot(normal).abs())
                .sum();

            // The slack lets faces that touch meet through rounding.
            offset.dot(normal).abs() > half * (reach + 1e-9)
        })
}

/// The name a report gives the kind of `step`.
fn step_kind(step: &SdfStep) -> &'static str {
    match step {
        SdfStep::Add { .. } => "add",
        SdfStep::Carve { .. } => "carve",
        SdfStep::Coat { .. } => "coat",
        SdfStep::Paint { .. } => "paint",
        SdfStep::Set { .. } => "set",
    }
}

/// `lines` as text, with the fields of consecutive lines of one kind and depth
/// padded into columns.
fn layout(lines: &[ReportLine]) -> String {
    let mut text = String::new();
    let mut start = 0;

    while start < lines.len() {
        let end = start
            + lines[start..]
                .iter()
                .take_while(|line| {
                    line.kind == lines[start].kind && line.depth == lines[start].depth
                })
                .count();
        let group = &lines[start..end];
        let columns = group
            .iter()
            .map(|line| line.fields.len())
            .max()
            .unwrap_or(0);

        let widths: Vec<(usize, usize)> = (0..columns)
            .map(|column| {
                group
                    .iter()
                    .filter_map(|line| line.fields.get(column).and_then(Option::as_ref))
                    .fold((0, 0), |(first, second), field| match field {
                        Field::Count(count, label) => {
                            (first.max(count.to_string().len()), second.max(label.len()))
                        }

                        Field::Text(text) => (first.max(text.len()), second),
                    })
            })
            .collect();

        // A column no line of the group fills takes no space.
        let filled: Vec<usize> = (0..columns)
            .filter(|&column| {
                group
                    .iter()
                    .any(|line| matches!(line.fields.get(column), Some(Some(_))))
            })
            .collect();

        for line in group {
            let fields: Vec<String> = filled
                .iter()
                .map(|&column| {
                    let (first, second) = widths[column];

                    match line.fields.get(column).and_then(Option::as_ref) {
                        Some(Field::Count(count, label)) => {
                            format!("{count:>first$} {label:<second$}")
                        }

                        Some(Field::Text(text)) => format!("{text:<first$}"),

                        None if second > 0 => " ".repeat(first + 1 + second),

                        None => " ".repeat(first),
                    }
                })
                .collect();

            text.push_str(&"  ".repeat(line.depth));
            text.push_str(fields.join("  ").trim_end());
            text.push('\n');
        }

        start = end;
    }

    text
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        dependencies::DependenciesImpl,
        operations::sdf_doc::{
            SdfSampleOptions, parts_main, report, sample, shared_leaf_main, single_part_main,
        },
        utilities::{FillMode, GridResolution, VoxelFrame},
    };
    use branded_id::U32Id;
    use sdfcore::{
        SdfAxes3d, SdfAxisRotation, SdfMain, SdfMaterial, SdfProperty, SdfPropertyValue,
        SdfShape3d, SdfStep, SdfStepMaterial,
    };
    use ty_math::{TyAxis3, TyVector3F64};

    fn cuboid(min: [f64; 3], max: [f64; 3]) -> SdfShape3d {
        SdfShape3d::Box {
            min: TyVector3F64::from_array(min),
            max: TyVector3F64::from_array(max),
            round: None,
        }
    }

    /// The report on `main` sampled at `voxel_size` under the world frame.
    fn report_on(main: &SdfMain, voxel_size: f64, fill_mode: FillMode) -> String {
        report_sampled(main, voxel_size, fill_mode, false)
    }

    /// The report on `main` sampled at `voxel_size` under the world frame, at
    /// each part's pose when `posed`.
    fn report_sampled(main: &SdfMain, voxel_size: f64, fill_mode: FillMode, posed: bool) -> String {
        let sampling = sample(
            &DependenciesImpl,
            main,
            &SdfSampleOptions {
                resolution: GridResolution::VoxelSize(voxel_size),
                frame: VoxelFrame::World,
                posed,
                record_writes: true,
            },
        )
        .unwrap();

        report(main, &sampling, fill_mode, "model.voxj").unwrap()
    }

    #[test]
    fn a_step_line_counts_the_cells_the_step_wrote_kept_and_shows() {
        let cuboid = |min: [f64; 3], max: [f64; 3]| SdfShape3d::Box {
            min: TyVector3F64::from_array(min),
            max: TyVector3F64::from_array(max),
            round: None,
        };

        let main = single_part_main(
            vec![
                cuboid([0.0, 0.0, 0.0], [0.1, 0.075, 0.1]),
                cuboid([0.0, 0.05, 0.0], [0.1, 0.075, 0.1]),
                cuboid([0.0, 0.025, 0.0], [0.05, 0.05, 0.1]),
                cuboid([1.0, 1.0, 1.0], [1.025, 1.025, 1.025]),
            ],
            vec![
                SdfStep::Add {
                    name: "block".to_string(),
                    shape_id: U32Id::from_u32(0),
                    material: SdfStepMaterial::Material(U32Id::from_u32(0)),
                },
                SdfStep::Carve {
                    name: "top".to_string(),
                    shape_id: U32Id::from_u32(1),
                },
                SdfStep::Paint {
                    name: "band".to_string(),
                    shape_id: U32Id::from_u32(2),
                    material: SdfStepMaterial::Material(U32Id::from_u32(1)),
                    onto_ids: None,
                },
                SdfStep::Paint {
                    name: "miss".to_string(),
                    shape_id: U32Id::from_u32(3),
                    material: SdfStepMaterial::Material(U32Id::from_u32(1)),
                    onto_ids: None,
                },
            ],
        );

        assert_eq!(
            report_on(&main, 0.025, FillMode::Solid),
            "\
model.voxj  32 voxels of 0.025 m  1 piece  4x2x4  [0, 0, 0] .. [0.1, 0.05, 0.1]
  add    block  48 cells  24 kept  24 exposed  4x3x4  [0, 0, 0] .. [0.1, 0.075, 0.1]                  lost to top, band
  carve  top    16 cells  16 kept              4x1x4  [0, 0.05, 0] .. [0.1, 0.075, 0.1]
  paint  band    8 cells   8 kept   8 exposed  2x1x4  [0, 0.025, 0] .. [0.05, 0.05, 0.1]  over block
  paint  miss    0 cells   0 kept   0 exposed
check  1 line  model 32 voxels, 1 piece
  paint  miss  0 cells  0 kept  0 exposed
"
        );
    }

    #[test]
    fn parts_take_part_lines_and_separate_pieces_take_piece_lines() {
        assert_eq!(
            report_on(&parts_main(), 1.0, FillMode::Solid),
            "\
model.voxj  37 voxels of 1 m  2 pieces  7x3x4  [0, 0, 0] .. [7, 3, 4]
  add  body  32 cells  32 kept  32 exposed  4x2x4  [0, 0, 0] .. [4, 2, 4]
  part  lid  4 voxels  2x1x2  [1, 2, 1] .. [3, 3, 3]  pivot [2, 2, 2]
    add  lid  4 cells  4 kept  4 exposed  2x1x2  [1, 2, 1] .. [3, 3, 3]
  part  pebble  1 voxel  1x1x1  [6, 0, 0] .. [7, 1, 1]  pivot [6, 0, 0]  detached
    add  pebble  1 cell  1 kept  1 exposed  1x1x1  [6, 0, 0] .. [7, 1, 1]
  piece 1  36 voxels  4x3x4  [0, 0, 0] .. [4, 3, 4]  from body, lid
  piece 2   1 voxel   1x1x1  [6, 0, 0] .. [7, 1, 1]  from pebble
check  1 line  model 37 voxels, 2 pieces
  piece 2  1 voxel  1x1x1  [6, 0, 0] .. [7, 1, 1]  from pebble
"
        );
    }

    #[test]
    fn a_part_under_a_turn_reads_its_posed_box_and_world_pivot() {
        // The model's quarter turn about y at the origin carries +x onto -z.
        let mut state = parts_main().state().clone();
        state.nodes[U32Id::from_u32(2).to_usize_id()].rotation = vec![SdfAxisRotation {
            axis: TyAxis3::Y,
            degrees: 90.0,
        }];

        assert_eq!(
            report_on(&SdfMain::new(state).unwrap(), 1.0, FillMode::Solid),
            "\
model.voxj  37 voxels of 1 m  2 pieces  [0, 0, -7] .. [4, 3, 0]
  add  body  32 cells  32 kept  32 exposed  4x2x4  [0, 0, 0] .. [4, 2, 4]
  part  lid  4 voxels  2x1x2  [1, 2, 1] .. [3, 3, 3]  pivot [2, 2, -2]  posed [1, 2, -3] .. [3, 3, -1]
    add  lid  4 cells  4 kept  4 exposed  2x1x2  [1, 2, 1] .. [3, 3, 3]
  part  pebble  1 voxel  1x1x1  [6, 0, 0] .. [7, 1, 1]  pivot [0, 0, -6]  posed [0, 0, -7] .. [1, 1, -6]  detached
    add  pebble  1 cell  1 kept  1 exposed  1x1x1  [6, 0, 0] .. [7, 1, 1]
  piece 1  36 voxels  [0, 0, -4] .. [4, 3, 0]   from body, lid
  piece 2   1 voxel   [0, 0, -7] .. [1, 1, -6]  from pebble
check  1 line  model 37 voxels, 2 pieces
  piece 2  1 voxel  [0, 0, -7] .. [1, 1, -6]  from pebble
"
        );
    }

    #[test]
    fn a_part_posed_onto_its_root_reads_its_turn_once() {
        // The pebble's quarter turn about y at its pivot carries +x onto -z.
        let mut state = parts_main().state().clone();
        state.nodes[U32Id::from_u32(1).to_usize_id()].rotation = vec![SdfAxisRotation {
            axis: TyAxis3::Y,
            degrees: 90.0,
        }];

        assert_eq!(
            report_sampled(&SdfMain::new(state).unwrap(), 1.0, FillMode::Solid, true),
            "\
model.voxj  37 voxels of 1 m  2 pieces  7x3x5  [0, 0, -1] .. [7, 3, 4]
  add  body  32 cells  32 kept  32 exposed  4x2x4  [0, 0, 0] .. [4, 2, 4]
  part  lid  4 voxels  2x1x2  [1, 2, 1] .. [3, 3, 3]  pivot [2, 2, 2]
    add  lid  4 cells  4 kept  4 exposed  2x1x2  [1, 2, 1] .. [3, 3, 3]
  part  pebble  1 voxel  1x1x1  [6, 0, -1] .. [7, 1, 0]  pivot [6, 0, 0]  detached
    add  pebble  1 cell  1 kept  1 exposed  1x1x1  [6, 0, -1] .. [7, 1, 0]
  piece 1  36 voxels  4x3x4  [0, 0, 0] .. [4, 3, 4]   from body, lid
  piece 2   1 voxel   1x1x1  [6, 0, -1] .. [7, 1, 0]  from pebble
check  1 line  model 37 voxels, 2 pieces
  piece 2  1 voxel  1x1x1  [6, 0, -1] .. [7, 1, 0]  from pebble
"
        );
    }

    /// `parts_main` with the pebble's box at `pebble`, turned `degrees` about y
    /// at `pivot`.
    fn turned_pebble(pebble: SdfShape3d, pivot: [f64; 3], degrees: f64) -> SdfMain {
        let mut state = parts_main().state().clone();
        state.shapes3d[U32Id::from_u32(2).to_usize_id()] = pebble;

        let node = &mut state.nodes[U32Id::from_u32(1).to_usize_id()];
        node.pivot = Some(TyVector3F64::from_array(pivot));
        node.rotation = vec![SdfAxisRotation {
            axis: TyAxis3::Y,
            degrees,
        }];

        SdfMain::new(state).unwrap()
    }

    #[test]
    fn a_part_turned_against_its_root_reads_attached() {
        // The quarter turn carries the pebble from x 6 against the body's z 0
        // face.
        let main = turned_pebble(
            cuboid([6.0, 0.0, 0.0], [7.0, 1.0, 1.0]),
            [4.5, 0.0, 1.5],
            90.0,
        );

        assert_eq!(
            report_on(&main, 1.0, FillMode::Solid),
            "\
model.voxj  37 voxels of 1 m  1 piece  [0, 0, -1] .. [4, 3, 4]
  add  body  32 cells  32 kept  32 exposed  4x2x4  [0, 0, 0] .. [4, 2, 4]
  part  lid  4 voxels  2x1x2  [1, 2, 1] .. [3, 3, 3]  pivot [2, 2, 2]
    add  lid  4 cells  4 kept  4 exposed  2x1x2  [1, 2, 1] .. [3, 3, 3]
  part  pebble  1 voxel  1x1x1  [6, 0, 0] .. [7, 1, 1]  pivot [4.5, 0, 1.5]  posed [3, 0, -1] .. [4, 1, 0]
    add  pebble  1 cell  1 kept  1 exposed  1x1x1  [6, 0, 0] .. [7, 1, 1]
check  0 lines  model 37 voxels, 1 piece
"
        );
    }

    #[test]
    fn a_part_at_an_odd_turn_reads_where_its_voxels_reach() {
        // The turn carries the pebble from x 6 into the body's corner.
        let main = turned_pebble(
            cuboid([6.0, 0.0, 0.0], [7.0, 1.0, 1.0]),
            [5.0, 0.5, 0.5],
            135.0,
        );
        assert_eq!(
            report_on(&main, 1.0, FillMode::Solid),
            "\
model.voxj  37 voxels of 1 m  1 piece  [0, 0, -1.267767] .. [4.646447, 3, 4]
  add  body  32 cells  32 kept  32 exposed  4x2x4  [0, 0, 0] .. [4, 2, 4]
  part  lid  4 voxels  2x1x2  [1, 2, 1] .. [3, 3, 3]  pivot [2, 2, 2]
    add  lid  4 cells  4 kept  4 exposed  2x1x2  [1, 2, 1] .. [3, 3, 3]
  part  pebble  1 voxel  1x1x1  [6, 0, 0] .. [7, 1, 1]  pivot [5, 0.5, 0.5]  posed [3.232233, 0, -1.267767] .. [4.646447, 1, 0.146447]
    add  pebble  1 cell  1 kept  1 exposed  1x1x1  [6, 0, 0] .. [7, 1, 1]
check  0 lines  model 37 voxels, 1 piece
"
        );

        // The turn swings the pebble off the body's x 4 face.
        let main = turned_pebble(
            cuboid([4.0, 0.0, 2.0], [5.0, 1.0, 3.0]),
            [5.5, 0.5, 2.5],
            45.0,
        );
        assert_eq!(
            report_on(&main, 1.0, FillMode::Solid),
            "\
model.voxj  37 voxels of 1 m  2 pieces  [0, 0, 0] .. [5.5, 3, 4]
  add  body  32 cells  32 kept  32 exposed  4x2x4  [0, 0, 0] .. [4, 2, 4]
  part  lid  4 voxels  2x1x2  [1, 2, 1] .. [3, 3, 3]  pivot [2, 2, 2]
    add  lid  4 cells  4 kept  4 exposed  2x1x2  [1, 2, 1] .. [3, 3, 3]
  part  pebble  1 voxel  1x1x1  [4, 0, 2] .. [5, 1, 3]  pivot [5.5, 0.5, 2.5]  posed [4.085786, 0, 2.5] .. [5.5, 1, 3.914214]  detached
    add  pebble  1 cell  1 kept  1 exposed  1x1x1  [4, 0, 2] .. [5, 1, 3]
  piece 1  36 voxels  4x3x4  [0, 0, 0] .. [4, 3, 4]                    from body, lid
  piece 2   1 voxel          [4.085786, 0, 2.5] .. [5.5, 1, 3.914214]  from pebble
check  1 line  model 37 voxels, 2 pieces
  piece 2  1 voxel  [4.085786, 0, 2.5] .. [5.5, 1, 3.914214]  from pebble
"
        );
    }

    #[test]
    fn voxels_where_nodes_overlap_count_once_for_each_node() {
        // The lid sinks into the body. Flattened, the lid writes into the
        // body's node.
        let mut state = parts_main().state().clone();
        state.shapes3d[U32Id::from_u32(1).to_usize_id()] = cuboid([1.0, 1.0, 1.0], [3.0, 2.0, 3.0]);
        let main = SdfMain::new(state).unwrap();

        assert_eq!(
            report_sampled(&main, 1.0, FillMode::Solid, false),
            "\
model.voxj  37 voxels of 1 m  2 pieces  7x2x4  [0, 0, 0] .. [7, 2, 4]
  add  body  32 cells  32 kept  32 exposed  4x2x4  [0, 0, 0] .. [4, 2, 4]
  part  lid  4 voxels  2x1x2  [1, 1, 1] .. [3, 2, 3]  pivot [2, 2, 2]
    add  lid  4 cells  4 kept  4 exposed  2x1x2  [1, 1, 1] .. [3, 2, 3]
  part  pebble  1 voxel  1x1x1  [6, 0, 0] .. [7, 1, 1]  pivot [6, 0, 0]  detached
    add  pebble  1 cell  1 kept  1 exposed  1x1x1  [6, 0, 0] .. [7, 1, 1]
  piece 1  36 voxels  4x2x4  [0, 0, 0] .. [4, 2, 4]  from body, lid
  piece 2   1 voxel   1x1x1  [6, 0, 0] .. [7, 1, 1]  from pebble
check  1 line  model 37 voxels, 2 pieces
  piece 2  1 voxel  1x1x1  [6, 0, 0] .. [7, 1, 1]  from pebble
"
        );
        assert_eq!(
            report_sampled(&main, 1.0, FillMode::Solid, true),
            "\
model.voxj  33 voxels of 1 m  2 pieces  7x2x4  [0, 0, 0] .. [7, 2, 4]
  add  body  32 cells  32 kept  32 exposed  4x2x4  [0, 0, 0] .. [4, 2, 4]
  part  lid  4 voxels  2x1x2  [1, 1, 1] .. [3, 2, 3]  pivot [2, 2, 2]
    add  lid  4 cells  4 kept  4 exposed  2x1x2  [1, 1, 1] .. [3, 2, 3]
  part  pebble  1 voxel  1x1x1  [6, 0, 0] .. [7, 1, 1]  pivot [6, 0, 0]  detached
    add  pebble  1 cell  1 kept  1 exposed  1x1x1  [6, 0, 0] .. [7, 1, 1]
  piece 1  32 voxels  4x2x4  [0, 0, 0] .. [4, 2, 4]  from body, lid
  piece 2   1 voxel   1x1x1  [6, 0, 0] .. [7, 1, 1]  from pebble
check  1 line  model 33 voxels, 2 pieces
  piece 2  1 voxel  1x1x1  [6, 0, 0] .. [7, 1, 1]  from pebble
"
        );
    }

    #[test]
    fn a_part_resting_on_a_sibling_reads_attached() {
        // The pebble sits on the lid and clears the body.
        let mut state = parts_main().state().clone();
        state.shapes3d[U32Id::from_u32(2).to_usize_id()] = SdfShape3d::Box {
            min: TyVector3F64::new(1.0, 3.0, 1.0),
            max: TyVector3F64::new(2.0, 4.0, 2.0),
            round: None,
        };
        state.nodes[U32Id::from_u32(1).to_usize_id()].pivot =
            Some(TyVector3F64::new(1.0, 3.0, 1.0));

        assert_eq!(
            report_on(&SdfMain::new(state).unwrap(), 1.0, FillMode::Solid),
            "\
model.voxj  37 voxels of 1 m  1 piece  4x4x4  [0, 0, 0] .. [4, 4, 4]
  add  body  32 cells  32 kept  32 exposed  4x2x4  [0, 0, 0] .. [4, 2, 4]
  part  lid  4 voxels  2x1x2  [1, 2, 1] .. [3, 3, 3]  pivot [2, 2, 2]
    add  lid  4 cells  4 kept  4 exposed  2x1x2  [1, 2, 1] .. [3, 3, 3]
  part  pebble  1 voxel  1x1x1  [1, 3, 1] .. [2, 4, 2]  pivot [1, 3, 1]
    add  pebble  1 cell  1 kept  1 exposed  1x1x1  [1, 3, 1] .. [2, 4, 2]
check  0 lines  model 37 voxels, 1 piece
"
        );
    }

    #[test]
    fn parts_floating_together_off_their_parent_read_detached() {
        // The lid hovers a cell above the body, and the pebble sits on the lid.
        let mut state = parts_main().state().clone();
        state.shapes3d[U32Id::from_u32(1).to_usize_id()] = SdfShape3d::Box {
            min: TyVector3F64::new(1.0, 3.0, 1.0),
            max: TyVector3F64::new(3.0, 4.0, 3.0),
            round: None,
        };
        state.shapes3d[U32Id::from_u32(2).to_usize_id()] = SdfShape3d::Box {
            min: TyVector3F64::new(1.0, 4.0, 1.0),
            max: TyVector3F64::new(2.0, 5.0, 2.0),
            round: None,
        };
        state.nodes[U32Id::from_u32(0).to_usize_id()].pivot =
            Some(TyVector3F64::new(2.0, 3.0, 2.0));
        state.nodes[U32Id::from_u32(1).to_usize_id()].pivot =
            Some(TyVector3F64::new(1.0, 4.0, 1.0));

        assert_eq!(
            report_on(&SdfMain::new(state).unwrap(), 1.0, FillMode::Solid),
            "\
model.voxj  37 voxels of 1 m  2 pieces  4x5x4  [0, 0, 0] .. [4, 5, 4]
  add  body  32 cells  32 kept  32 exposed  4x2x4  [0, 0, 0] .. [4, 2, 4]
  part  lid  4 voxels  2x1x2  [1, 3, 1] .. [3, 4, 3]  pivot [2, 3, 2]  detached
    add  lid  4 cells  4 kept  4 exposed  2x1x2  [1, 3, 1] .. [3, 4, 3]
  part  pebble  1 voxel  1x1x1  [1, 4, 1] .. [2, 5, 2]  pivot [1, 4, 1]  detached
    add  pebble  1 cell  1 kept  1 exposed  1x1x1  [1, 4, 1] .. [2, 5, 2]
  piece 1  32 voxels  4x2x4  [0, 0, 0] .. [4, 2, 4]  from body
  piece 2   5 voxels  2x2x2  [1, 3, 1] .. [3, 5, 3]  from lid, pebble
check  1 line  model 37 voxels, 2 pieces
  piece 2  5 voxels  2x2x2  [1, 3, 1] .. [3, 5, 3]  from lid, pebble
"
        );
    }

    #[test]
    fn a_step_line_counts_its_pieces_and_the_steps_that_took_its_cells() {
        // The cut splits the bar, and the stain recolors both halves.
        let main = single_part_main(
            vec![
                cuboid([0.0, 0.0, 0.0], [5.0, 1.0, 1.0]),
                cuboid([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]),
            ],
            vec![
                SdfStep::Add {
                    name: "bar".to_string(),
                    shape_id: U32Id::from_u32(0),
                    material: SdfStepMaterial::Material(U32Id::from_u32(0)),
                },
                SdfStep::Carve {
                    name: "cut".to_string(),
                    shape_id: U32Id::from_u32(1),
                },
                SdfStep::Paint {
                    name: "stain".to_string(),
                    shape_id: U32Id::from_u32(0),
                    material: SdfStepMaterial::Material(U32Id::from_u32(1)),
                    onto_ids: None,
                },
            ],
        );

        assert_eq!(
            report_on(&main, 1.0, FillMode::Solid),
            "\
model.voxj  4 voxels of 1 m  2 pieces  5x1x1  [0, 0, 0] .. [5, 1, 1]
  add    bar    5 cells  0 kept  0 exposed  2 pieces  5x1x1  [0, 0, 0] .. [5, 1, 1]            lost to cut, stain
  carve  cut    1 cell   1 kept                       1x1x1  [2, 0, 0] .. [3, 1, 1]
  paint  stain  4 cells  4 kept  4 exposed  2 pieces  5x1x1  [0, 0, 0] .. [5, 1, 1]  over bar
  piece 1  2 voxels  2x1x1  [0, 0, 0] .. [2, 1, 1]  from bar, stain
  piece 2  2 voxels  2x1x1  [3, 0, 0] .. [5, 1, 1]  from bar, stain
check  3 lines  model 4 voxels, 2 pieces
  add    bar    5 cells  0 kept  0 exposed  2 pieces  5x1x1  [0, 0, 0] .. [5, 1, 1]            lost to cut, stain
  paint  stain  4 cells  4 kept  4 exposed  2 pieces  5x1x1  [0, 0, 0] .. [5, 1, 1]  over bar
  piece 2  2 voxels  2x1x1  [3, 0, 0] .. [5, 1, 1]  from bar, stain
"
        );
    }

    #[test]
    fn an_opaque_cell_beside_a_see_through_one_reads_exposed() {
        // The rock sits inside the water.
        let mut state = single_part_main(
            vec![
                cuboid([0.0, 0.0, 0.0], [3.0, 3.0, 3.0]),
                cuboid([1.0, 1.0, 1.0], [2.0, 2.0, 2.0]),
            ],
            vec![
                SdfStep::Add {
                    name: "water".to_string(),
                    shape_id: U32Id::from_u32(0),
                    material: SdfStepMaterial::Material(U32Id::from_u32(1)),
                },
                SdfStep::Add {
                    name: "rock".to_string(),
                    shape_id: U32Id::from_u32(1),
                    material: SdfStepMaterial::Material(U32Id::from_u32(0)),
                },
            ],
        )
        .state()
        .clone();
        state.materials[U32Id::from_u32(1).to_usize_id()] = SdfMaterial::Material {
            properties: vec![SdfProperty {
                name: "transmission".to_string(),
                value: SdfPropertyValue::Number(1.0),
            }],
        };

        assert_eq!(
            report_on(&SdfMain::new(state).unwrap(), 1.0, FillMode::Solid),
            "\
model.voxj  27 voxels of 1 m  1 piece  3x3x3  [0, 0, 0] .. [3, 3, 3]
  add  water  27 cells  26 kept  26 exposed  3x3x3  [0, 0, 0] .. [3, 3, 3]  lost to rock
  add  rock    1 cell    1 kept   1 exposed  1x1x1  [1, 1, 1] .. [2, 2, 2]
check  0 lines  model 27 voxels, 1 piece
"
        );
    }

    #[test]
    fn the_check_block_repeats_a_step_that_keeps_cells_and_shows_none() {
        let main = single_part_main(
            vec![
                cuboid([0.0, 0.0, 0.0], [3.0, 3.0, 3.0]),
                cuboid([1.0, 1.0, 1.0], [2.0, 2.0, 2.0]),
            ],
            vec![
                SdfStep::Add {
                    name: "block".to_string(),
                    shape_id: U32Id::from_u32(0),
                    material: SdfStepMaterial::Material(U32Id::from_u32(0)),
                },
                SdfStep::Add {
                    name: "core".to_string(),
                    shape_id: U32Id::from_u32(1),
                    material: SdfStepMaterial::Material(U32Id::from_u32(1)),
                },
            ],
        );

        assert_eq!(
            report_on(&main, 1.0, FillMode::Solid),
            "\
model.voxj  27 voxels of 1 m  1 piece  3x3x3  [0, 0, 0] .. [3, 3, 3]
  add  block  27 cells  26 kept  26 exposed  3x3x3  [0, 0, 0] .. [3, 3, 3]  lost to core
  add  core    1 cell    1 kept   0 exposed  1x1x1  [1, 1, 1] .. [2, 2, 2]
check  1 line  model 27 voxels, 1 piece
  add  core  1 cell  1 kept  0 exposed  1x1x1  [1, 1, 1] .. [2, 2, 2]
"
        );
    }

    #[test]
    fn copies_in_their_own_pieces_leave_the_step_out_and_fold_their_piece_lines() {
        // Two posts stand on the slab, and two links float off each end.
        let mirrored = |shape_id| SdfShape3d::Mirror {
            shape_id: U32Id::from_u32(shape_id),
            axes: SdfAxes3d::X,
            center: None,
        };
        let main = single_part_main(
            vec![
                cuboid([-3.0, 0.0, 0.0], [3.0, 1.0, 1.0]),
                cuboid([1.0, 1.0, 0.0], [2.0, 2.0, 1.0]),
                mirrored(1),
                cuboid([5.0, 0.0, 0.0], [6.0, 1.0, 1.0]),
                mirrored(3),
            ],
            vec![
                SdfStep::Add {
                    name: "slab".to_string(),
                    shape_id: U32Id::from_u32(0),
                    material: SdfStepMaterial::Material(U32Id::from_u32(0)),
                },
                SdfStep::Add {
                    name: "posts".to_string(),
                    shape_id: U32Id::from_u32(2),
                    material: SdfStepMaterial::Material(U32Id::from_u32(0)),
                },
                SdfStep::Add {
                    name: "links".to_string(),
                    shape_id: U32Id::from_u32(4),
                    material: SdfStepMaterial::Material(U32Id::from_u32(0)),
                },
            ],
        );

        assert_eq!(
            report_on(&main, 1.0, FillMode::Solid),
            "\
model.voxj  10 voxels of 1 m  3 pieces  12x2x1  [-6, 0, 0] .. [6, 2, 1]
  add  slab   6 cells  6 kept  6 exposed            6x1x1   [-3, 0, 0] .. [3, 1, 1]
  add  posts  2 cells  2 kept  2 exposed  2 pieces  4x1x1   [-2, 1, 0] .. [2, 2, 1]
  add  links  2 cells  2 kept  2 exposed  2 pieces  12x1x1  [-6, 0, 0] .. [6, 1, 1]
  piece 1  8 voxels  6x2x1  [-3, 0, 0] .. [3, 2, 1]   from slab, posts
  piece 2  1 voxel   1x1x1  [-6, 0, 0] .. [-5, 1, 1]  from links
  piece 3  1 voxel   1x1x1  [5, 0, 0] .. [6, 1, 1]    from links
check  1 line  model 10 voxels, 3 pieces
  2 pieces  1 voxel each  12x1x1  [-6, 0, 0] .. [6, 1, 1]  from links
"
        );
    }

    #[test]
    fn union_operands_and_set_points_in_no_more_pieces_than_forms_leave_the_step_out() {
        // Two posts stand on the slab, a cap tops one, and two links float off
        // each end.
        let main = single_part_main(
            vec![
                cuboid([-3.0, 0.0, 0.0], [3.0, 1.0, 1.0]),
                cuboid([-2.0, 1.0, 0.0], [-1.0, 2.0, 1.0]),
                cuboid([1.0, 1.0, 0.0], [2.0, 2.0, 1.0]),
                cuboid([-2.0, 2.0, 0.0], [-1.0, 3.0, 1.0]),
                SdfShape3d::Union {
                    shape_ids: vec![U32Id::from_u32(1), U32Id::from_u32(2), U32Id::from_u32(3)],
                },
            ],
            vec![
                SdfStep::Add {
                    name: "slab".to_string(),
                    shape_id: U32Id::from_u32(0),
                    material: SdfStepMaterial::Material(U32Id::from_u32(0)),
                },
                SdfStep::Add {
                    name: "posts".to_string(),
                    shape_id: U32Id::from_u32(4),
                    material: SdfStepMaterial::Material(U32Id::from_u32(0)),
                },
                SdfStep::Set {
                    name: "links".to_string(),
                    points: vec![
                        TyVector3F64::new(-5.5, 0.5, 0.5),
                        TyVector3F64::new(5.5, 0.5, 0.5),
                    ],
                    size: 1.0,
                    material: SdfStepMaterial::Material(U32Id::from_u32(0)),
                },
            ],
        );

        assert_eq!(
            report_on(&main, 1.0, FillMode::Solid),
            "\
model.voxj  11 voxels of 1 m  3 pieces  12x3x1  [-6, 0, 0] .. [6, 3, 1]
  add  slab   6 cells  6 kept  6 exposed            6x1x1   [-3, 0, 0] .. [3, 1, 1]
  add  posts  3 cells  3 kept  3 exposed  2 pieces  4x2x1   [-2, 1, 0] .. [2, 3, 1]
  set  links  2 cells  2 kept  2 exposed  2 pieces  12x1x1  [-6, 0, 0] .. [6, 1, 1]
  piece 1  9 voxels  6x3x1  [-3, 0, 0] .. [3, 3, 1]   from slab, posts
  piece 2  1 voxel   1x1x1  [-6, 0, 0] .. [-5, 1, 1]  from links
  piece 3  1 voxel   1x1x1  [5, 0, 0] .. [6, 1, 1]    from links
check  1 line  model 11 voxels, 3 pieces
  2 pieces  1 voxel each  12x1x1  [-6, 0, 0] .. [6, 1, 1]  from links
"
        );
    }

    #[test]
    fn clipped_and_cut_operands_keep_their_forms() {
        // Two bands clip to the post as hoops, and two rungs beside it lose
        // their far ends.
        let main = single_part_main(
            vec![
                cuboid([0.0, 0.0, 0.0], [1.0, 4.0, 1.0]),
                cuboid([-1.0, 0.0, 0.0], [2.0, 1.0, 1.0]),
                cuboid([-1.0, 2.0, 0.0], [2.0, 3.0, 1.0]),
                SdfShape3d::Union {
                    shape_ids: vec![U32Id::from_u32(1), U32Id::from_u32(2)],
                },
                SdfShape3d::Intersect {
                    shape_ids: vec![U32Id::from_u32(0), U32Id::from_u32(3)],
                },
                cuboid([1.0, 0.0, 0.0], [3.0, 1.0, 1.0]),
                cuboid([1.0, 2.0, 0.0], [3.0, 3.0, 1.0]),
                SdfShape3d::Union {
                    shape_ids: vec![U32Id::from_u32(5), U32Id::from_u32(6)],
                },
                cuboid([2.0, 0.0, 0.0], [3.0, 3.0, 1.0]),
                SdfShape3d::Subtract {
                    base_id: U32Id::from_u32(7),
                    cutter_ids: vec![U32Id::from_u32(8)],
                },
            ],
            vec![
                SdfStep::Add {
                    name: "post".to_string(),
                    shape_id: U32Id::from_u32(0),
                    material: SdfStepMaterial::Material(U32Id::from_u32(0)),
                },
                SdfStep::Add {
                    name: "hoops".to_string(),
                    shape_id: U32Id::from_u32(4),
                    material: SdfStepMaterial::Material(U32Id::from_u32(0)),
                },
                SdfStep::Add {
                    name: "rungs".to_string(),
                    shape_id: U32Id::from_u32(9),
                    material: SdfStepMaterial::Material(U32Id::from_u32(0)),
                },
            ],
        );

        assert_eq!(
            report_on(&main, 1.0, FillMode::Solid),
            "\
model.voxj  6 voxels of 1 m  1 piece  2x4x1  [0, 0, 0] .. [2, 4, 1]
  add  post   4 cells  2 kept  2 exposed            1x4x1  [0, 0, 0] .. [1, 4, 1]  lost to hoops
  add  hoops  2 cells  2 kept  2 exposed  2 pieces  1x3x1  [0, 0, 0] .. [1, 3, 1]
  add  rungs  2 cells  2 kept  2 exposed  2 pieces  1x3x1  [1, 0, 0] .. [2, 3, 1]
check  0 lines  model 6 voxels, 1 piece
"
        );
    }

    #[test]
    fn the_surface_fill_thins_the_voxel_counts_to_the_shell() {
        let main = single_part_main(
            vec![SdfShape3d::Box {
                min: TyVector3F64::ZERO,
                max: TyVector3F64::splat(3.0),
                round: None,
            }],
            vec![SdfStep::Add {
                name: "block".to_string(),
                shape_id: U32Id::from_u32(0),
                material: SdfStepMaterial::Material(U32Id::from_u32(0)),
            }],
        );

        assert_eq!(
            report_on(&main, 1.0, FillMode::Surface),
            "\
model.voxj  26 voxels of 1 m  1 piece  3x3x3  [0, 0, 0] .. [3, 3, 3]
  add  block  27 cells  27 kept  26 exposed  3x3x3  [0, 0, 0] .. [3, 3, 3]
check  0 lines  model 26 voxels, 1 piece
"
        );
    }

    #[test]
    fn a_part_in_several_places_shows_its_steps_under_the_first() {
        assert_eq!(
            report_on(&shared_leaf_main(2.0), 1.0, FillMode::Solid),
            "\
model.voxj  2 voxels of 1 m  2 pieces  3x3x1  [0, 0, 0] .. [3, 3, 1]
  part  a  0 voxels  pivot [2.5, 0, 0]
    part  leaf  1 voxel  1x1x1  [2, 0, 0] .. [3, 1, 1]  pivot [2.5, 0, 0]
      add  cube  1 cell  1 kept  1 exposed  1x1x1  [2, 0, 0] .. [3, 1, 1]
  part  b  0 voxels  pivot [0.5, 2, 0]
    part  leaf  1 voxel  1x1x1  [0, 2, 0] .. [1, 3, 1]  pivot [0.5, 2, 0]  repeats a/leaf
  piece 1  1 voxel  1x1x1  [0, 2, 0] .. [1, 3, 1]  from b/leaf/cube
  piece 2  1 voxel  1x1x1  [2, 0, 0] .. [3, 1, 1]  from a/leaf/cube
check  1 line  model 2 voxels, 2 pieces
  piece 2  1 voxel  1x1x1  [2, 0, 0] .. [3, 1, 1]  from a/leaf/cube
"
        );
    }

    #[test]
    fn the_report_needs_the_writes_recorded() {
        let main = parts_main();
        let sampling = sample(
            &DependenciesImpl,
            &main,
            &SdfSampleOptions {
                resolution: GridResolution::VoxelSize(1.0),
                frame: VoxelFrame::World,
                posed: false,
                record_writes: false,
            },
        )
        .unwrap();

        assert_eq!(
            report(&main, &sampling, FillMode::Solid, "model.voxj")
                .unwrap_err()
                .to_string(),
            "report must read grids sampled with their writes recorded"
        );
    }

    #[test]
    fn the_report_needs_the_world_frame() {
        let main = parts_main();
        let sampling = sample(
            &DependenciesImpl,
            &main,
            &SdfSampleOptions {
                resolution: GridResolution::VoxelSize(1.0),
                frame: VoxelFrame::Local,
                posed: false,
                record_writes: true,
            },
        )
        .unwrap();

        assert_eq!(
            report(&main, &sampling, FillMode::Solid, "model.voxj")
                .unwrap_err()
                .to_string(),
            "report must read grids sampled under the world frame, not the local frame"
        );
    }
}
