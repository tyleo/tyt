use crate::operations::sdf_doc::SdfCellBounds;
use ty_math::TyVector3I32;

/// What one step wrote as it ran.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SdfStepRecord {
    /// The position in the grid's `cells` of each cell the step wrote, in write
    /// order: every cell an `add` or a `set` filled, every live cell a `carve`
    /// emptied, and every live cell a `paint` or a `coat` recolored.
    pub written: Vec<u32>,

    /// The box around the cells the step wrote, or `None` when it wrote none.
    pub written_bounds: Option<SdfCellBounds>,

    /// The index in the object's step list of each earlier step whose cells
    /// the step changed, in list order.
    pub overwritten_steps: Vec<u32>,
}

impl SdfStepRecord {
    /// Records `cell` as written. `index` gives its position in the grid's
    /// `cells`, and `previous_step` the step that changed it last.
    pub fn push(&mut self, index: u32, cell: TyVector3I32, previous_step: Option<u32>) {
        self.written.push(index);

        self.written_bounds = Some(self.written_bounds.map_or(
            SdfCellBounds {
                min: cell,
                max: cell,
            },
            |bounds| bounds.with(cell),
        ));

        if let Some(previous_step) = previous_step
            && let Err(at) = self.overwritten_steps.binary_search(&previous_step)
        {
            self.overwritten_steps.insert(at, previous_step);
        }
    }
}
