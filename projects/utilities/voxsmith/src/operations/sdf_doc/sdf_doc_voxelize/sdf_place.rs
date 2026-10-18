use branded_id::U32Id;
use sdfcore::BSdfNode;
use ty_math::{TyQuaternionF64, TyVector3F64};

/// One place of a part in the hierarchy.
#[derive(Clone, Debug, PartialEq)]
pub struct SdfPlace {
    /// The part's node.
    pub node_id: U32Id<BSdfNode>,

    /// The index in the sampling's `places` of the place this one sits under,
    /// or `None` for a root part.
    pub parent: Option<usize>,

    /// The part names from the root part down to this one.
    pub path: Vec<String>,

    /// The joint the part turns about, in meters, where the offsets on the
    /// place's path move it.
    pub pivot: TyVector3F64,

    /// The offsets on the place's path added up, in meters.
    pub offset: TyVector3F64,

    /// The part's rotation about its pivot on the parent's axes.
    pub rotation: TyQuaternionF64,

    /// The indices in the sampling's `grids` of the grids of the part's objects
    /// at this place.
    pub grid_indices: Vec<usize>,
}
