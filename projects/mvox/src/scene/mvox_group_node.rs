/// A group node (`nGRP`): groups a set of child nodes referenced by id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MVoxGroupNode {
    /// The ids of the child nodes.
    pub children: Vec<i32>,
}
