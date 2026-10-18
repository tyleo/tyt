/// The traversal the GPU runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpuWalk {
    /// voxrender's integer DDA in i32.
    Integer,

    /// An f32 DDA over the same inputs, for the record.
    Float,
}
