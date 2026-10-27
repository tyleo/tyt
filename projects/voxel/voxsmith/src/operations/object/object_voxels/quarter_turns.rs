/// How far [`rotate_object_voxels`](crate::operations::object::rotate_object_voxels)
/// turns, in quarter turns by the right-hand rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuarterTurns {
    /// A quarter turn.
    One,

    /// A half turn.
    Two,

    /// Three quarter turns.
    Three,
}
