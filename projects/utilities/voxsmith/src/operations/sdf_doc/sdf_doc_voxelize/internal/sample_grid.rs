use crate::{
    Error, Result,
    dependencies::sdf_doc::MapCells,
    operations::sdf_doc::{
        ArgumentCheck, Bounds3d, LatticeShapes, SdfCell, SdfEvaluation, SdfGrid, SdfShapes,
        SdfStepRecord, ShapeSource, meters, pattern_material, side_direction,
    },
};
use branded_id::{IdVec, U32Id};
use sdfcore::{
    BSdfMaterial, BSdfObject, BSdfPattern, BSdfShape3d, BSdfStep, SdfPattern, SdfSide, SdfState,
    SdfStep, SdfStepMaterial,
};
use std::result::Result as StdResult;
use ty_math::{
    TyPoseF64, TyQuaternionExt, TyQuaternionF64, TyVector3F64, TyVector3I32, TyVector3U32,
};

/// The most cells one part's grid holds.
const MOST_CELLS: f64 = (1 << 27) as f64;

/// The most cells a step decides before it writes them. The cap limits the
/// decisions held in memory.
const CHUNK_CELLS: usize = 1 << 16;

/// The sides a `coat` reads when the model leaves them out.
const ALL_SIDES: [SdfSide; 6] = [
    SdfSide::NegativeX,
    SdfSide::NegativeY,
    SdfSide::NegativeZ,
    SdfSide::PositiveX,
    SdfSide::PositiveY,
    SdfSide::PositiveZ,
];

/// Runs the steps of the object at `object_id` over its grid on the lattice of
/// cells `voxel_size` across, with the object's shapes posed by `pose`, which
/// takes a shape's point to its lattice point in meters.
/// `place_path` holds the part names of the place an error reports. Each
/// step's record holds its writes under `record_writes`. `dependencies`
/// spreads each step's cells across threads.
#[expect(
    clippy::too_many_arguments,
    reason = "sampling reads the document, the place, and the lattice"
)]
pub fn sample_grid<D: MapCells>(
    dependencies: &D,
    state: &SdfState,
    shapes: &SdfShapes,
    object_id: U32Id<BSdfObject>,
    place_path: &[String],
    voxel_size: f64,
    pose: TyPoseF64,
    record_writes: bool,
) -> Result<SdfGrid> {
    let lattice = LatticeShapes::new(shapes, voxel_size, pose);
    let object = &state.objects[object_id.to_usize_id()];
    let path = place_path.join("/");

    let range = object
        .step_ids
        .iter()
        .flat_map(|step_id| match &state.steps[step_id.to_usize_id()] {
            SdfStep::Add { shape_id, .. } => {
                vec![lattice.cells(*shape_id).expect("an add shape has a box")]
            }

            SdfStep::Set { points, size, .. } => points
                .iter()
                .map(|point| set_cells(*point, *size, voxel_size, pose))
                .collect(),

            SdfStep::Carve { .. } | SdfStep::Coat { .. } | SdfStep::Paint { .. } => Vec::new(),
        })
        .reduce(|range, cells| range.union(&cells));

    let (min, size) = match range {
        Some(range) => {
            let extent = range.max - range.min;
            let count = extent.x * extent.y * extent.z;

            if count > MOST_CELLS {
                let corner = |cells: TyVector3F64| {
                    let position = cells * voxel_size;

                    format!(
                        "[{}, {}, {}]",
                        meters(position.x),
                        meters(position.y),
                        meters(position.z)
                    )
                };

                return Err(Error::invalid(format!(
                    "{path}: grid over {} .. {} must hold at most {MOST_CELLS} cells, not {count}",
                    corner(range.min),
                    corner(range.max)
                )));
            }

            (
                lattice_cell(range.min)
                    .map_err(|message| Error::invalid(format!("{path}: {message}")))?,
                TyVector3U32::new(extent.x as u32, extent.y as u32, extent.z as u32),
            )
        }

        None => (TyVector3I32::ZERO, TyVector3U32::ZERO),
    };

    let mut grid = SdfGrid {
        object_id,
        min,
        size,
        cells: vec![SdfCell::default(); (size.x * size.y * size.z) as usize],
        steps: Vec::with_capacity(object.step_ids.len()),
    };

    // The `add` step owning each cell, kept only for a part whose steps read
    // it.
    let mut owners = object
        .step_ids
        .iter()
        .any(|step_id| match &state.steps[step_id.to_usize_id()] {
            SdfStep::Coat { onto_ids, .. } | SdfStep::Paint { onto_ids, .. } => onto_ids.is_some(),
            SdfStep::Add { .. } | SdfStep::Carve { .. } | SdfStep::Set { .. } => false,
        })
        .then(|| vec![None; grid.cells.len()]);

    // Each `add` or `carve` shape run so far, with `true` for an `add`.
    let mut solids = Vec::new();

    for (index, step_id) in object.step_ids.iter().enumerate() {
        let step = &state.steps[step_id.to_usize_id()];

        let mut run = StepRun {
            lattice: &lattice,
            patterns: &state.patterns,
            grid: &mut grid,
            owners: owners.as_deref_mut(),
            solids: &solids,
            step_id: *step_id,
            step: index as u32,
            voxel_size,
            pose,
            record_writes,
            record: SdfStepRecord::default(),
        };

        run.run(dependencies, step)
            .map_err(|message| Error::invalid(format!("{path}/{}: {message}", step.name())))?;

        let record = run.record;
        grid.steps.push(record);

        match step {
            SdfStep::Add { shape_id, .. } => solids.push((*shape_id, true)),
            SdfStep::Carve { shape_id, .. } => solids.push((*shape_id, false)),
            SdfStep::Coat { .. } | SdfStep::Paint { .. } | SdfStep::Set { .. } => {}
        }
    }

    Ok(grid)
}

