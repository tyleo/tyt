use crate::{
    VXSnapshotIdSerde, VXSnapshotSerde, VXStatsSerde, VXStorageSerde, decode_morton_3d,
    encode_morton_3d,
};
use std::collections::BTreeMap;
use vmax::VMaxVoxel;

/// Voxel pitch of a chunk along each axis; chunks tile an 8×8×8 grid into a
/// 256³ model.
const CHUNK_PITCH: i32 = 32;

/// Snapshot type written for every rebuilt chunk (4 = checkpoint).
const CHECKPOINT: i64 = 4;

/// Encodes voxels into a `VXObjectData`'s `snapshots` array — the inverse of
/// [`VXObjectDataSerde::voxels`](crate::VXObjectDataSerde::voxels). Groups voxels
/// into the 32-pitch 8×8×8 chunk grid, lays each chunk's voxels out by in-chunk
/// Morton code into a dense 2-bytes-per-slot `(material, color)` stream (gaps left
/// as `color == 0` empty), and emits one checkpoint snapshot per occupied chunk
/// with the per-chunk statistics Voxel Max expects.
pub fn encode_snapshots(voxels: &[VMaxVoxel]) -> Vec<VXSnapshotSerde> {
    let mut chunks: BTreeMap<u32, BTreeMap<u32, (u8, u8)>> = BTreeMap::new();
    for voxel in voxels {
        let grid = [
            (voxel.x / CHUNK_PITCH) as u32,
            (voxel.y / CHUNK_PITCH) as u32,
            (voxel.z / CHUNK_PITCH) as u32,
        ];
        let local = [
            (voxel.x % CHUNK_PITCH) as u32,
            (voxel.y % CHUNK_PITCH) as u32,
            (voxel.z % CHUNK_PITCH) as u32,
        ];
        chunks
            .entry(encode_morton_3d(grid))
            .or_default()
            .insert(encode_morton_3d(local), (voxel.material, voxel.color));
    }

    chunks
        .into_iter()
        .map(|(chunk_id, slots)| {
            let min_morton = *slots.keys().next().expect("non-empty chunk");
            let max_morton = *slots.keys().next_back().expect("non-empty chunk");
            let count = slots.len() as i64;

            // In-chunk bounding box of the occupied voxels.
            let mut low = [u32::MAX; 3];
            let mut high = [0u32; 3];
            for &morton in slots.keys() {
                let local = decode_morton_3d(morton);
                for axis in 0..3 {
                    low[axis] = low[axis].min(local[axis]);
                    high[axis] = high[axis].max(local[axis]);
                }
            }

            let mut ds = vec![0u8; 2 * (max_morton - min_morton + 1) as usize];
            for (morton, (material, color)) in slots {
                let slot = (morton - min_morton) as usize;
                ds[2 * slot] = material;
                ds[2 * slot + 1] = color;
            }

            let min = vec![
                low[0] as i64,
                low[1] as i64,
                low[2] as i64,
                min_morton as i64,
            ];
            let max = vec![
                high[0] as i64,
                high[1] as i64,
                high[2] as i64,
                max_morton as i64,
            ];
            let extent = (0..3).map(|a| (high[a] - low[a] + 1) as i64).collect();
            VXSnapshotSerde {
                s: VXStorageSerde {
                    id: VXSnapshotIdSerde {
                        c: chunk_id,
                        s: 0,
                        t: CHECKPOINT,
                    },
                    ds,
                    st: VXStatsSerde {
                        min: min.clone(),
                        max: max.clone(),
                        extent,
                        count,
                        smin: min,
                        smax: max,
                        scount: count,
                    },
                    lc: vec![0u8; 256],
                    dlc: vec![0u8; 256],
                },
            }
        })
        .collect()
}
