use crate::utilities::{FillMode, FlattenMode};

/// The options [`to_vox_main`](crate::operations::sdf_doc::to_vox_main()) writes
/// under.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfVoxMainOptions {
    /// How much of each part's body the objects keep.
    pub fill_mode: FillMode,

    /// How much of the part hierarchy the document flattens. Flattening needs
    /// grids sampled under the world frame.
    pub flatten: FlattenMode,
}