/// One step running over a grid.
struct StepRun<'a, 'b> {
    lattice: &'a LatticeShapes<'b>,

    patterns: &'a IdVec<BSdfPattern, SdfPattern>,

    grid: &'a mut SdfGrid,

    owners: Option<&'a mut [Option<U32Id<BSdfStep>>]>,

    solids: &'a [(U32Id<BSdfShape3d>, bool)],

    step_id: U32Id<BSdfStep>,

    step: u32,

    voxel_size: f64,

    pose: TyPoseF64,

    record_writes: bool,

    record: SdfStepRecord,
}

impl StepRun<'_, '_> {
    fn run<D: MapCells>(&mut self, dependencies: &D, step: &SdfStep) -> StdResult<(), String> {
        match step {
            SdfStep::Add {
                shape_id, material, ..
            } => {
                let cells = self.cells_within(self.lattice.cells(*shape_id));

                self.write_cells(dependencies, &cells, true, |run, cell| {
                    let evaluation = run.evaluate(*shape_id, cell)?;

                    Ok((evaluation.distance <= 0.0)
                        .then(|| Some(run.pick(*material, evaluation.frame_position, cell))))
                })?;
            }

            SdfStep::Carve { shape_id, .. } => {
                let cells = self.cells_within(self.lattice.cells(*shape_id));

                self.write_cells(dependencies, &cells, true, |run, cell| {
                    Ok(
                        (run.is_live(cell) && run.evaluate(*shape_id, cell)?.distance <= 0.0)
                            .then_some(None),
                    )
                })?;
            }

            SdfStep::Coat {
                material,
                sides,
                depth,
                within_id,
                onto_ids,
                ..
            } => {
                ArgumentCheck::new("coat").at_least(
                    "depth",
                    *depth,
                    self.voxel_size,
                    &format!("at least the voxel size {}", meters(self.voxel_size)),
                )?;

                let sides = sides.as_deref().unwrap_or(&ALL_SIDES);
                let bounds = within_id.and_then(|within_id| self.lattice.cells(within_id));
                let cells = self.cells_within(bounds);

                self.write_cells(dependencies, &cells, false, |run, cell| {
                    if !run.is_live(cell) || !run.is_onto(onto_ids.as_deref(), cell) {
                        return Ok(None);
                    }

                    let mut reaches_outside = false;

                    for side in sides {
                        let (axis, sign) = side_direction(*side);
                        let mut offset = TyVector3F64::ZERO;
                        offset[axis.index()] = sign * depth;

                        if !run.is_solid(run.center(cell) + offset)? {
                            reaches_outside = true;
                            break;
                        }
                    }

                    let within = match within_id {
                        Some(within_id) => run.evaluate(*within_id, cell)?.distance <= 0.0,
                        None => true,
                    };

                    Ok((reaches_outside && within)
                        .then(|| Some(run.pick(*material, run.center(cell), cell))))
                })?;
            }

            SdfStep::Paint {
                shape_id,
                material,
                onto_ids,
                ..
            } => {
                let cells = self.cells_within(self.lattice.cells(*shape_id));

                self.write_cells(dependencies, &cells, false, |run, cell| {
                    if !run.is_live(cell) || !run.is_onto(onto_ids.as_deref(), cell) {
                        return Ok(None);
                    }

                    let evaluation = run.evaluate(*shape_id, cell)?;

                    Ok((evaluation.distance <= 0.0)
                        .then(|| Some(run.pick(*material, evaluation.frame_position, cell))))
                })?;
            }

            SdfStep::Set {
                points,
                size,
                material,
                ..
            } => {
                for point in points {
                    let cells = set_cells(*point, *size, self.voxel_size, self.pose);

                    for cell in self.cells_within(Some(cells)) {
                        if !self.set_covers(*point, *size, cell) {
                            continue;
                        }

                        let material_id = self.pick(*material, self.center(cell), cell);
                        self.write(cell, Some(material_id));
                    }
                }
            }
        }

        Ok(())
    }

