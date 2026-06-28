use ty_math::{TyVector3I32, TyVector3U32};

/// One object's edit grid: the author's build volume, which contains the
/// runtime grid. Distinct from the runtime grid only when it carries margin
/// around the object.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VoxEditObject {
    /// Size of the edit grid in voxels.
    pub bounds: TyVector3U32,

    /// Translation from the placing hierarchy node to the edit grid's min
    /// corner, in voxels.
    pub origin: TyVector3I32,
}
