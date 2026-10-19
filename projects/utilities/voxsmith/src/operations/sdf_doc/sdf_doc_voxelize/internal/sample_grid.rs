use crate::{
    Error, Result,
    operations::sdf_doc::{
        Bounds3d, LatticeShapes, SdfCell, SdfEvaluation, SdfGrid, SdfShapes, SdfStepRecord,
        ShapeSource, meters, pattern_material, side_direction, whole_count,
    },
};
use branded_id::{IdVec, U32Id};
use sdfcore::{
    BSdfMaterial, BSdfObject, BSdfPattern, BSdfShape3d, BSdfStep, SdfPattern, SdfSide, SdfState,
    SdfStep, SdfStepMaterial,
};
use std::result::Result as StdResult;
use ty_math::{TyVector3F64, TyVector3I32, TyVector3U32};

/// The most cells one part's grid holds.
const MOST_CELLS: f64 = (1 << 27) as f64;

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
/// cells `voxel_size` across, with the object's shapes moved by `shift`.
/// `place_path` holds the part names of the place an error reports. Each
/// step's record holds its writes under `record_writes`.
pub fn sample_grid(
    state: &SdfState,
    shapes: &SdfShapes,
    object_id: U32Id<BSdfObject>,
    place_path: &[String],
    voxel_size: f64,
    shift: TyVector3F64,
    record_writes: bool,
) -> Result<SdfGrid> {
    let lattice = LatticeShapes::new(shapes, voxel_size, shift);
    let object = &state.objects[object_id.to_usize_id()];
    let path = place_path.join("/");

    let set_cell = |point: TyVector3F64| ((point + shift) / voxel_size).floor();

    let range = object
        .step_ids
        .iter()
        .flat_map(|step_id| match &state.steps[step_id.to_usize_id()] {
            SdfStep::Add { shape_id, .. } => {
                vec![lattice.cells(*shape_id).expect("an add shape has a box")]
            }

            SdfStep::Set { points, .. } => points
                .iter()
                .map(|point| Bounds3d {
                    min: set_cell(*point),
                    max: set_cell(*point) + 1.0,
                })
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

    for (index, step_id) in object.step_ids.iter().enumerate() {
        let step = &state.steps[step_id.to_usize_id()];

        let mut run = StepRun {
            lattice: &lattice,
            patterns: &state.patterns,
            grid: &mut grid,
            owners: owners.as_deref_mut(),
            step_id: *step_id,
            step: index as u32,
            voxel_size,
            shift,
            record_writes,
            record: SdfStepRecord::default(),
        };

        run.run(step)
            .map_err(|message| Error::invalid(format!("{path}/{}: {message}", step.name())))?;

        let record = run.record;
        grid.steps.push(record);
    }

    Ok(grid)
}

/// One step running over a grid.
struct StepRun<'a, 'b> {
    lattice: &'a LatticeShapes<'b>,

    patterns: &'a IdVec<BSdfPattern, SdfPattern>,

    grid: &'a mut SdfGrid,

    owners: Option<&'a mut [Option<U32Id<BSdfStep>>]>,

    step_id: U32Id<BSdfStep>,

    step: u32,

    voxel_size: f64,

    shift: TyVector3F64,

    record_writes: bool,

    record: SdfStepRecord,
}

impl StepRun<'_, '_> {
    fn run(&mut self, step: &SdfStep) -> StdResult<(), String> {
        match step {
            SdfStep::Add {
                shape_id, material, ..
            } => {
                for cell in self.cells_within(self.lattice.cells(*shape_id)) {
                    let evaluation = self.evaluate(*shape_id, cell)?;

                    if evaluation.distance <= 0.0 {
                        let material_id = self.pick(*material, evaluation.frame_position, cell);
                        self.write(cell, Some(material_id));
                        self.own(cell, Some(self.step_id));
                    }
                }
            }

            SdfStep::Carve { shape_id, .. } => {
                for cell in self.cells_within(self.lattice.cells(*shape_id)) {
                    if self.is_live(cell) && self.evaluate(*shape_id, cell)?.distance <= 0.0 {
                        self.write(cell, None);
                        self.own(cell, None);
                    }
                }
            }

            SdfStep::Coat {
                material,
                sides,
                depth,
                within_id,
                onto_ids,
                ..
            } => {
                let snapshot: Vec<bool> = self
                    .grid
                    .cells
                    .iter()
                    .map(|cell| cell.material.is_some())
                    .collect();
                let sides = sides.as_deref().unwrap_or(&ALL_SIDES);
                let depth = depth.map_or(1, whole_count);

                let is_empty = |grid: &SdfGrid, cell: TyVector3I32| {
                    grid.index(cell).is_none_or(|index| !snapshot[index])
                };

                let bounds = within_id.and_then(|within_id| self.lattice.cells(within_id));

                for cell in self.cells_within(bounds) {
                    if is_empty(self.grid, cell) || !self.is_onto(onto_ids.as_deref(), cell) {
                        continue;
                    }

                    let reaches_empty = sides.iter().any(|side| {
                        let (axis, sign) = side_direction(*side);
                        let mut step = TyVector3I32::ZERO;
                        step[axis.index()] = sign as i32;

                        (1..=depth as i32)
                            .any(|distance| is_empty(self.grid, cell + step * distance))
                    });

                    let within = match within_id {
                        Some(within_id) => self.evaluate(*within_id, cell)?.distance <= 0.0,
                        None => true,
                    };

                    if reaches_empty && within {
                        let material_id = self.pick(*material, self.center(cell), cell);
                        self.write(cell, Some(material_id));
                    }
                }
            }

            SdfStep::Paint {
                shape_id,
                material,
                onto_ids,
                ..
            } => {
                for cell in self.cells_within(self.lattice.cells(*shape_id)) {
                    if !self.is_live(cell) || !self.is_onto(onto_ids.as_deref(), cell) {
                        continue;
                    }

                    let evaluation = self.evaluate(*shape_id, cell)?;

                    if evaluation.distance <= 0.0 {
                        let material_id = self.pick(*material, evaluation.frame_position, cell);
                        self.write(cell, Some(material_id));
                    }
                }
            }

            SdfStep::Set {
                points, material, ..
            } => {
                for point in points {
                    let cell = ((*point + self.shift) / self.voxel_size).floor().as_ivec3();
                    let material_id = self.pick(*material, self.center(cell), cell);
                    self.write(cell, Some(material_id));
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
        let center = self.center(cell);
        let evaluation = self.lattice.evaluate(shape3d_id, center);

        if evaluation.distance.is_nan() {
            return Err(format!(
                "distance must be a number, not NaN at [{}, {}, {}]",
                center.x, center.y, center.z
            ));
        }

        Ok(evaluation)
    }

    /// The center of `cell` where the shapes evaluate, moved back by the
    /// shift.
    fn center(&self, cell: TyVector3I32) -> TyVector3F64 {
        (cell.as_dvec3() + 0.5) * self.voxel_size - self.shift
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
                cell,
                self.voxel_size,
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