    /// Writes into each of `cells` the material `decide` picks. A cell that
    /// `decide` picks nothing for keeps its value. `dependencies` spreads the
    /// `decide` calls across threads. The writes land in cell order and stop
    /// at the first error. Under `owns`, each write also makes the step the
    /// cell's owner, or clears the owner of an emptied cell.
    fn write_cells<D, F>(
        &mut self,
        dependencies: &D,
        cells: &[TyVector3I32],
        owns: bool,
        decide: F,
    ) -> StdResult<(), String>
    where
        D: MapCells,
        F: Fn(&Self, TyVector3I32) -> StdResult<Option<Option<U32Id<BSdfMaterial>>>, String> + Sync,
    {
        for chunk in cells.chunks(CHUNK_CELLS) {
            let run = &*self;
            let decisions = dependencies.map_cells(chunk, |cell| decide(run, cell));

            for (&cell, decision) in chunk.iter().zip(decisions) {
                if let Some(material) = decision? {
                    self.write(cell, material);

                    if owns {
                        self.own(cell, material.map(|_| self.step_id));
                    }
                }
            }
        }

        Ok(())
    }

    /// The grid's cells inside the cell box `bounds`, or every cell of the
    /// grid for a shape that reaches without end.
    fn cells_within(&self, bounds: Option<Bounds3d>) -> Vec<TyVector3I32> {
        let grid_min = self.grid.min.as_dvec3();
        let grid_max = grid_min + self.grid.size.as_dvec3();

        let (min, max) = match bounds {
            Some(bounds) => (bounds.min.max(grid_min), bounds.max.min(grid_max)),
            None => (grid_min, grid_max),
        };

        if min.cmpge(max).any() {
            return Vec::new();
        }

        let (min, max) = (min.as_ivec3(), max.as_ivec3());

        (min.x..max.x)
            .flat_map(|x| (min.y..max.y).map(move |y| (x, y)))
            .flat_map(|(x, y)| (min.z..max.z).map(move |z| TyVector3I32::new(x, y, z)))
            .collect()
    }

    /// The shape at `shape3d_id` evaluated at the center of `cell`. Errors on a
    /// distance that reads NaN.
    fn evaluate(
        &self,
        shape3d_id: U32Id<BSdfShape3d>,
        cell: TyVector3I32,
    ) -> StdResult<SdfEvaluation, String> {
        self.evaluate_point(shape3d_id, self.center(cell))
    }

    /// The shape at `shape3d_id` evaluated at `point`. Errors on a distance
    /// that reads NaN.
    fn evaluate_point(
        &self,
        shape3d_id: U32Id<BSdfShape3d>,
        point: TyVector3F64,
    ) -> StdResult<SdfEvaluation, String> {
        let evaluation = self.lattice.evaluate(shape3d_id, point);

        if evaluation.distance.is_nan() {
            return Err(format!(
                "distance must be a number, not NaN at [{}, {}, {}]",
                point.x, point.y, point.z
            ));
        }

        Ok(evaluation)
    }

    /// Whether `point` lies inside the part's shapes so far. The last `add` or
    /// `carve` whose shape holds `point` decides.
    fn is_solid(&self, point: TyVector3F64) -> StdResult<bool, String> {
        let lattice_point = (self.pose.position + self.pose.rotation * point) / self.voxel_size;

        for &(shape3d_id, fills) in self.solids.iter().rev() {
            let outside_box = self.lattice.cells(shape3d_id).is_some_and(|cells| {
                lattice_point.cmplt(cells.min).any() || lattice_point.cmpgt(cells.max).any()
            });

            if !outside_box && self.evaluate_point(shape3d_id, point)?.distance <= 0.0 {
                return Ok(fills);
            }
        }

        Ok(false)
    }

