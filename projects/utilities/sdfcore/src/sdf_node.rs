use crate::{BSdfNode, BSdfObject, SdfAxisRotation};
use branded_id::U32Id;
use ty_math::TyVector3F64;

/// A part's place in the hierarchy: an entry of
/// [`SdfState::nodes`](crate::SdfState::nodes).
#[derive(Clone, Debug, PartialEq)]
pub struct SdfNode {
    /// The part's name.
    pub name: String,

    /// The joint the part turns about, in meters.
    pub pivot: Option<TyVector3F64>,

    /// How far the part moves, in meters.
    pub offset: Option<TyVector3F64>,

    /// The rotations of the part about its pivot, in order.
    pub rotation: Vec<SdfAxisRotation>,

    /// The part's object, when its list holds steps.
    pub child_object_ids: Vec<U32Id<BSdfObject>>,

    /// The nodes of the part's child parts.
    pub child_node_ids: Vec<U32Id<BSdfNode>>,
}
