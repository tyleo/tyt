/// Voxel pitch of a chunk along each axis.
pub const CHUNK_PITCH: i32 = 32;

/// The voxels a chunk holds.
pub const CHUNK_VOXELS: usize = (CHUNK_PITCH * CHUNK_PITCH * CHUNK_PITCH) as usize;