    /// The center of `cell` where the shapes evaluate, posed back into the
    /// shapes' frame.
    fn center(&self, cell: TyVector3I32) -> TyVector3F64 {
        self.pose.rotation.inverse()
            * ((cell.as_dvec3() + 0.5) * self.voxel_size - self.pose.position)
    }

    /// Whether the `set` cube `size` wide around `point` covers the center of
    /// `cell`. An unturned cube covers its whole cell box.
    fn set_covers(&self, point: TyVector3F64, size: f64, cell: TyVector3I32) -> bool {
        if self.pose.rotation == TyQuaternionF64::IDENTITY {
            return true;
        }

        let half = size / 2.0;
        let from_point = self.center(cell) - point;

        from_point.cmpge(TyVector3F64::splat(-half)).all()
            && from_point.cmplt(TyVector3F64::splat(half)).all()
    }

    /// The material `material` writes into `cell` at the frame position
    /// `frame_position`.
    fn pick(
        &self,
        material: SdfStepMaterial,
        frame_position: TyVector3F64,
        cell: TyVector3I32,
    ) -> U32Id<BSdfMaterial> {
        match material {
            SdfStepMaterial::Material(material_id) => material_id,

            SdfStepMaterial::Pattern(pattern_id) => pattern_material(
                &self.patterns[pattern_id.to_usize_id()],
                frame_position,
                (cell.as_dvec3() + 0.5) * self.voxel_size,
            ),
        }
    }

    /// Whether `cell` holds a material.
    fn is_live(&self, cell: TyVector3I32) -> bool {
        self.grid
            .cell(cell)
            .is_some_and(|cell| cell.material.is_some())
    }

    /// Whether one of the `add` steps `onto_ids` owns `cell`, or `true` for a
    /// step without `onto`.
    fn is_onto(&self, onto_ids: Option<&[U32Id<BSdfStep>]>, cell: TyVector3I32) -> bool {
        let Some(onto_ids) = onto_ids else {
            return true;
        };

        let owners = self
            .owners
            .as_deref()
            .expect("a part reading onto keeps owners");
        let index = self.grid.index(cell).expect("a step reads inside its grid");

        owners[index].is_some_and(|owner_id| onto_ids.contains(&owner_id))
    }

    /// Makes `owner_id` the owner of `cell` in a part that keeps owners.
    fn own(&mut self, cell: TyVector3I32, owner_id: Option<U32Id<BSdfStep>>) {
        if let Some(owners) = self.owners.as_deref_mut() {
            let index = self
                .grid
                .index(cell)
                .expect("a step writes inside its grid");
            owners[index] = owner_id;
        }
    }

    /// Writes `material` into `cell` and records the cell once for the step.
    fn write(&mut self, cell: TyVector3I32, material: Option<U32Id<BSdfMaterial>>) {
        let index = self
            .grid
            .index(cell)
            .expect("a step writes inside its grid");
        let written = &mut self.grid.cells[index];

        if self.record_writes && written.step != Some(self.step) {
            self.record.push(index as u32, cell, written.step);
        }

        *written = SdfCell {
            material,
            step: Some(self.step),
        };
    }
}

/// The cell box of a `set` cube `size` wide around `point` posed by `pose`. A
/// center on the cube's min face lies inside it, and a center on its max face
/// outside.
fn set_cells(point: TyVector3F64, size: f64, voxel_size: f64, pose: TyPoseF64) -> Bounds3d {
    let center = (pose.position + pose.rotation * point) / voxel_size;
    // A turned cube's half extents on the lattice's axes bound it. Cell `i`
    // holds its center `i + 0.5` within `[center - half, center + half)`.
    let half = pose
        .rotation
        .rotate_extents_abs(TyVector3F64::splat(size / voxel_size / 2.0));

    Bounds3d {
        min: (center - half - 0.5).ceil(),
        max: (center + half - 0.5).ceil(),
    }
}

/// The lattice index of the whole-number cell corner `corner`. Errors on a
/// corner past the 32-bit range.
fn lattice_cell(corner: TyVector3F64) -> StdResult<TyVector3I32, String> {
    let range = f64::from(i32::MIN)..=f64::from(i32::MAX);

    if corner
        .to_array()
        .iter()
        .all(|coordinate| range.contains(coordinate))
    {
        Ok(corner.as_ivec3())
    } else {
        Err(format!(
            "grid must lie within 2^31 cells of the origin, not at [{}, {}, {}] cells",
            corner.x, corner.y, corner.z
        ))
    }
}
