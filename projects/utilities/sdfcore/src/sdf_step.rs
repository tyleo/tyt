use crate::{BSdfShape3d, BSdfStep, SdfSide, SdfStepMaterial};
use branded_id::U32Id;
use ty_math::TyVector3F64;

/// One step of a part's list: an entry of
/// [`SdfState::steps`](crate::SdfState::steps). A field left `None` takes its
/// default.
#[derive(Clone, Debug, PartialEq)]
pub enum SdfStep {
    /// Fills the shape's cells.
    Add {
        name: String,

        shape_id: U32Id<BSdfShape3d>,

        material: SdfStepMaterial,
    },

    /// Empties the shape's cells.
    Carve {
        name: String,

        shape_id: U32Id<BSdfShape3d>,
    },

    /// Recolors the live cells whose point `depth` meters toward a listed side
    /// lies outside the part's earlier shapes. `onto_ids` limits it to the
    /// cells those `add` steps own.
    Coat {
        name: String,

        material: SdfStepMaterial,

        sides: Option<Vec<SdfSide>>,

        depth: f64,

        within_id: Option<U32Id<BSdfShape3d>>,

        onto_ids: Option<Vec<U32Id<BSdfStep>>>,
    },

    /// Recolors the live cells inside the shape. `onto_ids` limits it to the
    /// cells those `add` steps own.
    Paint {
        name: String,

        shape_id: U32Id<BSdfShape3d>,

        material: SdfStepMaterial,

        onto_ids: Option<Vec<U32Id<BSdfStep>>>,
    },

    /// Fills a cube `size` wide around each point.
    Set {
        name: String,

        points: Vec<TyVector3F64>,

        size: f64,

        material: SdfStepMaterial,
    },
}

impl SdfStep {
    /// The step's name.
    pub fn name(&self) -> &str {
        match self {
            SdfStep::Add { name, .. }
            | SdfStep::Carve { name, .. }
            | SdfStep::Coat { name, .. }
            | SdfStep::Paint { name, .. }
            | SdfStep::Set { name, .. } => name,
        }
    }
}
