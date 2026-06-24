use crate::VoxError;

/// A `Result` whose error is a voxcore [`VoxError`].
pub type VoxResult<T> = Result<T, VoxError>;
