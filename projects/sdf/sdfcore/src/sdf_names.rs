use crate::{BSdfMaterial, BSdfNode, BSdfPattern, BSdfShape2d, BSdfShape3d, BSdfStep};
use branded_id::U32Id;

/// The names a model gives its entries. Each table holds a name at most once,
/// and one entry can take several names.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SdfNames {
    /// Names of 3D shapes.
    pub shapes3d: Vec<(String, U32Id<BSdfShape3d>)>,

    /// Names of 2D shapes.
    pub shapes2d: Vec<(String, U32Id<BSdfShape2d>)>,

    /// Names of materials.
    pub materials: Vec<(String, U32Id<BSdfMaterial>)>,

    /// Names of patterns.
    pub patterns: Vec<(String, U32Id<BSdfPattern>)>,

    /// Names of steps.
    pub steps: Vec<(String, U32Id<BSdfStep>)>,

    /// Names of parts, each beside the node the part writes.
    pub parts: Vec<(String, U32Id<BSdfNode>)>,
}
