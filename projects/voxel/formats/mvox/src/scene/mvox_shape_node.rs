use crate::MVoxShapeModel;

/// A shape node (`nSHP`): draws one or more models.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MVoxShapeNode {
    /// The models this shape draws.
    pub models: Vec<MVoxShapeModel>,
}
