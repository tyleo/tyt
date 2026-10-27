/// The outcome of one validation check from
/// [`check_voxj_file`](crate::validation::check_voxj_file()).
#[derive(Clone, Debug, PartialEq)]
pub enum VoxjCheckStatus {
    /// The check ran and found no problems.
    Passed,

    /// The check ran and found one or more problems, one message each, in
    /// discovery order.
    Failed(Vec<String>),

    /// The check is an authoring invariant no document can witness, so it is
    /// neither passed nor failed: whether the sample channels follow the
    /// position block's voxel order.
    Unverifiable,
}
