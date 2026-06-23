use std::fmt::Debug;

/// Selects the in-memory representation of a [`VoxjFile`](crate::VoxjFile)'s
/// objects, so one document hierarchy serves both the serde form (objects carry
/// encoded position and sample blocks) and the codec form (objects carry
/// decoded geometry).
pub trait VoxjBackend {
    /// The object representation: [`VoxjSerdeObject`](crate::VoxjSerdeObject) for the
    /// serde backend, [`VoxjCodecObject`](crate::VoxjCodecObject) for the codec
    /// backend. Serde bounds are required per-backend by
    /// [`VoxjMain`](crate::VoxjMain) / [`VoxjFile`](crate::VoxjFile), not here.
    type Object: Clone + Debug + PartialEq;
}
