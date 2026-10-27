use crate::{BSdfMaterial, BSdfPattern};
use branded_id::U32Id;

/// What a step writes into its cells.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SdfStepMaterial {
    /// One material in every cell.
    Material(U32Id<BSdfMaterial>),

    /// A pattern picking a material per cell.
    Pattern(U32Id<BSdfPattern>),
}
