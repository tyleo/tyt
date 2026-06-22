use std::fmt::Debug;

/// Selects the in-memory representation of a [`VoxjFile`](crate::VoxjFile)'s
/// objects, so one document hierarchy serves both the serde form (objects carry
/// encoded position and sample blocks) and the codec form (objects carry
/// decoded geometry).
pub trait VoxjBackend {
    /// The object representation: [`VoxjSerdeObject`](crate::VoxjSerdeObject) for the
    /// serde backend, [`VoxjCodecObject`](crate::VoxjCodecObject) for the codec
    /// backend. The serde (de)serializability the encoded form needs is required
    /// per-backend by [`VoxjMain`](crate::VoxjMain) / [`VoxjFile`](crate::VoxjFile),
    /// not here, so the codec object stays free of serde.
    type Object: Clone + Debug + PartialEq;
}
