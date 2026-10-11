#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One chunk's voxels in a contents file older than version 4, as stored: a
/// byte per voxel, its color, or in a chunk of two-byte voxels its material
/// byte then its color, in Morton order across the chunk's 32-voxel cube.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize), serde(transparent))]
pub struct VMaxLegacyChunkVoxels(
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] pub Vec<u8>,
);
